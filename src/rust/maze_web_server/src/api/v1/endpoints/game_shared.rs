//! Request/response types and small helpers shared by the game-definition and
//! game-collection endpoints: the share-list request body, the image upload
//! form/response, and the admin-override owner resolver. Kept here so
//! `game_collections.rs` doesn't import them from `game_definitions.rs` — they're
//! not definition-specific.

use actix_multipart::form::{bytes::Bytes as MultipartBytes, MultipartForm};
use actix_web::{error::ErrorInternalServerError, Error};
use chrono::{DateTime, Utc};
use data_model::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

/// Multipart upload form for a definition / collection image — a single `file`
/// part, oversize-rejected during extraction. Shared by both endpoints
/// (identical shape to the avatar upload).
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

/// Resolves the owner to pass to an owner-scoped storage `update_*` call, given
/// the `caller` and the item's real `owner_id`. When the caller owns the item
/// this is the caller themselves; when an **admin** is editing an item they
/// don't own (the admin-override on the update / set-visibility handlers), it
/// loads the item's real owner so the owner-scoped update keeps ownership with
/// the original owner rather than transferring it to the admin.
///
/// The handler must have already authorized the caller (owner ∨ admin) before
/// calling this — it does not itself gate access.
pub(crate) async fn resolve_owner(
    store: &dyn storage::Store,
    caller: &User,
    owner_id: Uuid,
) -> Result<User, Error> {
    if caller.id == owner_id {
        return Ok(caller.clone());
    }
    store.get_user(owner_id).await.map_err(|err| {
        log::warn!("resolve owner {owner_id} for admin override: {err}");
        ErrorInternalServerError("Failed to load item owner")
    })
}
