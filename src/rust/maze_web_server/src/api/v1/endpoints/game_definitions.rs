//! Stored 3D game-definition endpoints under `/api/v1/game-definitions`.
//!
//! A game definition is a parametric, reproducible 3D game: it stores no maze
//! grid, only an opaque client-owned `config` blob plus a first-class,
//! server-minted `seed` from which the client regenerates the whole game. This
//! module owns the definition CRUD, the share grants, and the **publish
//! lifecycle**:
//!
//!   * **Access policy lives here, not in storage.** Storage exposes owner-scoped
//!     mutations, scoped reads, and two unconditional primitives
//!     (`get_game_definition`, `get_definition_grantees`); the server composes
//!     the `owner ∨ curated ∨ public ∨ granted` view decision at the handler.
//!   * **Seed is server-owned.** It is minted on create and preserved verbatim
//!     across updates — never taken from the client — so a definition's layout
//!     stays stable and its leaderboard fair.
//!   * **Setting `Curated` requires an admin;** every other visibility is
//!     owner-only (the mutations are already owner-scoped by storage).
//!   * **Publishing** (a `Private` → published transition) starts a fresh board
//!     by clearing the definition's score rows; **deleting** likewise resets the
//!     board. Unpublishing back to `Private` freezes the board (a later
//!     re-publish starts fresh again).
//!
//! `GET /{id}` is the **play-fetch**: it returns the definition with the
//! *effective* seed spliced into `config` plus the computed `challengeKey`.
//! A `Static` definition uses its fixed seed and the key `def:<id>`; a `Daily`
//! definition folds today's UTC date into both, giving a fresh, comparable board
//! each day (`def:<id>:<yyyy-mm-dd>`).

use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{
        ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound,
        ErrorUnauthorized,
    },
};
use chrono::{Datelike, NaiveDate, Utc};
use data_model::{GameDefinition, Rotation, User, Visibility};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

/// Request body for creating or updating a game definition. The server owns the
/// `id`, `seed`, `ownerId`, image, and timestamps — none are read from here — so
/// the caller supplies only the editable presentation + generation fields.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinitionRequest {
    /// Display name (unique per owner, non-empty).
    pub name: String,
    /// Optional description shown wherever the game appears.
    #[serde(default)]
    pub description: Option<String>,
    /// Access tier. Defaults to `private`. Setting `curated` requires an admin.
    #[serde(default)]
    #[schema(value_type = String)]
    pub visibility: Visibility,
    /// Layout/board rotation policy. Defaults to `static`.
    #[serde(default)]
    #[schema(value_type = String)]
    pub rotation: Rotation,
    /// Opaque, client-owned generation + render parameters. Stored and forwarded
    /// verbatim; only its byte size is validated (by the storage layer).
    #[schema(value_type = Object)]
    pub config: serde_json::Value,
}

/// A list of game definitions the caller may see — the merge of their own, those
/// shared with them, and every public / curated definition.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinitionListResponse {
    /// The visible definitions, de-duplicated and ordered by name.
    pub definitions: Vec<GameDefinition>,
}

/// The play-fetch response for a single definition: the stored definition with
/// the **effective seed** spliced into `config`, plus the computed leaderboard
/// subject and whether that board is tracked.
#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayResponse {
    /// The definition, with `config.seed` replaced by the effective seed for
    /// this fetch — the fixed seed for a `Static` game, the date-mixed seed for
    /// a `Daily` one.
    #[serde(flatten)]
    pub definition: GameDefinition,
    /// The leaderboard subject to record runs against: `def:<id>` for `Static`,
    /// `def:<id>:<yyyy-mm-dd>` (today, UTC) for `Daily`.
    pub challenge_key: String,
    /// Whether runs are leaderboard-tracked — true once published (`Public`,
    /// `Curated`, or `Shared` with at least one grantee).
    pub leaderboard_tracked: bool,
}

/// Request body for granting a share.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GrantShareRequest {
    /// The user to grant access to.
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Uuid,
}

/// The current grantee list for a definition, returned by the share endpoints.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionSharesResponse {
    /// The user ids currently granted access.
    #[schema(value_type = Vec<String>)]
    pub grantees: Vec<Uuid>,
}

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

/// The `challenge` prefix under which a definition's leaderboard rows live —
/// `def:<id>` for the `Static` board, and the parent of every `def:<id>:<date>`
/// `Daily` board. A board reset sweeps this prefix.
fn challenge_prefix(id: Uuid) -> String {
    format!("def:{id}")
}

