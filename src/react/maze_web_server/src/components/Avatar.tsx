import { useEffect, useState } from 'react'
import { getAvatarObjectUrl } from '../utils/imageCache'
import { useToken } from '../context/AuthContext'

// Shipped generic placeholder (a person silhouette), shown whenever the user
// has no avatar. A static public asset — loaded directly as an <img src>,
// unlike the real avatar which goes through an authenticated fetch.
const PLACEHOLDER_SRC = '/images/avatar-placeholder.png'

interface AvatarProps {
  /** The user whose avatar to show. */
  userId: string
  /** The user's `avatar_updated_at` marker. When absent/null, the placeholder
   *  is shown and no request is made; doubles as the cache-buster otherwise. */
  avatarUpdatedAt?: string | null
  /** Rendered square size in pixels. */
  size?: number
  /** Accessible label (e.g. the username); empty = decorative. */
  alt?: string
  className?: string
}

/**
 * Circular user avatar. When `avatarUpdatedAt` is set it resolves the image
 * through the shared {@link getAvatarObjectUrl} cache — fetched over an
 * authenticated request (the serve route is guarded, so a bare `<img src>`
 * can't carry the bearer token) once per user across the whole app — and renders
 * it from the shared object URL; otherwise, or on any fetch error (e.g. a 404),
 * it shows the generic placeholder. The URL is owned by the cache (shared across
 * instances), so it is NOT revoked when a single Avatar unmounts.
 */
export function Avatar({ userId, avatarUpdatedAt, size = 28, alt = '', className }: AvatarProps) {
  const token = useToken()
  const [objectUrl, setObjectUrl] = useState<string | null>(null)

  useEffect(() => {
    // No avatar (or no token to authenticate the fetch) → nothing to load; the
    // render guard below falls back to the placeholder.
    if (!avatarUpdatedAt || !token) {
      return
    }
    let cancelled = false
    getAvatarObjectUrl(token, userId, avatarUpdatedAt)
      .then(url => { if (!cancelled) setObjectUrl(url) })
    return () => { cancelled = true }
  }, [userId, avatarUpdatedAt, token])

  // Only show the fetched blob when the user actually has an avatar; the
  // `avatarUpdatedAt` guard means a stale object URL from a previous user is
  // never shown after the marker clears (the effect's deps changed but its
  // async setState may not have run yet).
  const src = avatarUpdatedAt && objectUrl ? objectUrl : PLACEHOLDER_SRC
  return (
    <img
      src={src}
      alt={alt}
      width={size}
      height={size}
      className={className ? `avatar ${className}` : 'avatar'}
      style={{ width: `${size}px`, height: `${size}px` }}
    />
  )
}
