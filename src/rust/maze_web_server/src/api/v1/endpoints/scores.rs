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
    delete, get, post, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
};
use chrono::{DateTime, Utc};
use data_model::User;
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, ScoreEntry, ScoreMetric, ScoreOrdering, ScoreboardEntry, SharedStore, SortDirection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::v1::endpoints::game_definitions::can_read_challenge_board;
use crate::api::v1::endpoints::listing::effective_limit;

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
    /// The player's avatar timestamp. Omitted on the
    /// record-score response and for players without an avatar.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    pub avatar_updated_at: Option<DateTime<Utc>>,
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
            avatar_updated_at: None,
        }
    }
}

impl From<ScoreboardEntry> for ScoreResponse {
    fn from(row: ScoreboardEntry) -> Self {
        Self {
            username: row.username,
            avatar_updated_at: row.avatar_updated_at,
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
    if let Some(challenge) = entry.challenge.as_deref() {
        if !can_read_challenge_board(&**store_lock, &user, challenge).await {
            return Err(ErrorForbidden("Not authorized to record a score for this game"));
        }
    }
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
    let user = get_authorized_user(&req)?;
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
            if !can_read_challenge_board(&**store_lock, &user, challenge).await {
                return Err(ErrorForbidden("Not authorized to read this leaderboard"));
            }
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
                .map(|entry| ScoreboardEntry {
                    entry,
                    username: None,
                    avatar_updated_at: None,
                })
                .collect();
            Ok(HttpResponse::Ok().json(build_board(rows, limit, offset)))
        }
        Err(err) => {
            log::warn!("user_history store error: {err}");
            Err(ErrorInternalServerError("Failed to read run history"))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/scores/me/completed  (which of these challenges has the caller scored on)
// ---------------------------------------------------------------------------

/// The most challenge keys one `/scores/me/completed` request may ask about. A
/// campaign is a handful of games; this bounds the `IN (…)` list well above any
/// real collection while rejecting abusive payloads.
const MAX_COMPLETED_CHALLENGES: usize = 200;

/// Request body for `POST /api/v1/scores/me/completed`: the challenge board keys
/// to check (e.g. `def:<id>` for a stored game, or a daily `def:<id>:<date>`).
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedChallengesRequest {
    /// The challenge keys to check, at most `MAX_COMPLETED_CHALLENGES`.
    pub challenges: Vec<String>,
}

/// The subset of the requested challenges the caller has completed (has ≥1 score
/// on). Order is unspecified; treat it as a set.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedChallengesResponse {
    /// The requested challenges the caller has scored on.
    pub completed: Vec<String>,
}

#[utoipa::path(
    summary = "Which of these challenges the caller has completed",
    description = "Given a set of challenge board keys (e.g. a campaign's games as `def:<id>`), \
                   returns the subset the authenticated caller has recorded at least one score \
                   against — used to derive campaign progress in one request instead of paging the \
                   caller's whole history. Scoped to the caller's own scores.",
    post,
    path = "/api/v1/scores/me/completed",
    request_body = CompletedChallengesRequest,
    responses(
        (status = 200, description = "The completed subset", body = CompletedChallengesResponse),
        (status = 400, description = "Too many challenges requested"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/scores/me/completed")]
pub async fn get_my_completed_challenges(
    body: web::Json<CompletedChallengesRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let challenges = body.into_inner().challenges;
    if challenges.len() > MAX_COMPLETED_CHALLENGES {
        return Err(ErrorBadRequest(format!(
            "At most {MAX_COMPLETED_CHALLENGES} challenges may be queried at once"
        )));
    }

    let store_lock = store.read().await;
    match store_lock.completed_challenges(user.id, &challenges).await {
        Ok(completed) => Ok(HttpResponse::Ok().json(CompletedChallengesResponse { completed })),
        Err(err) => {
            log::warn!("completed_challenges store error: {err}");
            Err(ErrorInternalServerError("Failed to read completed challenges"))
        }
    }
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/scores  (reset a leaderboard)
// ---------------------------------------------------------------------------

/// Query parameters for `DELETE /api/v1/scores`. Exactly one of `maze_id` /
/// `challenge` selects the board to reset.
#[derive(Deserialize, Debug)]
pub struct ResetScoresQuery {
    pub maze_id: Option<String>,
    pub challenge: Option<String>,
}

/// Response body for a leaderboard reset — the number of score rows removed.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Clone)]
pub struct ResetScoresResponse {
    /// Number of score rows deleted (0 if the board was already empty).
    pub deleted: u64,
}

#[utoipa::path(
    summary = "Reset a leaderboard to empty",
    description = "Deletes every score for one subject (exactly one of maze_id / challenge), \
                   resetting that leaderboard to empty. Authorization depends on the subject: a \
                   curated challenge board is global and requires an admin; a stored maze board \
                   requires the requesting user to own that maze. Returns the number of rows \
                   removed (0 if the board was already empty).",
    delete,
    path = "/api/v1/scores",
    params(
        ("maze_id" = Option<String>, Query, description = "Reset this stored maze's board — owner only (mutually exclusive with challenge)"),
        ("challenge" = Option<String>, Query, description = "Reset this curated challenge's board — admin only")
    ),
    responses(
        (status = 200, description = "Leaderboard reset", body = ResetScoresResponse),
        (status = 400, description = "Invalid request (must set exactly one of maze_id / challenge)"),
        (status = 401, description = "Unauthorized request"),
        (status = 403, description = "Not allowed to reset this leaderboard")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/scores")]
pub async fn reset_leaderboard(
    query: web::Query<ResetScoresQuery>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req)?;
    let q = query.into_inner();

    let mut store_lock = store.write().await;
    let deleted = match (q.maze_id.as_deref(), q.challenge.as_deref()) {
        // A stored maze board: only the maze's owner may reset it. `get_maze`
        // enforces ownership (it errors for a maze the user doesn't own), so a
        // failure here is a 403 — without leaking whether the maze exists.
        (Some(maze_id), None) => {
            if store_lock.get_maze(&user, maze_id).await.is_err() {
                return Err(ErrorForbidden("Not allowed to reset this leaderboard"));
            }
            store_lock.clear_maze_scores(maze_id).await
        }
        // A curated challenge board is global — admin only.
        (None, Some(challenge)) => {
            if !user.is_admin {
                return Err(ErrorForbidden("Not allowed to reset this leaderboard"));
            }
            store_lock.clear_challenge_scores(challenge).await
        }
        _ => {
            return Err(ErrorBadRequest("must set exactly one of maze_id / challenge"));
        }
    };

    match deleted {
        Ok(deleted) => Ok(HttpResponse::Ok().json(ResetScoresResponse { deleted })),
        Err(err) => {
            log::warn!("reset_leaderboard store error: {err}");
            Err(ErrorInternalServerError("Failed to reset leaderboard"))
        }
    }
}