fn get_authorized_user(req: &HttpRequest, admin_required: bool) -> Result<User, Error> {
    match req.extensions().get::<User>() {
        Some(user) if !admin_required || user.is_admin => Ok(user.clone()),
        _ => Err(ErrorUnauthorized("Unauthorized request")),
    }
}

/// Mints a fresh generation seed. Server-owned so a definition's layout is
/// stable and its leaderboard fair; never taken from the client.
fn mint_seed() -> u64 {
    rand_core::OsRng.next_u64()
}

/// Whether a definition is *published* — accessible to anyone but the owner and
/// therefore leaderboard-tracked. `Shared` counts only once it has a grantee.
fn is_published(visibility: Visibility, grantees: &[Uuid]) -> bool {
    match visibility {
        Visibility::Public | Visibility::Curated => true,
        Visibility::Shared => !grantees.is_empty(),
        Visibility::Private => false,
    }
}

/// Whether `viewer` may see/play `def`, composed from the storage primitives:
/// the owner always may; `Curated`/`Public` are open to any signed-in user; a
/// `Shared` definition is open to its explicit grantees.
fn can_view(def: &GameDefinition, viewer: &User, grantees: &[Uuid]) -> bool {
    def.owner_id == viewer.id
        || matches!(def.visibility, Visibility::Curated | Visibility::Public)
        || (def.visibility == Visibility::Shared && grantees.contains(&viewer.id))
}

