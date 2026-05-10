import { useEffect, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { PasswordInput } from '../components/PasswordInput'
import { OAuthButtons } from '../components/OAuthButtons'
import { getOAuthErrorMessage } from '../utils/oauth'
import appIcon from '../assets/app.png'

export function LoginPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [flash, setFlash] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const { login, isLoading } = useAuth()
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()
  const { allow_signup, oauth_providers } = useAppFeatures()
  const [searchParams, setSearchParams] = useSearchParams()

  // Surface OAuth-flow errors that the server (or the OAuthCallbackPage)
  // delivers via `?error=<code>` on this URL. Strip the query param after
  // reading so a refresh or a follow-up successful sign-in doesn't keep the
  // stale error visible. The same channel carries `?message=` flashes from
  // sibling pages (e.g. successful password reset).
  useEffect(() => {
    const code = searchParams.get('error')
    const message = getOAuthErrorMessage(code)
    const flashMessage = searchParams.get('message')
    if (message || flashMessage) {
      if (message) setError(message)
      if (flashMessage) setFlash(flashMessage)
      const next = new URLSearchParams(searchParams)
      next.delete('error')
      next.delete('message')
      setSearchParams(next, { replace: true })
    }
  }, [searchParams, setSearchParams])

  const isBusy = isLoading || isSubmitting
  const submitDisabled = !email.trim() || !password || isBusy

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setIsSubmitting(true)
    try {
      const result = await login(email, password)
      // Mirror the OAuthCallbackPage routing: first-ever-sign-ins land
      // directly on /account with the welcome-banner state so the user
      // sees it before anything else; returning users go to /mazes.
      navigate(
        result.isFirstSignIn ? '/account' : '/mazes',
        { replace: true, state: result.isFirstSignIn ? { welcome: true } : undefined },
      )
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 401 ? 'Invalid email or password' : 'Login failed. Please try again.')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="auth-page">
      <button
        className="theme-toggle auth-theme-toggle"
        onClick={toggleTheme}
        aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
      >
        {theme === 'dark' ? '☀' : '☾'}
      </button>
      {isBusy && <div className="spinner-overlay"><div>Loading...</div></div>}

      <img src={appIcon} alt="Maze" width={100} height={100} className="auth-logo" />
      <h1 className="auth-title">Maze</h1>
      <p className="auth-subtitle">Sign in to your account</p>

      <form onSubmit={handleSubmit} className="auth-form">
        <label htmlFor="email">Email</label>
        <input
          type="email"
          id="email"
          value={email}
          onChange={e => setEmail(e.target.value)}
          disabled={isBusy}
          autoComplete="email"
        />

        <label htmlFor="password">Password</label>
        <PasswordInput id="password" value={password} onChange={setPassword} disabled={isBusy} />

        {flash && <p role="status" className="success-msg">{flash}</p>}
        {error && <p role="alert" className="error-msg">{error}</p>}

        <button type="submit" disabled={submitDisabled} className="btn-submit">
          Sign In
        </button>
        <button type="button" onClick={() => navigate('/forgot-password')} disabled={isBusy} className="btn-link">
          Forgot password?
        </button>
        {allow_signup && (
          <button type="button" onClick={() => navigate('/signup')} disabled={isBusy} className="btn-link">
            Sign Up
          </button>
        )}

        <OAuthButtons providers={oauth_providers} disabled={isBusy} />
      </form>
    </div>
  )
}
