//! Avatar endpoints under `/api/v1/users`.
//!
//! | Method | Path | Auth | Purpose |
//! |:-------|:-----|:-----|:--------|
//! | `POST`   | `/api/v1/users/me/avatar`   | Bearer / API key | Upload or replace the caller's avatar |
//! | `DELETE` | `/api/v1/users/me/avatar`   | Bearer / API key | Remove the caller's avatar |
//! | `GET`    | `/api/v1/users/{id}/avatar` | Bearer / API key | Serve any user's avatar as `image/png`, or `404` |
//!
//! The server **canonicalises every upload** before storing: it decodes the
//! uploaded PNG/JPEG (validating by decoding, not by trusting the client's
//! content-type), centre-crops it to a square, resizes to 256×256, and
//! re-encodes PNG. A stored avatar is therefore always a PNG, so `GET` always
//! serves `image/png` and the storage layer treats the bytes as opaque.
//!
//! All three routes require authentication. `GET` is readable for **any** user
//! id (not just the caller) so a signed-in viewer sees other players' avatars
//! on leaderboards and in headers; it only ever exposes the image bytes, never
//! profile data. Because a guarded route can't be hit by a bare `<img src>`
//! (the browser won't attach the bearer token), clients load the image via an
//! authenticated request and an object URL.

use actix_multipart::form::{bytes::Bytes as MultipartBytes, MultipartForm};
use actix_web::{
    delete,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
    get,
    http::header::{CacheControl, CacheDirective, ETag, EntityTag},
    post, web, Error, HttpMessage, HttpRequest, HttpResponse,
};
use chrono::{DateTime, Utc};
use data_model::User;
use image::{imageops::FilterType, ImageError, ImageFormat, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use std::io::Cursor;
use utoipa::ToSchema;
use uuid::Uuid;

/// Canonical avatar edge length, in pixels (square).
const AVATAR_SIZE: u32 = 256;

/// Largest edge, in pixels, accepted from an upload before it is canonicalised.
///
/// The multipart limit bounds *bytes*, not pixels, and a smooth image
/// compresses hard — so without this a file well inside the size limit can
/// still declare tens of megapixels and buy a long decode and a large buffer.
/// The decoders check this against the header, so an oversize image costs a
/// header parse rather than an allocation. The byte limit is the binding
/// constraint for real photographs (a genuine image at this many pixels does
/// not fit inside it), so this mainly closes the gap the byte limit leaves.
const MAX_UPLOAD_EDGE: u32 = 4096;

/// Multipart upload form. The client sends a single file part named `file`;
/// the `#[multipart(limit)]` attribute rejects an oversize part during
/// extraction (before the handler runs), so a too-large body never gets fully
/// buffered.
#[derive(MultipartForm)]
pub struct AvatarUploadForm {
    #[multipart(limit = "2 MiB")]
    pub file: MultipartBytes,
}

/// `200` response body for a successful upload. Carries only the new
/// `avatar_updated_at` — the one field the upload changed and the client needs
/// to refresh the avatar (it builds `/api/v1/users/{id}/avatar?v=<ts>`); the
/// rest of the profile is unchanged, so it is not echoed back.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AvatarUpdatedResponse {
    /// The freshly-stamped avatar cache-buster (RFC 3339).
    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    pub avatar_updated_at: DateTime<Utc>,
}

fn get_authorized_user(req: &HttpRequest) -> Result<User, Error> {
    req.extensions()
        .get::<User>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))
}

/// `/me` avatar mutations operate on the authenticated user, who always exists,
/// so any store error here is a server-side fault rather than a client one.
fn map_store_err(err: StoreError) -> Error {
    log::warn!("avatar store error: {err}");
    ErrorInternalServerError("avatar operation failed")
}

