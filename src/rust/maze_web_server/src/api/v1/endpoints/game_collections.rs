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
//!     `get_collection_grantees`).
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

use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{
        ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound,
        ErrorUnauthorized,
    },
};
use chrono::{DateTime, Utc};
use data_model::{GameCollection, GameDefinition, User, Visibility};
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use super::game_definitions::GrantShareRequest;

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

/// Query parameters for the list endpoint — a page of the merged result.
#[derive(Deserialize, Debug)]
pub struct ListGameCollectionsQuery {
    /// Page size (defaults to [`DEFAULT_PAGE_SIZE`], capped at [`MAX_PAGE_SIZE`]).
    pub limit: Option<u32>,
    /// Zero-based page offset (defaults to 0).
    pub offset: Option<u32>,
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

/// Request body for adding a game to a collection.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddCollectionItemRequest {
    /// The definition to append (idempotent — re-adding is a no-op).
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub definition_id: Uuid,
}

/// Request body for re-ordering a collection's members.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReorderCollectionItemsRequest {
    /// The member definition ids in the desired order. Ids that are not members
    /// are ignored; members omitted here keep their prior relative order after
    /// the listed ones.
    #[schema(value_type = Vec<String>)]
    pub ordered: Vec<Uuid>,
}

/// The current grantee list for a collection, returned by the share endpoints.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSharesResponse {
    /// The user ids currently granted access.
    #[schema(value_type = Vec<String>)]
    pub grantees: Vec<Uuid>,
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
    summary = "List visible game collections",
    description = "Returns a page of the collections the caller may see — the merge of their own \
                   (all visibilities), those shared with them, and all public and curated \
                   collections — de-duplicated and ordered by name. Paging is via limit \
                   (server-capped) and offset, applied after the merge.",
    get,
    path = "/api/v1/game-collections",
    params(
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)")
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
    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);
    let store_lock = store.read().await;

    let mut merged: Vec<GameCollection> = Vec::new();
    for source in [
        store_lock.get_collections_for_owner(&user).await,
        store_lock.get_collections_shared_with(user.id).await,
        store_lock.get_public_collections().await,
        store_lock.get_curated_collections().await,
    ] {
        match source {
            Ok(cols) => merged.extend(cols),
            Err(err) => {
                log::warn!("list game collections store error: {err}");
                return Err(ErrorInternalServerError("Failed to list game collections"));
            }
        }
    }

    // The same collection can arrive from several sources (e.g. an owned curated
    // one). Keep the first occurrence per id, then order by name for a stable list.
    let mut seen = std::collections::HashSet::new();
    merged.retain(|col| seen.insert(col.id));
    merged.sort_by_key(|col| col.name.to_lowercase());

    // The four scoped reads have no cross-source paging primitive, so the merge
    // is assembled in memory and the page sliced from it. Collection sets are
    // small, so this is cheap.
    let total = merged.len();
    let collections: Vec<GameCollection> =
        merged.into_iter().skip(offset as usize).take(limit as usize).collect();
    let has_more = total > offset as usize + collections.len();

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

    let col_grantees = store_lock.get_collection_grantees(id).await.map_err(|err| {
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
        let def_grantees = store_lock.get_definition_grantees(definition.id).await.map_err(|err| {
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
// Membership — POST/DELETE /items, PUT /items/reorder
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Add a game to a collection",
    description = "Appends a definition to a collection owned by the caller (idempotent — re-adding \
                   is a no-op) and returns the updated collection. Only the reference is stored; a \
                   ref to an inaccessible or since-deleted definition is filtered at detail time.",
    post,
    path = "/api/v1/game-collections/{id}/items",
    params(("id" = String, Path, description = "Collection id")),
    request_body = AddCollectionItemRequest,
    responses(
        (status = 200, description = "Item added", body = GameCollection),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[post("/game-collections/{id}/items")]
pub async fn add_collection_item(
    path: web::Path<Uuid>,
    body: web::Json<AddCollectionItemRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let definition_id = body.into_inner().definition_id;
    let mut store_lock = store.write().await;

    match store_lock.add_collection_item(&user, id, definition_id).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("add collection item store error: {err}");
            return Err(ErrorInternalServerError("Failed to add collection item"));
        }
    }

    let updated = owned_collection(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(updated))
}

#[utoipa::path(
    summary = "Remove a game from a collection",
    description = "Removes a definition from a collection owned by the caller (idempotent) and \
                   returns the updated collection.",
    delete,
    path = "/api/v1/game-collections/{id}/items/{definition_id}",
    params(
        ("id" = String, Path, description = "Collection id"),
        ("definition_id" = String, Path, description = "The member definition id to remove")
    ),
    responses(
        (status = 200, description = "Item removed", body = GameCollection),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-collections/{id}/items/{definition_id}")]
pub async fn remove_collection_item(
    path: web::Path<(Uuid, Uuid)>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let (id, definition_id) = path.into_inner();
    let mut store_lock = store.write().await;

    match store_lock.remove_collection_item(&user, id, definition_id).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("remove collection item store error: {err}");
            return Err(ErrorInternalServerError("Failed to remove collection item"));
        }
    }

    let updated = owned_collection(&**store_lock, &user, id).await?;
    Ok(HttpResponse::Ok().json(updated))
}

#[utoipa::path(
    summary = "Reorder a collection's members",
    description = "Rewrites the member order of a collection owned by the caller to the given \
                   sequence (ids not members are ignored; omitted members keep their prior relative \
                   order after the listed ones) and returns the updated collection.",
    put,
    path = "/api/v1/game-collections/{id}/items/reorder",
    params(("id" = String, Path, description = "Collection id")),
    request_body = ReorderCollectionItemsRequest,
    responses(
        (status = 200, description = "Members reordered", body = GameCollection),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-collections/{id}/items/reorder")]
pub async fn reorder_collection_items(
    path: web::Path<Uuid>,
    body: web::Json<ReorderCollectionItemsRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let ordered = body.into_inner().ordered;
    let mut store_lock = store.write().await;

    match store_lock.reorder_collection_items(&user, id, &ordered).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("reorder collection items store error: {err}");
            return Err(ErrorInternalServerError("Failed to reorder collection items"));
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
    description = "Returns the user ids granted access to a collection owned by the caller. A \
                   collection owned by someone else returns 404.",
    get,
    path = "/api/v1/game-collections/{id}/shares",
    params(("id" = String, Path, description = "Collection id")),
    responses(
        (status = 200, description = "The current grantees", body = CollectionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/game-collections/{id}/shares")]
pub async fn list_collection_shares(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let store_lock = store.read().await;
    owned_collection(&**store_lock, &user, id).await?;
    let grantees = store_lock.get_collection_grantees(id).await.map_err(|err| {
        log::warn!("get collection grantees store error: {err}");
        ErrorInternalServerError("Failed to load collection shares")
    })?;
    Ok(HttpResponse::Ok().json(CollectionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Grant a user access to a collection",
    description = "Grants the given user access to a collection owned by the caller (idempotent) \
                   and returns the updated grantee list. A collection owned by someone else \
                   returns 404.",
    put,
    path = "/api/v1/game-collections/{id}/shares",
    params(("id" = String, Path, description = "Collection id")),
    request_body = GrantShareRequest,
    responses(
        (status = 200, description = "Access granted", body = CollectionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/game-collections/{id}/shares")]
pub async fn grant_collection_share(
    path: web::Path<Uuid>,
    body: web::Json<GrantShareRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let grantee = body.into_inner().user_id;
    let mut store_lock = store.write().await;

    match store_lock.grant_collection_access(&user, id, grantee).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("grant collection access store error: {err}");
            return Err(ErrorInternalServerError("Failed to grant access"));
        }
    }

    let grantees = store_lock.get_collection_grantees(id).await.map_err(|err| {
        log::warn!("get collection grantees store error: {err}");
        ErrorInternalServerError("Failed to load collection shares")
    })?;
    Ok(HttpResponse::Ok().json(CollectionSharesResponse { grantees }))
}

#[utoipa::path(
    summary = "Revoke a user's access to a collection",
    description = "Revokes the given user's access to a collection owned by the caller (idempotent) \
                   and returns the updated grantee list. A collection owned by someone else \
                   returns 404.",
    delete,
    path = "/api/v1/game-collections/{id}/shares/{grantee}",
    params(
        ("id" = String, Path, description = "Collection id"),
        ("grantee" = String, Path, description = "The user id to revoke")
    ),
    responses(
        (status = 200, description = "Access revoked", body = CollectionSharesResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Collection not found")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[delete("/game-collections/{id}/shares/{grantee}")]
pub async fn revoke_collection_share(
    path: web::Path<(Uuid, Uuid)>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    let (id, grantee) = path.into_inner();
    let mut store_lock = store.write().await;

    match store_lock.revoke_collection_access(&user, id, grantee).await {
        Ok(()) => {}
        Err(StoreError::GameCollectionIdNotFound(_)) => {
            return Err(ErrorNotFound(format!("Game collection '{id}' not found")))
        }
        Err(err) => {
            log::warn!("revoke collection access store error: {err}");
            return Err(ErrorInternalServerError("Failed to revoke access"));
        }
    }

    let grantees = store_lock.get_collection_grantees(id).await.map_err(|err| {
        log::warn!("get collection grantees store error: {err}");
        ErrorInternalServerError("Failed to load collection shares")
    })?;
    Ok(HttpResponse::Ok().json(CollectionSharesResponse { grantees }))
}
