import { fetchUserAvatar } from '../api/client'

// Session-scoped cache of object URLs for **guarded images** the app renders
// from an authenticated fetch (the serve routes require the bearer token, so a
// bare `<img src>` can't be used). Each image is fetched **once** per
// (kind, id, marker) and shared by every consumer, rather than once per
// component instance.
//
// The core is generic over the image kind; each kind gets a thin wrapper (a
// namespaced key + a fetcher) at the bottom. Only `user` (avatars) is wired
// today — game-definition / game-collection images would each add a `fetchBlob`
// + wrapper, with no change to the cache itself.

/** The kinds of guarded image the cache serves. */
export type ImageSubjectKind = 'user' | 'game-definition' | 'game-collection'

interface ImageCacheEntry {
  marker: string
  // In-flight-or-resolved fetch → object URL, or null when there is no image /
  // the fetch failed (cached too, so repeated 404s don't re-fetch).
  url: Promise<string | null>
}

const cache = new Map<string, ImageCacheEntry>()

// Namespaced cache key so, e.g., a game id and a user id that happen to share a
// UUID don't collide.
function imageKey(kind: ImageSubjectKind, id: string): string {
  return `${kind}:${id}`
}

/**
 * Resolves to a shared object URL for a subject's image — fetched at most once
 * per (kind, id, marker) and reused by every caller — or `null` when there is
 * none / the fetch fails. A changed `marker` (a replaced image) evicts and
 * revokes the stale entry first, so the old blob isn't leaked. `fetchBlob`
 * performs the kind-specific authenticated GET.
 */
export function getCachedImageUrl(
  kind: ImageSubjectKind,
  id: string,
  marker: string,
  fetchBlob: () => Promise<Blob>,
): Promise<string | null> {
  const key = imageKey(kind, id)
  const existing = cache.get(key)
  if (existing) {
    if (existing.marker === marker) return existing.url
    // The image changed — drop the stale blob once its fetch settles. (Callers
    // still showing the old URL re-render to the new one via their own effect.)
    void existing.url.then(url => { if (url) URL.revokeObjectURL(url) })
    cache.delete(key)
  }
  const url = fetchBlob()
    .then(blob => URL.createObjectURL(blob))
    .catch(() => null)
  cache.set(key, { marker, url })
  return url
}

/**
 * Clears the cache, revoking every resolved object URL. For test isolation and
 * for dropping the previous session's blobs on sign-out; it does NOT run on an
 * individual consumer's unmount, since the URLs are shared.
 */
export function resetImageCache(): void {
  for (const entry of cache.values()) {
    void entry.url.then(url => { if (url) URL.revokeObjectURL(url) })
  }
  cache.clear()
}

// ── Per-kind wrappers ───────────────────────────────────────────────────────

/** Shared object URL for a user's avatar (the only kind wired so far). */
export function getAvatarObjectUrl(token: string, userId: string, marker: string): Promise<string | null> {
  return getCachedImageUrl('user', userId, marker, () => fetchUserAvatar(token, userId, marker))
}
