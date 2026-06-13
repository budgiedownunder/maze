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
    get, post, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
};
use chrono::{DateTime, Utc};
use data_model::User;
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, ScoreEntry, ScoreMetric, ScoreOrdering, ScoreboardEntry, SharedStore, SortDirection};
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
    /// Optional player username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
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
            username: None,
        }
    }
}

impl From<ScoreboardEntry> for ScoreResponse {
    fn from(row: ScoreboardEntry) -> Self {
        Self {
            username: row.username,
            ..ScoreResponse::from(row.entry)
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

// ---------------------------------------------------------------------------
// Paging + ordering
// ---------------------------------------------------------------------------

/// Page size used when the caller omits `limit`.
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Hard server cap on `limit` — a caller asking for more is silently capped to
/// this, and the effective value is echoed back in the response so the client
/// can page correctly.
const MAX_PAGE_SIZE: u32 = 100;

/// Resolves the effective page size: the caller's `limit` (or the default when
/// omitted), capped at [`MAX_PAGE_SIZE`].
fn effective_limit(requested: Option<u32>) -> u32 {
    requested.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE)
}

/// Parses the `metric` query value into a [`ScoreMetric`], defaulting to
/// `Time` when omitted.
fn parse_metric(raw: Option<&str>) -> Result<ScoreMetric, Error> {
    match raw {
        None | Some("time") => Ok(ScoreMetric::Time),
        Some("score") => Ok(ScoreMetric::Score),
        Some(other) => Err(ErrorBadRequest(format!(
            "invalid metric '{other}' (expected 'time' or 'score')"
        ))),
    }
}

/// The "best first" direction for a metric — fastest time first, highest score
/// first — used when the caller does not pin `direction`.
fn natural_direction(metric: ScoreMetric) -> SortDirection {
    match metric {
        ScoreMetric::Time => SortDirection::Ascending,
        ScoreMetric::Score => SortDirection::Descending,
    }
}

/// Parses the `direction` query value, defaulting to the metric's natural
/// "best first" direction when omitted.
fn parse_direction(raw: Option<&str>, metric: ScoreMetric) -> Result<SortDirection, Error> {
    match raw {
        None => Ok(natural_direction(metric)),
        Some("asc") => Ok(SortDirection::Ascending),
        Some("desc") => Ok(SortDirection::Descending),
        Some(other) => Err(ErrorBadRequest(format!(
            "invalid direction '{other}' (expected 'asc' or 'desc')"
        ))),
    }
}

/// Trims an over-fetched page (`limit + 1` rows requested) down to `limit`,
/// deriving `has_more` from whether the extra row was present — avoids a
/// separate COUNT query. Each row carries its (optionally resolved) username.
fn build_board(mut rows: Vec<ScoreboardEntry>, limit: u32, offset: u32) -> ScoreboardResponse {
    let has_more = rows.len() as u32 > limit;
    rows.truncate(limit as usize);
    ScoreboardResponse {
        scores: rows.into_iter().map(ScoreResponse::from).collect(),
        limit,
        offset,
        has_more,
    }
}

/// Query parameters for the leaderboard endpoint. Exactly one of `maze_id` /
/// `challenge` selects the board; `metric` / `direction` choose the ordering;
/// `limit` / `offset` page it.
#[derive(Deserialize, Debug)]
pub struct LeaderboardQuery {
    pub maze_id: Option<String>,
    pub challenge: Option<String>,
    pub metric: Option<String>,
    pub direction: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub include_usernames: Option<bool>,
}

/// Query parameters for the personal-history endpoint.
#[derive(Deserialize, Debug)]
pub struct HistoryQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// A page of a board (leaderboard or personal history). `limit` is the
/// *effective* (server-capped) page size, and `has_more` tells the client
/// whether a further page exists.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct ScoreboardResponse {
    /// The page of entries, already ordered by the request's metric/direction
    /// (or recency, for personal history).
    pub scores: Vec<ScoreResponse>,
    /// The effective page size applied (the request's `limit` capped at the
    /// server maximum).
    pub limit: u32,
    /// The zero-based offset this page started at.
    pub offset: u32,
    /// Whether at least one further entry exists beyond this page.
    pub has_more: bool,
}

// ---------------------------------------------------------------------------
// GET /api/v1/scores  (leaderboard)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Read a leaderboard page",
    description = "Returns a page of the leaderboard for a single subject — exactly one of a \
                   stored maze (maze_id) or a curated challenge (challenge). Ordering is chosen \
                   by metric (time | score) and direction (asc | desc); when direction is \
                   omitted it defaults to 'best first' for the metric (fastest time / highest \
                   score). Paging is via limit (server-capped) and offset.",
    get,
    path = "/api/v1/scores",
    params(
        ("maze_id" = Option<String>, Query, description = "Stored maze id to rank (mutually exclusive with challenge)"),
        ("challenge" = Option<String>, Query, description = "Curated challenge to rank (mutually exclusive with maze_id)"),
        ("metric" = Option<String>, Query, description = "Ranking metric: 'time' (default) or 'score'"),
        ("direction" = Option<String>, Query, description = "Sort direction: 'asc' or 'desc' (defaults to best-first for the metric)"),
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)"),
        ("include_usernames" = Option<bool>, Query, description = "Resolve + include each row's player username (default true; set false for personal boards)")
    ),
    responses(
        (status = 200, description = "A leaderboard page", body = ScoreboardResponse),
        (status = 400, description = "Invalid request (must set exactly one of maze_id / challenge; or bad metric/direction)"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/scores")]
pub async fn get_leaderboard(
    query: web::Query<LeaderboardQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let _user = get_authorized_user(&req)?;
    let q = query.into_inner();

    let metric = parse_metric(q.metric.as_deref())?;
    let direction = parse_direction(q.direction.as_deref(), metric)?;
    let ordering = ScoreOrdering { metric, direction };

    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);
    // Over-fetch one extra row so `build_board` can report `has_more` without a
    // COUNT query.
    let fetch = limit + 1;
    let include_usernames = q.include_usernames.unwrap_or(true);

    let store_lock = store.read().await;
    let result = match (q.maze_id.as_deref(), q.challenge.as_deref()) {
        (Some(maze_id), None) => {
            store_lock.maze_leaderboard(maze_id, ordering, fetch, offset, include_usernames).await
        }
        (None, Some(challenge)) => {
            store_lock
                .challenge_leaderboard(challenge, ordering, fetch, offset, include_usernames)
                .await
        }
        _ => {
            return Err(ErrorBadRequest(
                "must set exactly one of maze_id / challenge",
            ))
        }
    };

    match result {
        Ok(rows) => Ok(HttpResponse::Ok().json(build_board(rows, limit, offset))),
        Err(err) => {
            log::warn!("leaderboard store error: {err}");
            Err(ErrorInternalServerError("Failed to read leaderboard"))
        }
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/scores/me  (personal history)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Read the caller's run history",
    description = "Returns a page of the authenticated player's own completed runs, most \
                   recent first. Paging is via limit (server-capped) and offset.",
    get,
    path = "/api/v1/scores/me",
    params(
        ("limit" = Option<u32>, Query, description = "Page size (default 20, capped at 100)"),
        ("offset" = Option<u32>, Query, description = "Zero-based page offset (default 0)")
    ),
    responses(
        (status = 200, description = "A page of the caller's run history", body = ScoreboardResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/scores/me")]
pub async fn get_my_history(
    query: web::Query<HistoryQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let q = query.into_inner();

    let limit = effective_limit(q.limit);
    let offset = q.offset.unwrap_or(0);
    let fetch = limit + 1;

    let store_lock = store.read().await;
    match store_lock.user_history(user.id, fetch, offset).await {
        Ok(entries) => {
            // History rows are always the caller — no usernames to resolve.
            let rows = entries
                .into_iter()
                .map(|entry| ScoreboardEntry { entry, username: None })
                .collect();
            Ok(HttpResponse::Ok().json(build_board(rows, limit, offset)))
        }
        Err(err) => {
            log::warn!("user_history store error: {err}");
            Err(ErrorInternalServerError("Failed to read run history"))
        }
    }
}
