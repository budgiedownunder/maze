//! Stored 3D game-definition endpoints under `/api/v1/game-definitions`.
//!
//! A game definition is a parametric, reproducible 3D game: it stores no maze
//! grid, only an opaque client-owned `config` blob plus a first-class,
//! server-minted `seed` from which the client regenerates the whole game. This
//! module owns the definition CRUD, the share grants, and the **publish
//! lifecycle**:
//!
//!   * **Access control is enforced here, not in storage.** Storage holds the
//!     access facts (a definition's `visibility`, its grantee list) but performs
//!     no access checks — it exposes owner-scoped mutations, scoped reads, and
//!     two unconditional primitives (`get_game_definition`,
//!     `get_game_definition_grantees`). The server reads those facts and composes the
//!     `owner ∨ curated ∨ public ∨ granted` view decision at the handler.
//!   * **Seed is server-owned.** It is minted on create and preserved verbatim
//!     across updates — never taken from the client — so a definition's layout
//!     stays stable and its leaderboard fair.
//!   * **Setting `Curated` requires an admin;** every other visibility is
//!     owner-only (the mutations are already owner-scoped by storage).
//!   * **Every game is leaderboard-tracked.** A published game's board is shared
//!     with everyone who can view it; a private game's board is the owner's own
//!     (only they can reach the play-fetch). `GET /scores` enforces who may *read*
//!     a `def:<id>` board (owner ∨ curated ∨ public ∨ granted).
//!   * **The board resets only when the game changes how it plays** — a reshuffle,
//!     or a PUT that alters a gameplay-affecting config field (structure, scene,
//!     content, mechanics) or the rotation. Cosmetic edits (title, status label,
//!     `levels.hideCompletedEnemies`, name, description) keep it; publishing /
//!     unpublishing keeps it; **deleting** clears it.
//!   * **Access is set explicitly.** The owner sets a definition's tier
//!     (`visibility`, a plain `PUT`) and its share list (`PUT /shares` replaces
//!     the whole grantee list in one operation) directly — there is no implicit
//!     coupling between them. Changing the tier is not a gameplay change, so the
//!     board is untouched.
//!
//! `GET /{id}` is the **play-fetch**: it returns the definition with the
//! *effective* seed spliced into `config` plus the computed `challengeKey`.
//! A `Static` definition uses its fixed seed and the key `def:<id>`; a `Daily`
//! definition folds today's UTC date into both, giving a fresh, comparable board
//! each day (`def:<id>:<yyyy-mm-dd>`).

use actix_multipart::form::{bytes::Bytes as MultipartBytes, MultipartForm};
use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{
        ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound,
        ErrorUnauthorized,
    },
    http::header::{CacheControl, CacheDirective, ETag, EntityTag},
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use data_model::{GameDefinition, GranteeSummary, Rotation, User, Visibility};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::v1::endpoints::avatar::canonicalise_to_png;

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

/// Query parameters for the list endpoint — a page of the merged result.
#[derive(Deserialize, Debug)]
pub struct ListGameDefinitionsQuery {
    /// Page size (defaults to [`DEFAULT_PAGE_SIZE`], capped at [`MAX_PAGE_SIZE`]).
    pub limit: Option<u32>,
    /// Zero-based page offset (defaults to 0).
    pub offset: Option<u32>,
}

/// A page of the game definitions the caller may see — the merge of their own,
/// those shared with them, and every public / curated definition.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinitionListResponse {
    /// The page of visible definitions, de-duplicated and ordered by name.
    pub definitions: Vec<GameDefinition>,
    /// The effective page size applied (the request's `limit` capped at the
    /// server maximum).
    pub limit: u32,
    /// The zero-based offset this page started at.
    pub offset: u32,
    /// Whether at least one further definition exists beyond this page.
    pub has_more: bool,
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

/// Request body for setting a share list — the complete set of users who should
/// have access after the call. The server reconciles the stored list to match:
/// anyone not listed is revoked, any new id granted, in one operation.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetGameSharesRequest {
    /// The complete desired grantee list. The owner's own id, if present, is
    /// ignored (you can't share with yourself).
    #[schema(value_type = Vec<String>)]
    pub user_ids: Vec<Uuid>,
}

