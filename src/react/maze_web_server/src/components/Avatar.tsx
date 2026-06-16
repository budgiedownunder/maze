import { useEffect, useState } from 'react'
import { fetchUserAvatar } from '../api/client'
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
 * Circular user avatar. When `avatarUpdatedAt` is set it fetches the image over
 * an authenticated request — the serve route is guarded, so a bare `<img src>`
 * can't carry the bearer token — and renders it from an object URL; otherwise,
 * or on any fetch error (e.g. a 404), it shows the generic placeholder. The
 * object URL is revoked on unmount and whenever the (user, marker, token)
 * changes, so blobs don't leak.
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
    let created: string | null = null
    fetchUserAvatar(token, userId, avatarUpdatedAt)
      .then(blob => {
        if (cancelled) return
        created = URL.createObjectURL(blob)
        setObjectUrl(created)
      })
      .catch(() => {
        if (!cancelled) setObjectUrl(null) // fall back to the placeholder
      })
    return () => {
      cancelled = true
      if (created) URL.revokeObjectURL(created)
    }
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
