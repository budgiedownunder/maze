//! Request/response types shared by the game-definition and game-collection
//! endpoints: the share-list request body and the image upload form/response.
//! Kept here so `game_collections.rs` doesn't import them from
//! `game_definitions.rs` — they're not definition-specific.

use actix_multipart::form::{bytes::Bytes as MultipartBytes, MultipartForm};
use chrono::{DateTime, Utc};
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
