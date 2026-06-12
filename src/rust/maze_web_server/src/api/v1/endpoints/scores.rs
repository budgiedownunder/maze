//! Score-recording endpoint under `/api/v1/scores`.
//!
//! A completed 3D run is recorded here: the client submits the run's
//! measures (`score`, `elapsed_ms`) and its subject (exactly one of a stored
//! `maze_id` or a curated `challenge` string), and the server persists a
//! [`ScoreEntry`].
//!
//! Two fields are **server-owned and never trusted from the client**:
//!   * `user_id` — taken from the authenticated session, so a caller can only
//!     record runs against their own player identity, and
//!   * `recorded_at` — stamped server-side at record time.
//!
//! The endpoint does not (and cannot) verify that the submitted run was won or
//! that the measures are genuine — the score formula is internal to the game
//! engine and there is no server-side replay. Win-only submission is a client
//! contract (the host posts only on a win); the endpoint records what an
//! authenticated client submits.

use actix_web::{
    post, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
};
use chrono::{DateTime, Utc};
use data_model::User;
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, ScoreEntry, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

/// Request body for `POST /api/v1/scores`. Carries only the run's measures and
/// its subject — `user_id` and `recorded_at` are set server-side and are not
/// part of the request. Exactly one of `maze_id` / `challenge` must be set.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct RecordScoreRequest {
    /// The stored maze that was played, or `None` for a curated/shared game.
    pub maze_id: Option<String>,
    /// The curated/shared game that was played (`"<difficulty>:<seed>"`), or
    /// `None` for a stored user maze.
    pub challenge: Option<String>,
    /// Final score at completion.
    pub score: u64,
    /// Elapsed run time in milliseconds.
    pub elapsed_ms: u64,
}

/// Response body for a successful record. Mirrors the persisted
/// [`ScoreEntry`], including the server-set `id`, `user_id`, and
/// `recorded_at`. This is the server-owned OpenAPI wire type — the storage
/// `ScoreEntry` carries no `ToSchema` derive.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct ScoreResponse {
    /// Row id (server-allocated).
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// The player who recorded the run (server-set from the session).
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    pub user_id: Uuid,
    /// The stored maze played, or `None` for a curated/shared game.
    pub maze_id: Option<String>,
    /// The curated/shared game played, or `None` for a user maze.
    pub challenge: Option<String>,
    /// Final score at completion.
    pub score: u64,
    /// Elapsed run time in milliseconds.
    pub elapsed_ms: u64,
    /// When the run was recorded (server-stamped).
    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    pub recorded_at: DateTime<Utc>,
}

impl From<ScoreEntry> for ScoreResponse {
    fn from(entry: ScoreEntry) -> Self {
        Self {
            id: entry.id,
            user_id: entry.user_id,
            maze_id: entry.maze_id,
            challenge: entry.challenge,
            score: entry.score,
            elapsed_ms: entry.elapsed_ms,
            recorded_at: entry.recorded_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Local helpers (mirror the private utilities in handlers.rs to keep this
// module self-contained without touching handler-private code paths).
// ---------------------------------------------------------------------------

fn get_authorized_user(req: &HttpRequest) -> Result<User, Error> {
    req.extensions()
        .get::<User>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))
}

// ---------------------------------------------------------------------------
// POST /api/v1/scores
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Record a completed run's score",
    description = "Records a completed 3D run for the authenticated player. The request \
                   carries the run's measures (score, elapsed_ms) and its subject — exactly \
                   one of a stored maze id or a curated challenge string. The server sets \
                   the player (user_id) from the session and stamps recorded_at; neither is \
                   trusted from the client. Win-only submission is a client contract; the \
                   endpoint records what an authenticated client submits.",
    post,
    path = "/api/v1/scores",
    request_body = RecordScoreRequest,
    responses(
        (status = 201, description = "Score recorded", body = ScoreResponse),
        (status = 400, description = "Invalid request (must set exactly one of maze_id / challenge)"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/scores")]
pub async fn record_score(
    record_req: web::Json<RecordScoreRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let body = record_req.into_inner();

    // Build the entry with server-owned identity + timestamp. The subject
    // invariant (exactly one of maze_id / challenge) is enforced by the store's
    // record_score, surfaced below as a 400.
    let entry = ScoreEntry {
        id: Uuid::new_v4(),
        user_id: user.id,
        maze_id: body.maze_id,
        challenge: body.challenge,
        score: body.score,
        elapsed_ms: body.elapsed_ms,
        recorded_at: Utc::now(),
    };

    let mut store_lock = store.write().await;
    match store_lock.record_score(&entry).await {
        Ok(_) => Ok(HttpResponse::Created().json(ScoreResponse::from(entry))),
        Err(StoreError::Other(msg)) => Err(ErrorBadRequest(msg)),
        Err(err) => {
            log::warn!("record_score store error: {err}");
            Err(ErrorInternalServerError("Failed to record score"))
        }
    }
}
