//! Game-collection endpoints under `/api/v1/game-collections`.
//!
//! A collection is an ordered, presentation-only grouping of
//! [`GameDefinition`]s — it does not affect generation or scoring (leaderboards
//! stay per-definition). It carries its own [`Visibility`] the same way a
//! definition does; that visibility gates the *grouping*, while each member
//! still enforces its own access when viewed or played. This module mirrors
//! the game-definition endpoints:
//!
//!   * **Access control is enforced here, not in storage.** Storage holds the
//!     access facts (a collection's `visibility`, its grantee list) but performs
//!     no checks; the server composes the `owner ∨ curated ∨ public ∨ granted`
//!     decision from the primitives (`get_game_collection`,
//!     `get_game_collection_grantees`).
//!   * **Setting `Curated` requires an admin;** every other visibility is
//!     owner-only (the mutations are already owner-scoped by storage).
//!   * **Membership is order-only.** Items carry just a `definition_id` +
//!     position; a game's name/description/image is intrinsic to its definition
//!     and shared across every collection it appears in.
//!
//! `GET /{id}` is the collection **detail**: it returns the collection metadata
//! plus its members **hydrated and filtered to what the viewer can access** —
//! dangling refs (to since-deleted definitions) and members the viewer cannot
//! see are dropped, so a public collection never leaks a private member.

use actix_multipart::form::MultipartForm;
use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{
        ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound,
        ErrorUnauthorized,
    },
    http::header::{CacheControl, CacheDirective, ETag, EntityTag},
};
use chrono::{DateTime, Utc};
use data_model::{GameCollection, GameDefinition, GranteeSummary, User, Visibility};
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::v1::endpoints::avatar::canonicalise_to_png;
use super::game_definitions::{ImageUpdatedResponse, ImageUploadForm, SetGameSharesRequest};
use crate::api::v1::endpoints::listing::{effective_limit, page_owned, parse_scope, ListScope};

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

/// Request body for creating or updating a collection's own metadata. Membership
/// is managed separately via the item endpoints, so it is not part of this body.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCollectionRequest {
    /// Display name (unique per owner, non-empty).
    pub name: String,
    /// Optional collection-level description.
    #[serde(default)]
    pub description: Option<String>,
    /// Access tier. Defaults to `private`. Setting `curated` requires an admin.
    #[serde(default)]
    #[schema(value_type = String)]
    pub visibility: Visibility,
}

/// Query parameters for the list endpoint — a page of the scoped result.
#[derive(Deserialize, Debug)]
pub struct ListGameCollectionsQuery {
    /// Page size (server default when omitted, capped at the server maximum).
    pub limit: Option<u32>,
    /// Zero-based page offset (defaults to 0).
    pub offset: Option<u32>,
    /// Result scope: `visible` (default — everything the caller may see) or
    /// `mine` (only the caller's own collections, any visibility).
    pub scope: Option<String>,
    /// Case-insensitive name substring filter (honoured with `scope=mine`).
    pub q: Option<String>,
}

/// A page of the collections the caller may see — the merge of their own, those
/// shared with them, and every public / curated collection.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCollectionListResponse {
    /// The page of visible collections, de-duplicated and ordered by name.
    pub collections: Vec<GameCollection>,
    /// The effective page size applied (the request's `limit` capped at the
    /// server maximum).
    pub limit: u32,
    /// The zero-based offset this page started at.
    pub offset: u32,
    /// Whether at least one further collection exists beyond this page.
    pub has_more: bool,
}

/// The collection **detail**: the collection's own metadata plus its member
/// definitions, hydrated, in order, and filtered to what the viewer may access.
#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCollectionDetailResponse {
    #[schema(value_type = String)]
    /// Unique identifier.
    pub id: Uuid,
    #[schema(value_type = String)]
    /// The user that owns the collection.
    pub owner_id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional collection-level description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[schema(value_type = String)]
    /// Access tier gating the grouping.
    pub visibility: Visibility,
    /// Cache-key for the optional collection-level image; `None` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_updated_at: Option<DateTime<Utc>>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
    /// The accessible member definitions, in collection order. Members the viewer
    /// cannot access, and dangling refs to since-deleted definitions, are omitted.
    pub definitions: Vec<GameDefinition>,
}