/// Decodes an uploaded image and produces the canonical 256×256 PNG. Decoding
/// is the real validation step — only the PNG and JPEG decoders are compiled
/// in, so any other (or corrupt) input fails here with a `400`, regardless of
/// the client-supplied content-type. An image declaring an edge longer than
/// [`MAX_UPLOAD_EDGE`] is rejected against the header, before its pixels are
/// allocated.
pub(crate) fn canonicalise_to_png(input: &[u8]) -> Result<Vec<u8>, Error> {
    let mut reader = ImageReader::new(Cursor::new(input))
        .with_guessed_format()
        .map_err(|e| ErrorBadRequest(format!("unsupported or invalid image: {e}")))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_UPLOAD_EDGE);
    limits.max_image_height = Some(MAX_UPLOAD_EDGE);
    reader.limits(limits);
    let img = reader.decode().map_err(|e| match e {
        ImageError::Limits(_) => ErrorBadRequest(format!(
            "image is larger than {MAX_UPLOAD_EDGE}x{MAX_UPLOAD_EDGE} pixels"
        )),
        other => ErrorBadRequest(format!("unsupported or invalid image: {other}")),
    })?;
    // Centre-crop to the largest centred square, then resize to the canonical
    // edge so non-square uploads aren't distorted.
    let (w, h) = (img.width(), img.height());
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    let square = img.crop_imm(x, y, side, side);
    let resized = square.resize_exact(AVATAR_SIZE, AVATAR_SIZE, FilterType::Lanczos3);
    let mut out = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|e| ErrorInternalServerError(format!("failed to encode avatar PNG: {e}")))?;
    Ok(out)
}