/// The current grantee list for a definition, returned by the share endpoints —
/// each grantee resolved to `{id, username}` for the owner's manage-shares view.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinitionSharesResponse {
    /// The users currently granted access (id + username).
    pub grantees: Vec<GranteeSummary>,
}

/// Multipart upload form for a definition / collection image — a single `file`
/// part, oversize-rejected during extraction. Shared with the collection
/// endpoints (identical shape to the avatar upload).
#[derive(MultipartForm)]
pub struct ImageUploadForm {
    #[multipart(limit = "2 MiB")]
    pub file: MultipartBytes,
}

/// `200` response for a successful image upload — the freshly-stamped
/// `image_updated_at` the client uses to cache-bust the image URL. Shared by the
/// definition + collection image endpoints.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageUpdatedResponse {
    /// The new image cache-buster (RFC 3339).
    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    pub image_updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

/// Page size used when the caller omits `limit` (mirrors the scores endpoint).
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Hard server cap on `limit` — a caller asking for more is silently capped to
/// this, and the effective value is echoed back so the client can page correctly.
const MAX_PAGE_SIZE: u32 = 100;

/// Resolves the effective page size: the caller's `limit` (or the default when
/// omitted), capped at [`MAX_PAGE_SIZE`].
fn effective_limit(requested: Option<u32>) -> u32 {
    requested.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE)
}

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

/// The gameplay-affecting projection of a config — the config minus the cosmetic
/// keys. Two configs with the same projection produce the same run, so an edit
/// that leaves the projection unchanged (splash `title`, status-bar `mode`,
/// `levels.hideCompletedEnemies`, plus the server-owned `seed`) doesn't
/// invalidate the leaderboard.
fn gameplay_signature(config: &serde_json::Value) -> serde_json::Value {
    let mut c = config.clone();
    if let Some(obj) = c.as_object_mut() {
        obj.remove("title");
        obj.remove("mode");
        obj.remove("seed");
        if let Some(levels) = obj.get_mut("levels").and_then(|v| v.as_object_mut()) {
            levels.remove("hideCompletedEnemies");
        }
    }
    c
}

/// The definition id behind a `def:<id>` (or `def:<id>:<date>`) leaderboard
/// challenge, or `None` for any other challenge namespace.
fn parse_definition_challenge(challenge: &str) -> Option<Uuid> {
    Uuid::parse_str(challenge.strip_prefix("def:")?.split(':').next()?).ok()
}

