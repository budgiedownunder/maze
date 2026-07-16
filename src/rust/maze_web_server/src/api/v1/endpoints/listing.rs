//! Shared list-endpoint helpers: page sizing (used by the game-definition,
//! game-collection and score list endpoints) and the game list scope + in-memory
//! owner-set paging used by the two game-list endpoints. Kept here so the
//! endpoints share one copy rather than duplicating it per module.

use actix_web::{error::ErrorBadRequest, Error};

/// Page size used when the caller omits `limit`.
pub(crate) const DEFAULT_PAGE_SIZE: u32 = 20;
/// Hard server cap on `limit` — a caller asking for more is silently capped to
/// this, and the effective value is echoed back so the client can page correctly.
pub(crate) const MAX_PAGE_SIZE: u32 = 100;

/// Resolves the effective page size: the caller's `limit` (or the default when
/// omitted), capped at [`MAX_PAGE_SIZE`].
pub(crate) fn effective_limit(requested: Option<u32>) -> u32 {
    requested.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE)
}

/// The result scope for the game-definition / game-collection list endpoints.
/// Shared by both so a caller can page one scope at a time rather than fetching
/// the whole "visible to me" merge and filtering client-side (which drops a
/// user's own items past the first page once the public tier grows).
pub(crate) enum ListScope {
    /// Everything the caller may see — their own + shared + public + curated.
    Visible,
    /// Only the caller's own items (any visibility).
    Mine,
    /// Only items shared with the caller — a `Shared` grant they don't own
    /// (public/curated excluded). Backs the play-side "Shared with me" list.
    Shared,
    /// Only cross-owner `Public` items (the caller's own excluded) — the
    /// unbounded Community pool, name-filterable via `q`.
    Public,
}

/// Parses the `scope` query value, defaulting to [`ListScope::Visible`] when
/// omitted.
pub(crate) fn parse_scope(raw: Option<&str>) -> Result<ListScope, Error> {
    match raw {
        None | Some("visible") => Ok(ListScope::Visible),
        Some("mine") => Ok(ListScope::Mine),
        Some("shared") => Ok(ListScope::Shared),
        Some("public") => Ok(ListScope::Public),
        Some(other) => Err(ErrorBadRequest(format!(
            "invalid scope '{other}' (expected 'visible', 'mine', 'shared' or 'public')"
        ))),
    }
}

/// Applies an optional case-insensitive name substring filter, then pages the
/// caller's own set in memory. The owner set is bounded by the per-user cap, so
/// paging it here (over the existing owner read) avoids a DB-paged owner query.
/// Returns the page and whether any rows remain beyond it.
pub(crate) fn page_owned<T>(
    items: Vec<T>,
    q: Option<&str>,
    name_of: impl Fn(&T) -> &str,
    limit: u32,
    offset: u32,
) -> (Vec<T>, bool) {
    let needle = q.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
    let filtered: Vec<T> = match needle {
        Some(n) => items
            .into_iter()
            .filter(|it| name_of(it).to_lowercase().contains(&n))
            .collect(),
        None => items,
    };
    let total = filtered.len();
    let page: Vec<T> = filtered
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    let has_more = offset as usize + page.len() < total;
    (page, has_more)
}
