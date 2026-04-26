/** Parses `#token=…&expires_at=…` from a URL fragment. Returns null if either
 *  field is missing — the OAuth callback page treats that as a flow failure.
 *
 *  Lives in utils (not on the page component) so the page file exports only
 *  components, keeping `react-refresh/only-export-components` happy. */
export function parseCallbackHash(hash: string): { token: string; expiresAt: string } | null {
  const fragment = hash.startsWith('#') ? hash.slice(1) : hash
  const params = new URLSearchParams(fragment)
  const token = params.get('token')
  const expiresAt = params.get('expires_at')
  if (!token || !expiresAt) return null
  return { token, expiresAt }
}