/// Folds a UTC date into a base seed to derive a `Daily` definition's per-day
/// seed — a splitmix64 finalizer over `seed XOR date`. Distinct dates yield
/// distinct seeds while a given date is stable, so each day gets a fresh but
/// reproducible layout.
fn mix_seed(seed: u64, date: NaiveDate) -> u64 {
    let day = date.num_days_from_ce() as u64;
    let mut x = seed ^ day.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Computes a definition's leaderboard subject key and the effective generation
/// seed for `today` (UTC). `Static` uses the fixed seed and a date-less key;
/// `Daily` folds the date into both.
fn compute_play_subject(def: &GameDefinition, today: NaiveDate) -> (String, u64) {
    match def.rotation {
        Rotation::Static => (challenge_prefix(def.id), def.seed),
        Rotation::Daily => (
            format!("def:{}:{}", def.id, today.format("%Y-%m-%d")),
            mix_seed(def.seed, today),
        ),
    }
}

/// Replaces the `seed` field of an opaque `config` object with `seed`, so the
/// client regenerates the layout the server's leaderboard subject assumes. A
/// non-object `config` is left unchanged.
fn splice_seed(mut config: serde_json::Value, seed: u64) -> serde_json::Value {
    if let Some(object) = config.as_object_mut() {
        object.insert("seed".to_string(), serde_json::json!(seed));
    }
    config
}

/// Maps a create/update store error to its HTTP status, falling back to a
/// logged 500 for anything unexpected.
fn map_write_error(err: StoreError) -> Error {
    match err {
        StoreError::GameDefinitionIdNotFound(id) => {
            ErrorNotFound(format!("Game definition '{id}' not found"))
        }
        StoreError::GameDefinitionNameMissing() => {
            ErrorBadRequest("Game definition name must not be empty")
        }
        StoreError::GameDefinitionNameAlreadyExists(name) => {
            ErrorConflict(format!("A game definition named '{name}' already exists"))
        }
        StoreError::GameDefinitionConfigTooLarge { bytes, max } => ErrorBadRequest(format!(
            "Game definition config is too large ({bytes} bytes; max {max})"
        )),
        other => {
            log::warn!("game definition store error: {other}");
            ErrorInternalServerError("Failed to store game definition")
        }
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/game-definitions
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Create a game definition",
    description = "Creates a stored 3D game definition owned by the caller. The server mints the \
                   generation seed and sets the id/owner/timestamps; the caller supplies name, \
                   description, visibility, rotation, and the opaque config. Setting visibility to \
                   'curated' requires an admin. Creating it published (shared/public/curated) makes \
                   it leaderboard-tracked immediately.",
    post,
    path = "/api/v1/game-definitions",
    request_body = GameDefinitionRequest,
    responses(
        (status = 201, description = "Definition created", body = GameDefinition),
        (status = 400, description = "Invalid request (empty name or over-size config)"),
        (status = 401, description = "Unauthorized request"),
        (status = 403, description = "Only an admin may set 'curated'"),
        (status = 409, description = "A definition with that name already exists")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-definitions")]
pub async fn create_game_definition(
    body: web::Json<GameDefinitionRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let body = body.into_inner();

    if body.visibility == Visibility::Curated && !user.is_admin {
        return Err(ErrorForbidden("Only an admin may set 'curated' visibility"));
    }

    let now = Utc::now();
    let mut definition = GameDefinition {
        id: Uuid::nil(),
        owner_id: Uuid::nil(),
        name: body.name,
        description: body.description,
        visibility: body.visibility,
        seed: mint_seed(),
        rotation: body.rotation,
        config: body.config,
        image_updated_at: None,
        created_at: now,
        updated_at: now,
    };

    let mut store_lock = store.write().await;
    match store_lock.create_game_definition(&user, &mut definition).await {
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Location", format!("/api/v1/game-definitions/{}", definition.id)))
            .json(definition)),
        Err(err) => Err(map_write_error(err)),
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/game-definitions  (list)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "List visible game definitions",
    description = "Returns every game definition the caller may see — the merge of their own \
                   (all visibilities, drafts included), those shared with them, and all public and \
                   curated definitions — de-duplicated and ordered by name.",
    get,
    path = "/api/v1/game-definitions",
    responses(
        (status = 200, description = "The visible definitions", body = GameDefinitionListResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions")]
pub async fn list_game_definitions(
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let store_lock = store.read().await;

    let mut merged: Vec<GameDefinition> = Vec::new();
    for source in [
        store_lock.get_definitions_for_owner(&user).await,
        store_lock.get_definitions_shared_with(user.id).await,
        store_lock.get_public_definitions().await,
        store_lock.get_curated_definitions().await,
    ] {
        match source {
            Ok(defs) => merged.extend(defs),
            Err(err) => {
                log::warn!("list game definitions store error: {err}");
                return Err(ErrorInternalServerError("Failed to list game definitions"));
            }
        }
    }

    // The same definition can arrive from several sources (e.g. an owned curated
    // one). Keep the first occurrence per id, then order by name for a stable list.
    let mut seen = std::collections::HashSet::new();
    merged.retain(|def| seen.insert(def.id));
    merged.sort_by_key(|def| def.name.to_lowercase());

    Ok(HttpResponse::Ok().json(GameDefinitionListResponse { definitions: merged }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/game-definitions/{id}  (play-fetch)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Fetch a game definition to play",
    description = "Returns a single definition the caller may access (owner, curated, public, or \
                   granted). The config is returned with the effective seed spliced in and the \
                   leaderboard subject computed: a Static game uses its fixed seed and the key \
                   'def:<id>'; a Daily game folds today's UTC date into both, yielding \
                   'def:<id>:<yyyy-mm-dd>'. An inaccessible or unknown definition returns 404.",
    get,
    path = "/api/v1/game-definitions/{id}",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "The definition, ready to play", body = GamePlayResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found or not accessible")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions/{id}")]
pub async fn get_game_definition(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;

    let mut definition = match store_lock.get_game_definition(id).await {
        Ok(def) => def,
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game definition store error: {err}");
            return Err(ErrorInternalServerError("Failed to load game definition"));
        }
    };

    let grantees = store_lock.get_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load game definition")
    })?;

    // Access is composed here, not in storage. An inaccessible definition is
    // reported as absent so its existence is not leaked.
    if !can_view(&definition, &user, &grantees) {
        return Err(ErrorNotFound(format!("Game definition '{id}' not found")));
    }

    let (challenge_key, effective_seed) = compute_play_subject(&definition, Utc::now().date_naive());
    let leaderboard_tracked = is_published(definition.visibility, &grantees);
    definition.config = splice_seed(definition.config, effective_seed);

    Ok(HttpResponse::Ok().json(GamePlayResponse {
        definition,
        challenge_key,
        leaderboard_tracked,
    }))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/game-definitions/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Update a game definition",
    description = "Updates a definition owned by the caller. The seed and image are server-owned \
                   and preserved; name, description, visibility, rotation, and config are replaced. \
                   Setting visibility to 'curated' requires an admin. Publishing (a private → \
                   shared/public/curated transition) starts a fresh leaderboard by clearing the \
                   definition's score rows; unpublishing back to private freezes the board.",
    put,
    path = "/api/v1/game-definitions/{id}",
    params(("id" = String, Path, description = "Definition id")),
    request_body = GameDefinitionRequest,
    responses(
        (status = 200, description = "Definition updated", body = GameDefinition),
        (status = 400, description = "Invalid request (empty name or over-size config)"),
        (status = 401, description = "Unauthorized request"),
        (status = 403, description = "Only an admin may set 'curated'"),
        (status = 404, description = "Definition not found"),
        (status = 409, description = "A definition with that name already exists")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-definitions/{id}")]
pub async fn update_game_definition(
    path: web::Path<Uuid>,
    body: web::Json<GameDefinitionRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let body = body.into_inner();

    if body.visibility == Visibility::Curated && !user.is_admin {
        return Err(ErrorForbidden("Only an admin may set 'curated' visibility"));
    }

    let mut store_lock = store.write().await;

    // Load the existing record to preserve the server-owned fields and to detect
    // a publish transition. A record owned by someone else is reported as absent.
    let existing = match store_lock.get_game_definition(id).await {
        Ok(def) if def.owner_id == user.id => def,
        Ok(_) | Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game definition store error: {err}");
            return Err(ErrorInternalServerError("Failed to load game definition"));
        }
    };

    let mut definition = GameDefinition {
        id: existing.id,
        owner_id: user.id,
        name: body.name,
        description: body.description,
        visibility: body.visibility,
        seed: existing.seed,                       // server-owned — preserved
        rotation: body.rotation,
        config: body.config,
        image_updated_at: existing.image_updated_at, // image bytes managed separately
        created_at: existing.created_at,
        updated_at: existing.updated_at,
    };

    store_lock
        .update_game_definition(&user, &mut definition)
        .await
        .map_err(map_write_error)?;

    // Publishing an unpublished (Private) definition starts a fresh board — a
    // re-publish must not inherit a prior board's rows (decision: unpublish
    // freezes, re-publish starts fresh).
    if existing.visibility == Visibility::Private && definition.visibility != Visibility::Private {
        if let Err(err) = store_lock.clear_challenge_scores_prefix(&challenge_prefix(id)).await {
            log::warn!("failed to reset board on publish for definition {id}: {err}");
        }
    }

    Ok(HttpResponse::Ok().json(definition))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/game-definitions/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Delete a game definition",
    description = "Deletes a definition owned by the caller, removing it and its share grants, and \
                   resets its leaderboard(s) by clearing every score row under its subject prefix.",
    delete,
    path = "/api/v1/game-definitions/{id}",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "Definition deleted"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-definitions/{id}")]
pub async fn delete_game_definition(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut store_lock = store.write().await;

    match store_lock.delete_game_definition(&user, id).await {
        Ok(()) => {
            // The definition is gone and was the caller's — safe to reset its
            // orphaned board rows.
            if let Err(err) = store_lock.clear_challenge_scores_prefix(&challenge_prefix(id)).await {
                log::warn!("failed to reset board on delete for definition {id}: {err}");
            }
            Ok(HttpResponse::Ok().body(format!("game definition '{id}' deleted")))
        }
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("delete game definition store error: {err}");
            Err(ErrorInternalServerError("Failed to delete game definition"))
        }
    }
}

// ---------------------------------------------------------------------------
// Share management — GET / PUT / DELETE /api/v1/game-definitions/{id}/shares
// ---------------------------------------------------------------------------

/// Loads the current grantee list for a definition owned by `user`, or a 404 if
/// the definition is absent or owned by someone else. Shared by the three share
/// endpoints so a non-owner cannot probe another user's grants.
async fn owned_definition_grantees(
    store_lock: &dyn storage::Store,
    user: &User,
    id: Uuid,
) -> Result<Vec<Uuid>, Error> {
    match store_lock.get_game_definition(id).await {
        Ok(def) if def.owner_id == user.id => {}
        Ok(_) | Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game definition store error: {err}");
            return Err(ErrorInternalServerError("Failed to load game definition"));
        }
    }
    store_lock.get_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load definition shares")
    })
}

#[utoipa::path(
    summary = "List a definition's grantees",
    description = "Returns the user ids granted access to a definition owned by the caller — the \
                   owner's manage-shares view. A definition owned by someone else returns 404.",
    get,
    path = "/api/v1/game-definitions/{id}/shares",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "The current grantees", body = DefinitionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions/{id}/shares")]
pub async fn list_definition_shares(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;
    let grantees = owned_definition_grantees(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(DefinitionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Grant a user access to a definition",
    description = "Grants the given user access to a definition owned by the caller (idempotent) \
                   and returns the updated grantee list. A definition owned by someone else \
                   returns 404.",
    put,
    path = "/api/v1/game-definitions/{id}/shares",
    params(("id" = String, Path, description = "Definition id")),
    request_body = GrantShareRequest,
    responses(
        (status = 200, description = "Access granted", body = DefinitionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-definitions/{id}/shares")]
pub async fn grant_definition_share(
    path: web::Path<Uuid>,
    body: web::Json<GrantShareRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let grantee = body.into_inner().user_id;
    let mut store_lock = store.write().await;

    match store_lock.grant_definition_access(&user, id, grantee).await {
        Ok(()) => {}
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("grant definition access store error: {err}");
            return Err(ErrorInternalServerError("Failed to grant access"));
        }
    }

    let grantees = store_lock.get_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load definition shares")
    })?;
    Ok(HttpResponse::Ok().json(DefinitionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Revoke a user's access to a definition",
    description = "Revokes the given user's access to a definition owned by the caller (idempotent) \
                   and returns the updated grantee list. A definition owned by someone else \
                   returns 404.",
    delete,
    path = "/api/v1/game-definitions/{id}/shares/{grantee}",
    params(
        ("id" = String, Path, description = "Definition id"),
        ("grantee" = String, Path, description = "The user id to revoke")
    ),
    responses(
        (status = 200, description = "Access revoked", body = DefinitionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-definitions/{id}/shares/{grantee}")]
pub async fn revoke_definition_share(
    path: web::Path<(Uuid, Uuid)>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let (id, grantee) = path.into_inner();
    let mut store_lock = store.write().await;

    match store_lock.revoke_definition_access(&user, id, grantee).await {
        Ok(()) => {}
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("revoke definition access store error: {err}");
            return Err(ErrorInternalServerError("Failed to revoke access"));
        }
    }

    let grantees = store_lock.get_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load definition shares")
    })?;
    Ok(HttpResponse::Ok().json(DefinitionSharesResponse { grantees }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn definition(rotation: Rotation, seed: u64) -> GameDefinition {
        let ts = Utc::now();
        GameDefinition {
            id: Uuid::from_u128(0x1234),
            owner_id: Uuid::nil(),
            name: "n".to_string(),
            description: None,
            visibility: Visibility::Public,
            seed,
            rotation,
            config: serde_json::json!({ "rows": 5, "seed": 0 }),
            image_updated_at: None,
            created_at: ts,
            updated_at: ts,
        }
    }

    #[test]
    fn static_subject_uses_fixed_seed_and_dateless_key() {
        let def = definition(Rotation::Static, 42);
        let day = NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
        let (key, seed) = compute_play_subject(&def, day);
        assert_eq!(key, format!("def:{}", def.id));
        assert_eq!(seed, 42);
    }

    #[test]
    fn daily_subject_folds_the_date_into_key_and_seed() {
        let def = definition(Rotation::Daily, 42);
        let day = NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
        let (key, seed) = compute_play_subject(&def, day);
        assert_eq!(key, format!("def:{}:2026-07-05", def.id));
        // The daily seed is mixed, not the base seed.
        assert_ne!(seed, 42);
        assert_eq!(seed, mix_seed(42, day));
    }

    #[test]
    fn two_utc_dates_yield_distinct_daily_keys_and_seeds() {
        let def = definition(Rotation::Daily, 42);
        let day_one = NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
        let day_two = NaiveDate::from_ymd_opt(2026, 7, 6).unwrap();
        let (key_one, seed_one) = compute_play_subject(&def, day_one);
        let (key_two, seed_two) = compute_play_subject(&def, day_two);
        assert_ne!(key_one, key_two);
        assert_ne!(seed_one, seed_two);
    }

    #[test]
    fn splice_seed_overwrites_the_config_seed() {
        let spliced = splice_seed(serde_json::json!({ "rows": 5, "seed": 0 }), 999);
        assert_eq!(spliced["seed"], serde_json::json!(999u64));
        assert_eq!(spliced["rows"], serde_json::json!(5));
    }

    #[test]
    fn is_published_reflects_the_publish_rule() {
        assert!(!is_published(Visibility::Private, &[]));
        assert!(!is_published(Visibility::Shared, &[]));
        assert!(is_published(Visibility::Shared, &[Uuid::nil()]));
        assert!(is_published(Visibility::Public, &[]));
        assert!(is_published(Visibility::Curated, &[]));
    }
}
