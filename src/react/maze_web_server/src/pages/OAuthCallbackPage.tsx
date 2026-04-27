import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { parseCallbackHash } from '../utils/oauth'

/** Landing page for the server's web-origin OAuth redirect. The server has
 *  already minted a bearer token and stuffed it into the URL fragment; we
 *  ingest it via AuthContext and navigate onwards. The token is then cleared
 *  from `window.location` so it does not end up in the browser history. */
export function OAuthCallbackPage() {
  const navigate = useNavigate()
  const { setAuthFromTokenResponse } = useAuth()

  useEffect(() => {
    const parsed = parseCallbackHash(window.location.hash)
    if (!parsed) {
      navigate('/login?error=oauth_callback_missing_token', { replace: true })
      return
    }
    // Clear the fragment immediately so the token is not retained in
    // history / sharable URLs / referer headers.
    window.history.replaceState(null, '', window.location.pathname + window.location.search)

    setAuthFromTokenResponse(parsed.token, parsed.expiresAt)
      .then(() => navigate('/mazes', { replace: true, state: { showWelcome: parsed.newUser } }))
      .catch(() => navigate('/login?error=oauth_session_init_failed', { replace: true }))
  }, [navigate, setAuthFromTokenResponse])

  return (
    <div className="auth-page">
      <div className="spinner-overlay"><div>Signing you in…</div></div>
    </div>
  )
}