impl GameCollectionDetailResponse {
    /// Builds a detail response from a collection's own metadata and its already
    /// access-filtered, ordered member definitions.
    fn from_parts(collection: GameCollection, definitions: Vec<GameDefinition>) -> Self {
        Self {
            id: collection.id,
            owner_id: collection.owner_id,
            name: collection.name,
            description: collection.description,
            visibility: collection.visibility,
            image_updated_at: collection.image_updated_at,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
            definitions,
        }
    }
}

/// Request body for setting a collection's whole membership in one operation.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetGameCollectionItemsRequest {
    /// The member definition ids, in the desired order. Replaces the current
    /// membership wholesale (duplicates collapse; anyone absent is dropped, any
    /// new id added). Only references are stored — a ref to an inaccessible or
    /// since-deleted definition is filtered at detail time.
    #[schema(value_type = Vec<String>)]
    pub definition_ids: Vec<Uuid>,
}

/// The current grantee list for a collection, returned by the share endpoints —
/// each grantee resolved to `{id, username}` for the owner's manage-shares view.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCollectionSharesResponse {
    /// The users currently granted access (id + username).
    pub grantees: Vec<GranteeSummary>,
}

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

fn get_authorized_user(req: &HttpRequest, admin_required: bool) -> Result<User, Error> {
    match req.extensions().get::<User>() {
        Some(user) if !admin_required || user.is_admin => Ok(user.clone()),
        _ => Err(ErrorUnauthorized("Unauthorized request")),
    }
}

/// Whether a viewer may access an entity with the given owner + visibility +
/// grantee list — the `owner ∨ curated ∨ public ∨ granted` rule, shared by the
/// collection-level check and the per-member definition check in the detail.
fn can_access(owner_id: Uuid, visibility: Visibility, viewer: Uuid, grantees: &[Uuid]) -> bool {
    owner_id == viewer
        || matches!(visibility, Visibility::Curated | Visibility::Public)
        || (visibility == Visibility::Shared && grantees.contains(&viewer))
}

/// Maps a create/update store error to its HTTP status, falling back to a
/// logged 500 for anything unexpected.
fn map_write_error(err: StoreError) -> Error {
    match err {
        StoreError::GameCollectionIdNotFound(id) => {
            ErrorNotFound(format!("Game collection '{id}' not found"))
        }
        StoreError::GameCollectionNameMissing() => {
            ErrorBadRequest("Game collection name must not be empty")
        }
        StoreError::GameCollectionNameAlreadyExists(name) => {
            ErrorConflict(format!("A game collection named '{name}' already exists"))
        }
        StoreError::GameCollectionCountLimitReached { count, max } => ErrorConflict(format!(
            "Game collection limit reached: you already own {count} collections (max {max})"
        )),
        other => {
            log::warn!("game collection store error: {other}");
            ErrorInternalServerError("Failed to store game collection")
        }
    }
}

