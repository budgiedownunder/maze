//! Featured-catalogue endpoints under `/api/v1/featured-game-items`.
//!
//! The featured catalogue is the admin-ordered list that drives the Play-3D
//! **Featured** section — one sequence mixing game definitions and collections.
//! It is a faithful projection of the `Curated` visibility tier maintained by the
//! storage layer: an entity becoming `Curated` (via the definition/collection
//! update handlers) appends a row, and un-curating or deleting it removes the row
//! and recompacts the order. **Featuring is therefore not done here** — this
//! module only *reads* the ordered catalogue and *reorders* it.
//!
//!   * `GET /featured-game-items` — any signed-in user; a page of the ordered
//!     catalogue (definitions + collections hydrated).
//!   * `PUT /featured-game-items/order` — admin only; rewrites the order in one
//!     operation. Order-only: an entry whose entity is not `Curated` is rejected
//!     (membership stays owned by the tier, not this endpoint).
//!
//! The path is deliberately distinct from the app-flags `/features` endpoint.

use actix_web::{
    get, put, web, Error, HttpMessage, HttpRequest, HttpResponse,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
};
use data_model::{FeaturedGameItem, FeaturedGameItemKind, GameCollection, GameDefinition, User};
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::v1::endpoints::listing::effective_limit;

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

/// One entry of the featured catalogue, tagged by `kind`. Exactly one of
/// `definition` / `collection` is present, matching `kind`.
#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedGameItemResponse {
    /// Which kind of entity this row is — `definition` or `collection`.
    #[schema(value_type = String)]
    pub kind: FeaturedGameItemKind,
    /// The owner's username, resolved server-side so the admin view can show who
    /// owns each featured item without a per-row lookup. `"unknown"` if the owner
    /// can't be resolved (e.g. a since-deleted account).
    pub owner_username: String,
    /// The featured game definition (present when `kind == definition`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<GameDefinition>,
    /// The featured game collection (present when `kind == collection`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<GameCollection>,
}

impl FeaturedGameItemResponse {
    fn from_item(item: FeaturedGameItem, owner_username: String) -> Self {
        match item {
            FeaturedGameItem::Definition(definition) => Self {
                kind: FeaturedGameItemKind::Definition,
                owner_username,
                definition: Some(definition),
                collection: None,
            },
            FeaturedGameItem::Collection(collection) => Self {
                kind: FeaturedGameItemKind::Collection,
                owner_username,
                definition: None,
                collection: Some(collection),
            },
        }
    }
}

/// The owner id behind a featured item (definition or collection).
fn featured_owner_id(item: &FeaturedGameItem) -> Uuid {
    match item {
        FeaturedGameItem::Definition(d) => d.owner_id,
        FeaturedGameItem::Collection(c) => c.meta.owner_id,
    }
}

/// Builds the response list, resolving each item's owner username from the store
/// (deduped by owner id, so a run of items by the same owner costs one lookup).
/// A username that can't be resolved falls back to `"unknown"` rather than
/// failing the whole list.
async fn featured_responses(
    store: &dyn storage::Store,
    items: Vec<FeaturedGameItem>,
) -> Vec<FeaturedGameItemResponse> {
    let mut usernames: std::collections::HashMap<Uuid, String> = std::collections::HashMap::new();
    for item in &items {
        // Vacant-entry (rather than contains_key + insert) so the async lookup
        // only runs for an owner not yet resolved.
        if let std::collections::hash_map::Entry::Vacant(entry) = usernames.entry(featured_owner_id(item)) {
            let owner_id = *entry.key();
            let username = store
                .get_user(owner_id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| "unknown".to_string());
            entry.insert(username);
        }
    }
    items
        .into_iter()
        .map(|item| {
            let owner_username = usernames.get(&featured_owner_id(&item)).cloned().unwrap_or_default();
            FeaturedGameItemResponse::from_item(item, owner_username)
        })
        .collect()
}

/// A page of the featured catalogue, in admin order.
#[derive(Serialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedGameItemsListResponse {
    /// The page of featured items, in `sort_order`.
    pub items: Vec<FeaturedGameItemResponse>,
    /// The effective page size applied (the request's `limit` capped at the
    /// server maximum).
    pub limit: u32,
    /// The zero-based offset this page started at.
    pub offset: u32,
    /// Whether at least one further item exists beyond this page.
    pub has_more: bool,
}

/// Query parameters for the featured-catalogue list endpoint.
#[derive(Deserialize, Debug)]
pub struct ListFeaturedGameItemsQuery {
    /// Page size (server default when omitted, capped at the server maximum).
    pub limit: Option<u32>,
    /// Zero-based page offset (defaults to 0).
    pub offset: Option<u32>,
}