#[utoipa::path(
    summary = "Upload or replace the caller's avatar",
    description = "Accepts a multipart/form-data upload with a single `file` part (PNG or JPEG, \
                   up to 2 MiB and 4096x4096 pixels). The server decodes it (validating by \
                   decode), centre-crops to a square, resizes to 256x256, and stores it as PNG, \
                   stamping the caller's avatar_updated_at. Returns the new avatar_updated_at so \
                   the client can refresh its avatar without re-fetching the profile.",
    post,
    path = "/api/v1/users/me/avatar",
    request_body(content_type = "multipart/form-data", description = "A single `file` part: PNG or JPEG, <= 2 MiB and <= 4096x4096"),
    responses(
        (status = 200, description = "Avatar stored", body = AvatarUpdatedResponse),
        (status = 400, description = "Missing/invalid image, or upload exceeds the size or dimension limit"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/users/me/avatar")]
pub async fn upload_avatar(
    MultipartForm(form): MultipartForm<AvatarUploadForm>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let png = canonicalise_to_png(&form.file.data)?;

    let mut store_lock = store.write().await;
    store_lock
        .set_user_avatar(user.id, png)
        .await
        .map_err(map_store_err)?;
    // Read the freshly-stamped marker back to return it (set_user_avatar stamps
    // it server-side). Same write guard, so no extra round-trip to the pool.
    let updated = store_lock.get_user(user.id).await.map_err(map_store_err)?;
    drop(store_lock);

    let avatar_updated_at = updated
        .avatar_updated_at
        .ok_or_else(|| ErrorInternalServerError("avatar marker missing after store"))?;
    Ok(HttpResponse::Ok().json(AvatarUpdatedResponse { avatar_updated_at }))
}

#[utoipa::path(
    summary = "Remove the caller's avatar",
    description = "Removes the caller's avatar (if any) and clears their avatar_updated_at. \
                   Idempotent — succeeds even when no avatar is set.",
    delete,
    path = "/api/v1/users/me/avatar",
    responses(
        (status = 204, description = "Avatar removed (or none was set)"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/users/me/avatar")]
pub async fn delete_avatar(
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let mut store_lock = store.write().await;
    store_lock
        .clear_user_avatar(user.id)
        .await
        .map_err(map_store_err)?;
    drop(store_lock);
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    summary = "Serve a user's avatar",
    description = "Returns the user's avatar as image/png, or 404 when the user has none. \
                   Requires authentication, but is readable for any user id (not just the \
                   caller) so a signed-in viewer sees other players' avatars on leaderboards \
                   and in headers. Clients cache-bust with ?v=<avatar_updated_at>.",
    get,
    path = "/api/v1/users/{id}/avatar",
    params(
        ("id" = String, Path, description = "User id whose avatar to serve")
    ),
    responses(
        (status = 200, description = "The avatar image", content_type = "image/png"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "The user has no avatar")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/users/{id}/avatar")]
pub async fn get_avatar(
    path: web::Path<Uuid>,
    store: web::Data<SharedStore>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let store_lock = store.read().await;
    // Bytes are authoritative for "is there an image to serve". Resolve the
    // marker only when there is one, to key the cache validators on it.
    let bytes = store_lock.get_user_avatar(id).await.map_err(map_store_err)?;
    let Some(data) = bytes else {
        drop(store_lock);
        return Ok(HttpResponse::NotFound().finish());
    };
    let marker = store_lock
        .get_user(id)
        .await
        .ok()
        .and_then(|u| u.avatar_updated_at);
    drop(store_lock);

    let mut builder = HttpResponse::Ok();
    builder.content_type("image/png");
    // Must-revalidate + the marker as a strong ETag: a client that sent
    // ?v=<ts> can cache aggressively, while a bare URL re-validates on change.
    builder.insert_header(CacheControl(vec![
        CacheDirective::Private,
        CacheDirective::NoCache,
    ]));
    if let Some(ts) = marker {
        builder.insert_header(ETag(EntityTag::new_strong(ts.timestamp_millis().to_string())));
    }
    Ok(builder.body(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PNG chunk CRC-32 (IEEE, reflected). Written out rather than pulled from
    /// a crate so the test fixture below needs no extra dependency.
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in bytes {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            }
        }
        !crc
    }

    /// A PNG that is nothing but a signature and an IHDR declaring `width` x
    /// `height`. It carries no pixel data at all, so anything that decodes it
    /// far enough to complain about the *size* must have done so from the
    /// header — which is the property under test.
    fn png_header_declaring(width: u32, height: u32) -> Vec<u8> {
        let mut ihdr = Vec::from(b"IHDR".as_slice());
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8bpc, truecolour, no interlace

        let mut png = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(&ihdr);
        png.extend_from_slice(&crc32(&ihdr).to_be_bytes());
        png
    }

    fn encode_png(width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(width, height))
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encoding the fixture must succeed");
        out
    }

    #[test]
    fn oversize_declared_dimensions_are_rejected_from_the_header() {
        let err = canonicalise_to_png(&png_header_declaring(MAX_UPLOAD_EDGE + 1, 8))
            .expect_err("an over-cap image must not decode");

        assert_eq!(err.as_response_error().status_code(), 400);
        assert!(
            err.to_string().contains("larger than"),
            "expected a dimension-limit message, got: {err}"
        );
    }

    #[test]
    fn an_oversize_height_is_rejected_too() {
        let err = canonicalise_to_png(&png_header_declaring(8, MAX_UPLOAD_EDGE + 1))
            .expect_err("an over-cap image must not decode");

        assert_eq!(err.as_response_error().status_code(), 400);
        assert!(
            err.to_string().contains("larger than"),
            "expected a dimension-limit message, got: {err}"
        );
    }

    #[test]
    fn an_image_at_the_cap_still_canonicalises() {
        // Encoded at the cap rather than near it: the limit is inclusive, so
        // the largest accepted image is the one most likely to be rejected by
        // an off-by-one.
        let png = canonicalise_to_png(&encode_png(MAX_UPLOAD_EDGE, 8))
            .expect("an image at the cap must be accepted");

        let decoded = image::load_from_memory(&png).expect("output must be a PNG");
        assert_eq!((decoded.width(), decoded.height()), (AVATAR_SIZE, AVATAR_SIZE));
    }

    #[test]
    fn a_small_image_still_canonicalises_to_the_avatar_size() {
        let png = canonicalise_to_png(&encode_png(8, 8)).expect("a small image must be accepted");

        let decoded = image::load_from_memory(&png).expect("output must be a PNG");
        assert_eq!((decoded.width(), decoded.height()), (AVATAR_SIZE, AVATAR_SIZE));
    }

    #[test]
    fn a_non_image_is_still_a_bad_request() {
        let err = canonicalise_to_png(b"not an image at all")
            .expect_err("garbage must not decode");

        assert_eq!(err.as_response_error().status_code(), 400);
    }
}