/// Re-loads a collection the caller owns and returns it, or a 404 if it is
/// absent or owned by someone else. Used by the membership + share endpoints so
/// a non-owner cannot mutate or probe another user's collection.
async fn owned_collection(
    store_lock: &dyn storage::Store,
    user: &User,
    id: Uuid,
) -> Result<GameCollection, Error> {
    match store_lock.get_game_collection(id).await {
        Ok(collection) if collection.owner_id == user.id => Ok(collection),
        Ok(_) | Err(StoreError::GameCollectionIdNotFound(_)) => {
            Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game collection store error: {err}");
            Err(ErrorInternalServerError("Failed to load game collection"))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/game-collections
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Create a game collection",
    description = "Creates a collection owned by the caller. The server sets the id/owner/timestamps; \
                   the body carries name, description, and visibility. Membership is managed \
                   separately via the item endpoints, so a new collection starts empty. Setting \
                   visibility to 'curated' requires an admin.",
    post,
    path = "/api/v1/game-collections",
    request_body = GameCollectionRequest,
    responses(
        (status = 201, description = "Collection created", body = GameCollection),
        (status = 400, description = "Invalid request (empty name)"),
        (status = 401, description = "Unauthorized request"),
        (status = 403, description = "Only an admin may set 'curated'"),
        (status = 409, description = "A collection with that name already exists")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-collections")]
pub async fn create_game_collection(
    body: web::Json<GameCollectionRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let body = body.into_inner();

    if body.visibility == Visibility::Curated && !user.is_admin {
        return Err(ErrorForbidden("Only an admin may set 'curated' visibility"));
    }

    let now = Utc::now();
    let mut collection = GameCollection {
        id: Uuid::nil(),
        owner_id: Uuid::nil(),
        name: body.name,
        visibility: body.visibility,
        description: body.description,
        image_updated_at: None,
        items: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    let mut store_lock = store.write().await;
    match store_lock.create_game_collection(&user, &mut collection).await {
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Location", format!("/api/v1/game-collections/{}", collection.id)))
            .json(collection)),
        Err(err) => Err(map_write_error(err)),
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/game-collections  (list)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "List game collections",
    description = "Returns a page of game collections ordered by name. With scope=visible (the \
                   default) it is the caller's visible set — their own (all visibilities), those \
                   shared with them, and all public and curated collections, de-duplicated. With \
                   scope=mine it is only the caller's own collections (any visibility), optionally \
                   filtered by a case-insensitive name substring q. Paged via limit (server-capped) \
                   and offset.",
    get,
    path = "/api/v1/game-collections",
    params(
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)"),
        ("scope" = Option<String>, Query, description = "Result scope: 'visible' (default) or 'mine' (the caller's own collections)"),
        ("q" = Option<String>, Query, description = "Case-insensitive name substring filter (honoured with scope=mine)")
    ),
    responses(
        (status = 200, description = "A page of visible collections", body = GameCollectionListResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-collections")]
pub async fn list_game_collections(
    query: web::Query<ListGameCollectionsQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let q = query.into_inner();
    let scope = parse_scope(q.scope.as_deref())?;
    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);
    let store_lock = store.read().await;

    let (collections, has_more) = match scope {
        ListScope::Visible => {
            // Storage composes + pages the "visible to me" set; over-fetch one row
            // for `has_more`.
            let mut cols = store_lock
                .get_visible_game_collections(&user, limit + 1, offset)
                .await
                .map_err(|err| {
                    log::warn!("list game collections store error: {err}");
                    ErrorInternalServerError("Failed to list game collections")
                })?;
            let has_more = cols.len() as u32 > limit;
            cols.truncate(limit as usize);
            (cols, has_more)
        }
        ListScope::Mine => {
            // The caller's own set is capped, so page it here (with the name
            // filter) over the owner read rather than a DB-paged owner query.
            let all = store_lock
                .get_game_collections_for_owner(&user)
                .await
                .map_err(|err| {
                    log::warn!("list game collections store error: {err}");
                    ErrorInternalServerError("Failed to list game collections")
                })?;
            page_owned(all, q.q.as_deref(), |c| c.name.as_str(), limit, offset)
        }
    };

    Ok(HttpResponse::Ok().json(GameCollectionListResponse {
        collections,
        limit,
        offset,
        has_more,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/game-collections/{id}  (detail)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Fetch a collection with its accessible members",
    description = "Returns a collection the caller may access (owner, curated, public, or granted; \
                   otherwise 404), with its member definitions hydrated, in order, and filtered to \
                   what the viewer may access — a public collection never exposes a private member, \
                   and refs to since-deleted definitions are dropped.",
    get,
    path = "/api/v1/game-collections/{id}",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "The collection and its accessible members", body = GameCollectionDetailResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found or not accessible")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-collections/{id}")]
pub async fn get_game_collection(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;

    let collection = match store_lock.get_game_collection(id).await {
        Ok(collection) => collection,
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get game collection store error: {err}");
            return Err(ErrorInternalServerError("Failed to load game collection"));
        }
    };

    let col_grantees = store_lock.get_game_collection_grantees(id).await.map_err(|err| {
        log::warn!("get collection grantees store error: {err}");
        ErrorInternalServerError("Failed to load game collection")
    })?;

    // Collection-level access is composed here; an inaccessible collection is
    // reported as absent so its existence is not leaked.
    if !can_access(collection.owner_id, collection.visibility, user.id, &col_grantees) {
        return Err(ErrorNotFound(format!("Game collection '{id}' not found")));
    }

    // Hydrate the members in order, dropping dangling refs and any the viewer
    // cannot access — the grouping is visible but each member enforces itself.
    let mut definitions: Vec<GameDefinition> = Vec::new();
    for item in &collection.items {
        let definition = match store_lock.get_game_definition(item.definition_id).await {
            Ok(def) => def,
            Err(StoreError::GameDefinitionIdNotFound(_)) => continue,
            Err(err) => {
                log::warn!("get member definition store error: {err}");
                return Err(ErrorInternalServerError("Failed to load game collection"));
            }
        };
        let def_grantees = store_lock.get_game_definition_grantees(definition.id).await.map_err(|err| {
            log::warn!("get member grantees store error: {err}");
            ErrorInternalServerError("Failed to load game collection")
        })?;
        if can_access(definition.owner_id, definition.visibility, user.id, &def_grantees) {
            definitions.push(definition);
        }
    }

    Ok(HttpResponse::Ok().json(GameCollectionDetailResponse::from_parts(collection, definitions)))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/game-collections/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Update a collection's metadata",
    description = "Updates the name, description, and visibility of a collection owned by the \
                   caller. Membership and the collection image are left unchanged. Setting \
                   visibility to 'curated' requires an admin.",
    put,
    path = "/api/v1/game-collections/{id}",
    params(("id" = String, Path, description = "Collection id")),
    request_body = GameCollectionRequest,
    responses(
        (status = 200, description = "Collection updated", body = GameCollection),
        (status = 400, description = "Invalid request (empty name)"),
        (status = 401, description = "Unauthorized request"),
        (status = 403, description = "Only an admin may set 'curated'"),
        (status = 404, description = "Collection not found"),
        (status = 409, description = "A collection with that name already exists")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-collections/{id}")]
pub async fn update_game_collection(
    path: web::Path<Uuid>,
    body: web::Json<GameCollectionRequest>,
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
    let existing = owned_collection(&**store_lock, &user, id).await?;

    let mut collection = GameCollection {
        id: existing.id,
        owner_id: user.id,
        name: body.name,
        visibility: body.visibility,
        description: body.description,
        image_updated_at: existing.image_updated_at, // image bytes managed separately
        items: existing.items,                        // membership is managed by the item endpoints
        created_at: existing.created_at,
        updated_at: existing.updated_at,
    };

    store_lock
        .update_game_collection(&user, &mut collection)
        .await
        .map_err(map_write_error)?;

    // Re-load so the response reflects the canonical stored state across backends.
    let updated = owned_collection(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(updated))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/game-collections/{id}
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Delete a game collection",
    description = "Deletes a collection owned by the caller, removing it, its items, and its share \
                   grants. The member definitions themselves are untouched (a collection is only a \
                   grouping).",
    delete,
    path = "/api/v1/game-collections/{id}",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "Collection deleted"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-collections/{id}")]
pub async fn delete_game_collection(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut store_lock = store.write().await;

    match store_lock.delete_game_collection(&user, id).await {
        Ok(()) => Ok(HttpResponse::Ok().body(format!("game collection '{id}' deleted"))),
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("delete game collection store error: {err}");
            Err(ErrorInternalServerError("Failed to delete game collection"))
        }
    }
}

// ---------------------------------------------------------------------------
// Membership — PUT /items (reconcile the whole ordered list in one operation)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Set a collection's games",
    description = "Replaces a collection's whole membership with the supplied ordered list in one \
                   operation — duplicates collapse, anyone absent is dropped, any new id added, and \
                   the sequence reordered to match — and returns the updated collection. Owner-only \
                   (a collection owned by someone else returns 404). Only references are stored; a \
                   ref to an inaccessible or since-deleted definition is filtered at detail time.",
    put,
    path = "/api/v1/game-collections/{id}/items",
    params(("id" = String, Path, description = "Collection id")),
    request_body = SetGameCollectionItemsRequest,
    responses(
        (status = 200, description = "Membership updated", body = GameCollection),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-collections/{id}/items")]
pub async fn set_game_collection_items(
    path: web::Path<Uuid>,
    body: web::Json<SetGameCollectionItemsRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let definition_ids = body.into_inner().definition_ids;
    let mut store_lock = store.write().await;

    match store_lock.set_game_collection_items(&user, id, &definition_ids).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("set collection items store error: {err}");
            return Err(ErrorInternalServerError("Failed to update collection games"));
        }
    }

    let updated = owned_collection(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(updated))
}

// ---------------------------------------------------------------------------
// Share management — GET / PUT / DELETE /game-collections/{id}/shares
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "List a collection's grantees",
    description = "Returns the users (id + username) granted access to a collection owned by the \
                   caller. A collection owned by someone else returns 404.",
    get,
    path = "/api/v1/game-collections/{id}/shares",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "The current grantees", body = GameCollectionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-collections/{id}/shares")]
pub async fn list_game_collection_shares(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;
    owned_collection(&**store_lock, &user, id).await?;
    let grantees = store_lock.get_game_collection_grantee_summaries(id).await.map_err(|err| {
        log::warn!("get collection grantee summaries store error: {err}");
        ErrorInternalServerError("Failed to load collection shares")
    })?;
    Ok(HttpResponse::Ok().json(GameCollectionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Set a collection's share list",
    description = "Replaces the collection's grantee list with the supplied set in one operation — \
                   anyone not listed is revoked, any new id granted — and returns the updated list. \
                   Owner-only (a collection owned by someone else returns 404). The owner's own id \
                   is ignored.",
    put,
    path = "/api/v1/game-collections/{id}/shares",
    params(("id" = String, Path, description = "Collection id")),
    request_body = SetGameSharesRequest,
    responses(
        (status = 200, description = "Share list updated", body = GameCollectionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-collections/{id}/shares")]
pub async fn set_game_collection_shares(
    path: web::Path<Uuid>,
    body: web::Json<SetGameSharesRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let user_ids = body.into_inner().user_ids;

    let mut store_lock = store.write().await;
    match store_lock.set_game_collection_grantees(&user, id, &user_ids).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("set collection shares store error: {err}");
            return Err(ErrorInternalServerError("Failed to update shares"));
        }
    }

    let grantees = store_lock.get_game_collection_grantee_summaries(id).await.map_err(|err| {
        log::warn!("get collection grantee summaries store error: {err}");
        ErrorInternalServerError("Failed to load collection shares")
    })?;
    Ok(HttpResponse::Ok().json(GameCollectionSharesResponse { grantees }))
}

// ---------------------------------------------------------------------------
// Image — POST / DELETE / GET /api/v1/game-collections/{id}/image
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Upload or replace a collection's image",
    description = "Accepts a multipart/form-data upload with a single `file` part (PNG or JPEG, up \
                   to 2 MiB), canonicalised to a 256x256 PNG. Owner-only. Returns the new \
                   image_updated_at.",
    post,
    path = "/api/v1/game-collections/{id}/image",
    params(("id" = String, Path, description = "Collection id")),
    request_body(content_type = "multipart/form-data", description = "A single `file` part: PNG or JPEG, <= 2 MiB"),
    responses(
        (status = 200, description = "Image stored", body = ImageUpdatedResponse),
        (status = 400, description = "Missing/invalid image or over the size limit"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-collections/{id}/image")]
pub async fn upload_game_collection_image(
    path: web::Path<Uuid>,
    MultipartForm(form): MultipartForm<ImageUploadForm>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let png = canonicalise_to_png(&form.file.data)?;

    let mut store_lock = store.write().await;
    match store_lock.set_game_collection_image(&user, id, png).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("set collection image store error: {err}");
            return Err(ErrorInternalServerError("Failed to store image"));
        }
    }
    let image_updated_at = store_lock
        .get_game_collection(id)
        .await
        .map_err(|err| {
            log::warn!("get collection after image set: {err}");
            ErrorInternalServerError("Failed to store image")
        })?
        .image_updated_at
        .ok_or_else(|| ErrorInternalServerError("image marker missing after store"))?;
    Ok(HttpResponse::Ok().json(ImageUpdatedResponse { image_updated_at }))
}

#[utoipa::path(
    summary = "Remove a collection's image",
    description = "Removes the image of a collection owned by the caller and clears its \
                   image_updated_at. Idempotent.",
    delete,
    path = "/api/v1/game-collections/{id}/image",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 204, description = "Image removed (or none was set)"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-collections/{id}/image")]
pub async fn delete_game_collection_image(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut store_lock = store.write().await;

    // Owner-check for a clean 404 (the storage clear no-ops for a non-owner).
    owned_collection(&**store_lock, &user, id).await?;
    store_lock.clear_game_collection_image(&user, id).await.map_err(|err| {
        log::warn!("clear collection image store error: {err}");
        ErrorInternalServerError("Failed to delete image")
    })?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    summary = "Serve a collection's image",
    description = "Returns the collection's image as image/png, or 404 when it has none. \
                   Access-checked (owner, curated, public, or granted); an inaccessible collection \
                   is reported as 404. Clients cache-bust with ?v=<imageUpdatedAt>.",
    get,
    path = "/api/v1/game-collections/{id}/image",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "The image", content_type = "image/png"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not accessible or has no image")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-collections/{id}/image")]
pub async fn serve_game_collection_image(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;

    let collection = match store_lock.get_game_collection(id).await {
        Ok(collection) => collection,
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("get collection for image serve: {err}");
            return Err(ErrorInternalServerError("Failed to load image"));
        }
    };
    let col_grantees = store_lock.get_game_collection_grantees(id).await.map_err(|err| {
        log::warn!("get collection grantees store error: {err}");
        ErrorInternalServerError("Failed to load image")
    })?;
    if !can_access(collection.owner_id, collection.visibility, user.id, &col_grantees) {
        return Err(ErrorNotFound(format!("Game collection '{id}' not found")));
    }

    let Some(data) = store_lock.get_game_collection_image(id).await.map_err(|err| {
        log::warn!("get collection image store error: {err}");
        ErrorInternalServerError("Failed to load image")
    })?
    else {
        return Err(ErrorNotFound(format!("Game collection '{id}' has no image")));
    };

    let mut builder = HttpResponse::Ok();
    builder.content_type("image/png");
    builder.insert_header(CacheControl(vec![CacheDirective::Private, CacheDirective::NoCache]));
    if let Some(ts) = collection.image_updated_at {
        builder.insert_header(ETag(EntityTag::new_strong(ts.timestamp_millis().to_string())));
    }
    Ok(builder.body(data))
}