/// Whether `user` may read the leaderboard behind `challenge`. A `def:<id>` board
/// is gated by the same view rule as the game (owner ∨ public ∨ curated ∨ granted)
/// so a private game's board stays owner-only; every other challenge namespace is
/// readable by any authenticated caller (the difficulty boards are global).
pub(crate) async fn can_read_challenge_board(
    store: &dyn storage::Store,
    user: &User,
    challenge: &str,
) -> bool {
    let Some(id) = parse_definition_challenge(challenge) else {
        return true;
    };
    let Ok(def) = store.get_game_definition(id).await else {
        return false;
    };
    let grantees = store.get_game_definition_grantees(id).await.unwrap_or_default();
    can_view(&def, user, &grantees)
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
        StoreError::GameDefinitionCountLimitReached { count, max } => ErrorConflict(format!(
            "Game definition limit reached: you already own {count} definitions (max {max})"
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
    description = "Returns a page of the game definitions the caller may see — their own (all \
                   visibilities, drafts included), those shared with them, and all public and \
                   curated definitions — de-duplicated and ordered by name. Paged via limit \
                   (server-capped) and offset.",
    get,
    path = "/api/v1/game-definitions",
    params(
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)")
    ),
    responses(
        (status = 200, description = "A page of visible definitions", body = GameDefinitionListResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions")]
pub async fn list_game_definitions(
    query: web::Query<ListGameDefinitionsQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let q = query.into_inner();
    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);
    let store_lock = store.read().await;

    // Storage composes + pages the "visible to me" set. Over-fetch one row so
    // `has_more` needs no separate count.
    let mut definitions = store_lock
        .get_visible_game_definitions(&user, limit + 1, offset)
        .await
        .map_err(|err| {
            log::warn!("list game definitions store error: {err}");
            ErrorInternalServerError("Failed to list game definitions")
        })?;
    let has_more = definitions.len() as u32 > limit;
    definitions.truncate(limit as usize);

    Ok(HttpResponse::Ok().json(GameDefinitionListResponse {
        definitions,
        limit,
        offset,
        has_more,
    }))
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

    let grantees = store_lock.get_game_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load game definition")
    })?;

    // Access is composed here, not in storage. An inaccessible definition is
    // reported as absent so its existence is not leaked.
    if !can_view(&definition, &user, &grantees) {
        return Err(ErrorNotFound(format!("Game definition '{id}' not found")));
    }

    let (challenge_key, effective_seed) = compute_play_subject(&definition, Utc::now().date_naive());
    // Every game the caller can reach records scores: a published game's board is
    // shared with everyone who can view it; a private game's board is the owner's
    // own (only they can reach this fetch). The read side (GET /scores) enforces
    // who may *see* a board.
    let leaderboard_tracked = true;
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
                   Setting visibility to 'curated' requires an admin. If the edit changes a \
                   gameplay-affecting field (structure/scene/content/mechanics) or the rotation, the \
                   leaderboard is reset; cosmetic-only edits (title, status label, hide-cleared \
                   enemies, name, description) and visibility changes keep it.",
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

    // A change that alters how the game plays makes past times incomparable, so
    // its board is reset. Cosmetic-only edits (title / status label / hide-cleared
    // enemies / name / description) keep it, and changing visibility on its own
    // (publish / unpublish) never resets — only the run itself does.
    if existing.rotation != definition.rotation
        || gameplay_signature(&existing.config) != gameplay_signature(&definition.config)
    {
        if let Err(err) = store_lock.clear_challenge_scores_prefix(&challenge_prefix(id)).await {
            log::warn!("failed to reset board on gameplay change for definition {id}: {err}");
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
// POST /api/v1/game-definitions/{id}/reshuffle
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Reshuffle a game definition's layout",
    description = "Re-mints the definition's seed, changing the generated layout. The seed is \
                   otherwise server-owned and preserved across updates, so reshuffling is its own \
                   endpoint. If the definition is published its leaderboard is reset (the layout — \
                   and thus fair comparison — has changed); a private draft has no board to clear. \
                   Owner-only.",
    post,
    path = "/api/v1/game-definitions/{id}/reshuffle",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "Reshuffled; returns the definition with its new seed", body = GameDefinition),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-definitions/{id}/reshuffle")]
pub async fn reshuffle_game_definition(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut store_lock = store.write().await;

    // Load the caller's own record; anyone else's is reported as absent.
    let mut definition = match store_lock.get_game_definition(id).await {
        Ok(def) if def.owner_id == user.id => def,
        Ok(_) | Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game definition store error: {err}");
            return Err(ErrorInternalServerError("Failed to load game definition"));
        }
    };

    // A fresh seed = a fresh layout. Persist it through the owner-scoped update.
    definition.seed = mint_seed();
    store_lock
        .update_game_definition(&user, &mut definition)
        .await
        .map_err(map_write_error)?;

    // The layout changed, so any board is no longer a fair comparison — reset it.
    // A private draft is not tracked, so this is a no-op there.
    if let Err(err) = store_lock.clear_challenge_scores_prefix(&challenge_prefix(id)).await {
        log::warn!("failed to reset board on reshuffle for definition {id}: {err}");
    }

    Ok(HttpResponse::Ok().json(definition))
}

// ---------------------------------------------------------------------------
// Share management — GET / PUT / DELETE /api/v1/game-definitions/{id}/shares
// ---------------------------------------------------------------------------

/// Loads the current grantee list (resolved to `{id, username}`) for a
/// definition owned by `user`, or a 404 if the definition is absent or owned by
/// someone else. Shared by the three share endpoints so a non-owner cannot probe
/// another user's grants.
async fn owned_definition_grantees(
    store_lock: &dyn storage::Store,
    user: &User,
    id: Uuid,
) -> Result<Vec<GranteeSummary>, Error> {
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
    store_lock.get_game_definition_grantee_summaries(id).await.map_err(|err| {
        log::warn!("get definition grantee summaries store error: {err}");
        ErrorInternalServerError("Failed to load definition shares")
    })
}

#[utoipa::path(
    summary = "List a definition's grantees",
    description = "Returns the users (id + username) granted access to a definition owned by the \
                   caller — the owner's manage-shares view. A definition owned by someone else \
                   returns 404.",
    get,
    path = "/api/v1/game-definitions/{id}/shares",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "The current grantees", body = GameDefinitionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions/{id}/shares")]
pub async fn list_game_definition_shares(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;
    let grantees = owned_definition_grantees(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(GameDefinitionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Set a definition's share list",
    description = "Replaces the definition's grantee list with the supplied set in one operation — \
                   anyone not listed is revoked, any new id granted — and returns the updated list. \
                   Owner-only (a definition owned by someone else returns 404). The owner's own id \
                   is ignored.",
    put,
    path = "/api/v1/game-definitions/{id}/shares",
    params(("id" = String, Path, description = "Definition id")),
    request_body = SetGameSharesRequest,
    responses(
        (status = 200, description = "Share list updated", body = GameDefinitionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-definitions/{id}/shares")]
pub async fn set_game_definition_shares(
    path: web::Path<Uuid>,
    body: web::Json<SetGameSharesRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let user_ids = body.into_inner().user_ids;

    let mut store_lock = store.write().await;
    match store_lock.set_game_definition_grantees(&user, id, &user_ids).await {
        Ok(()) => {}
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("set definition shares store error: {err}");
            return Err(ErrorInternalServerError("Failed to update shares"));
        }
    }

    let grantees = store_lock.get_game_definition_grantee_summaries(id).await.map_err(|err| {
        log::warn!("get definition grantee summaries store error: {err}");
        ErrorInternalServerError("Failed to load definition shares")
    })?;
    Ok(HttpResponse::Ok().json(GameDefinitionSharesResponse { grantees }))
}

// ---------------------------------------------------------------------------
// Image — POST / DELETE / GET /api/v1/game-definitions/{id}/image
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Upload or replace a definition's image",
    description = "Accepts a multipart/form-data upload with a single `file` part (PNG or JPEG, up \
                   to 2 MiB). The server canonicalises it to a 256x256 PNG and stamps the \
                   definition's image_updated_at. Owner-only. Returns the new image_updated_at.",
    post,
    path = "/api/v1/game-definitions/{id}/image",
    params(("id" = String, Path, description = "Definition id")),
    request_body(content_type = "multipart/form-data", description = "A single `file` part: PNG or JPEG, <= 2 MiB"),
    responses(
        (status = 200, description = "Image stored", body = ImageUpdatedResponse),
        (status = 400, description = "Missing/invalid image or over the size limit"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-definitions/{id}/image")]
pub async fn upload_game_definition_image(
    path: web::Path<Uuid>,
    MultipartForm(form): MultipartForm<ImageUploadForm>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let png = canonicalise_to_png(&form.file.data)?;

    let mut store_lock = store.write().await;
    match store_lock.set_game_definition_image(&user, id, png).await {
        Ok(()) => {}
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("set definition image store error: {err}");
            return Err(ErrorInternalServerError("Failed to store image"));
        }
    }
    // Read the freshly-stamped marker back to return it (same write guard).
    let image_updated_at = store_lock
        .get_game_definition(id)
        .await
        .map_err(|err| {
            log::warn!("get definition after image set: {err}");
            ErrorInternalServerError("Failed to store image")
        })?
        .image_updated_at
        .ok_or_else(|| ErrorInternalServerError("image marker missing after store"))?;
    Ok(HttpResponse::Ok().json(ImageUpdatedResponse { image_updated_at }))
}

#[utoipa::path(
    summary = "Remove a definition's image",
    description = "Removes the image of a definition owned by the caller and clears its \
                   image_updated_at. Idempotent — succeeds even when no image is set.",
    delete,
    path = "/api/v1/game-definitions/{id}/image",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 204, description = "Image removed (or none was set)"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-definitions/{id}/image")]
pub async fn delete_game_definition_image(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut store_lock = store.write().await;

    // Owner-check for a clean 404 (the storage clear is a silent no-op for a
    // non-owner, so the server enforces ownership here).
    match store_lock.get_game_definition(id).await {
        Ok(def) if def.owner_id == user.id => {}
        Ok(_) | Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get definition for image delete: {err}");
            return Err(ErrorInternalServerError("Failed to delete image"));
        }
    }
    store_lock.clear_game_definition_image(&user, id).await.map_err(|err| {
        log::warn!("clear definition image store error: {err}");
        ErrorInternalServerError("Failed to delete image")
    })?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    summary = "Serve a definition's image",
    description = "Returns the definition's image as image/png, or 404 when it has none. \
                   Access-checked like the play-fetch (owner, curated, public, or granted); an \
                   inaccessible definition is reported as 404. Clients cache-bust with \
                   ?v=<imageUpdatedAt>.",
    get,
    path = "/api/v1/game-definitions/{id}/image",
    params(("id" = String, Path, description = "Definition id")),
    responses(
        (status = 200, description = "The image", content_type = "image/png"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Definition not accessible or has no image")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-definitions/{id}/image")]
pub async fn serve_game_definition_image(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;

    let definition = match store_lock.get_game_definition(id).await {
        Ok(def) => def,
        Err(StoreError::GameDefinitionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game definition '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get definition for image serve: {err}");
            return Err(ErrorInternalServerError("Failed to load image"));
        }
    };
    let grantees = store_lock.get_game_definition_grantees(id).await.map_err(|err| {
        log::warn!("get definition grantees store error: {err}");
        ErrorInternalServerError("Failed to load image")
    })?;
    if !can_view(&definition, &user, &grantees) {
        return Err(ErrorNotFound(format!("Game definition '{id}' not found")));
    }

    let Some(data) = store_lock.get_game_definition_image(id).await.map_err(|err| {
        log::warn!("get definition image store error: {err}");
        ErrorInternalServerError("Failed to load image")
    })?
    else {
        return Err(ErrorNotFound(format!("Game definition '{id}' has no image")));
    };

    let mut builder = HttpResponse::Ok();
    builder.content_type("image/png");
    builder.insert_header(CacheControl(vec![CacheDirective::Private, CacheDirective::NoCache]));
    if let Some(ts) = definition.image_updated_at {
        builder.insert_header(ETag(EntityTag::new_strong(ts.timestamp_millis().to_string())));
    }
    Ok(builder.body(data))
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
    fn gameplay_signature_ignores_cosmetic_keys() {
        let base = serde_json::json!({
            "rows": 6, "cols": 6, "seed": 1, "title": "A", "mode": "B",
            "levels": { "count": 2, "hideCompletedEnemies": true }
        });
        // Cosmetic-only edits leave the signature unchanged.
        let cosmetic = serde_json::json!({
            "rows": 6, "cols": 6, "seed": 999, "title": "Z", "mode": "Y",
            "levels": { "count": 2, "hideCompletedEnemies": false }
        });
        assert_eq!(gameplay_signature(&base), gameplay_signature(&cosmetic));

        // A gameplay field (grid, a level setting) changes the signature.
        let structural = serde_json::json!({
            "rows": 8, "cols": 6, "seed": 1, "title": "A", "mode": "B",
            "levels": { "count": 2, "hideCompletedEnemies": true }
        });
        assert_ne!(gameplay_signature(&base), gameplay_signature(&structural));
        let level_change = serde_json::json!({
            "rows": 6, "cols": 6, "seed": 1, "title": "A", "mode": "B",
            "levels": { "count": 3, "hideCompletedEnemies": true }
        });
        assert_ne!(gameplay_signature(&base), gameplay_signature(&level_change));
    }

    #[test]
    fn parse_definition_challenge_extracts_the_id() {
        let id = Uuid::from_u128(0xabc);
        assert_eq!(parse_definition_challenge(&format!("def:{id}")), Some(id));
        assert_eq!(parse_definition_challenge(&format!("def:{id}:2026-07-11")), Some(id));
        assert_eq!(parse_definition_challenge("hard:12345"), None);
        assert_eq!(parse_definition_challenge("def:not-a-uuid"), None);
    }
}