/// One `(kind, id)` reference in a reorder request.
#[derive(Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeaturedGameItemEntry {
    /// The kind of the referenced entity — `definition` or `collection`.
    #[schema(value_type = String)]
    pub kind: FeaturedGameItemKind,
    /// The referenced definition / collection id.
    #[schema(value_type = String)]
    pub id: Uuid,
}

/// Request body for `PUT /featured-game-items/order`: the complete desired order
/// of the featured catalogue. Order-only — every entry must already be
/// `Curated`.
#[derive(Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReorderFeaturedGameItemsRequest {
    /// The featured items in their desired display order.
    pub entries: Vec<FeaturedGameItemEntry>,
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

// ---------------------------------------------------------------------------
// GET /api/v1/featured-game-items
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "List the featured catalogue",
    description = "Returns a page of the admin-ordered featured catalogue — the curated game \
                   definitions and collections that drive the Play-3D Featured section, hydrated and \
                   in sort order. Readable by any signed-in user. Paged via limit (server-capped) and \
                   offset.",
    get,
    path = "/api/v1/featured-game-items",
    params(
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)")
    ),
    responses(
        (status = 200, description = "A page of the featured catalogue", body = FeaturedGameItemsListResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[get("/featured-game-items")]
pub async fn get_featured_game_items(
    query: web::Query<ListFeaturedGameItemsQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let _user = get_authorized_user(&req, false)?;
    let q = query.into_inner();
    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);

    let store_lock = store.read().await;
    // The featured set is admin-curated and bounded, so the whole ordered list is
    // read and the page sliced in memory (there is no name filter or scope here).
    let all = store_lock.list_featured_game_items().await.map_err(|err| {
        log::warn!("list featured game items store error: {err}");
        ErrorInternalServerError("Failed to list featured game items")
    })?;
    let total = all.len();
    let page: Vec<FeaturedGameItem> =
        all.into_iter().skip(offset as usize).take(limit as usize).collect();
    let has_more = offset as usize + page.len() < total;
    let items = featured_responses(&**store_lock, page).await;

    Ok(HttpResponse::Ok().json(FeaturedGameItemsListResponse {
        items,
        limit,
        offset,
        has_more,
    }))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/featured-game-items/order
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Reorder the featured catalogue",
    description = "Rewrites the order of the featured catalogue in one operation to match the \
                   supplied entries (order-only; membership stays owned by the Curated tier). \
                   Admin-only. An entry whose entity is not Curated — or is unknown — is rejected \
                   with 400. Returns the full catalogue in its new order.",
    put,
    path = "/api/v1/featured-game-items/order",
    request_body = ReorderFeaturedGameItemsRequest,
    responses(
        (status = 200, description = "Reordered; returns the catalogue in its new order", body = FeaturedGameItemsListResponse),
        (status = 400, description = "An entry is not curated or is unknown"),
        (status = 401, description = "Unauthorized request (not signed in, or not an admin)")
    ),
    security(("api_key" = []), ("login_token" = [])),
    tags = ["v1"]
)]
#[put("/featured-game-items/order")]
pub async fn set_featured_game_items_order(
    body: web::Json<ReorderFeaturedGameItemsRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let _admin = get_authorized_user(&req, true)?;
    let ordered: Vec<(FeaturedGameItemKind, Uuid)> =
        body.into_inner().entries.into_iter().map(|e| (e.kind, e.id)).collect();

    let mut store_lock = store.write().await;
    match store_lock.reorder_featured_game_items(&ordered).await {
        Ok(()) => {}
        Err(StoreError::FeaturedGameItemNotCurated { kind, id }) => {
            return Err(ErrorBadRequest(format!("Cannot feature a non-curated {kind} '{id}'")));
        }
        Err(StoreError::GameDefinitionIdNotFound(id))
        | Err(StoreError::GameCollectionIdNotFound(id)) => {
            return Err(ErrorBadRequest(format!("Unknown featured item '{id}'")));
        }
        Err(err) => {
            log::warn!("reorder featured game items store error: {err}");
            return Err(ErrorInternalServerError("Failed to reorder featured game items"));
        }
    }

    // Return the whole catalogue in its new order so the caller renders it
    // directly (it is bounded, so returning it unpaged is cheap).
    let all = store_lock.list_featured_game_items().await.map_err(|err| {
        log::warn!("list featured game items store error: {err}");
        ErrorInternalServerError("Failed to load featured game items")
    })?;
    let items = featured_responses(&**store_lock, all).await;
    let limit = items.len() as u32;
    Ok(HttpResponse::Ok().json(FeaturedGameItemsListResponse {
        items,
        limit,
        offset: 0,
        has_more: false,
    }))
}
