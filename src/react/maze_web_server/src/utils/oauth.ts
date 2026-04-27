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

/** Maps an OAuth error code (the value of `?error=…` on /login) to a friendly
 *  user-facing message. The codes are emitted by the Rust callback handler in
 *  `oauth_callback` and the `OAuthCallbackPage` component. Unknown codes fall
 *  back to a generic "we couldn't sign you in" message rather than echoing
 *  the raw code. */
export function getOAuthErrorMessage(code: string | null): string | null {
  if (!code) return null
  // Provider-side errors arrive as `provider_error:<provider-code>` (e.g.
  // `provider_error:access_denied` when the user declines consent at Google).
  if (code.startsWith('provider_error:')) {
    const detail = code.slice('provider_error:'.length)
    if (detail === 'access_denied') return 'Sign-in was cancelled at the provider.'
    return `The provider reported an error (${detail}). Please try again.`
  }
  switch (code) {
    case 'signup_disabled':
      return 'Sign-up is disabled on this server. Sign-in via this provider is only available to existing users.'
    case 'email_not_verified':
      return 'The provider did not return a verified email address. Please verify your email with the provider and try again.'
    case 'missing_email':
      return 'The provider did not return an email address, so we cannot complete sign-in.'
    case 'invalid_state':
    case 'missing_state':
    case 'state_mismatch':
    case 'state_expired':
    case 'provider_mismatch':
      return 'The sign-in session expired or was invalid. Please try again.'
    case 'missing_code':
    case 'provider_response':
      return 'There was a problem completing sign-in with the provider. Please try again.'
    case 'store_error':
      return 'A server error occurred while completing sign-in. Please try again later.'
    case 'oauth_callback_missing_token':
    case 'oauth_session_init_failed':
      return 'Sign-in completed but we could not establish a session. Please try again.'
    default:
      return 'We could not sign you in. Please try again.'
  }
}
