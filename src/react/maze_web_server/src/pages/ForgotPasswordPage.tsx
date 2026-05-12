import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useTheme } from '../context/ThemeContext'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { isValidEmail } from '../utils/validation'
import appIcon from '../assets/app.png'

const SUCCESS_COPY = "If that email is registered, we've sent a password reset link. Check your inbox."
const UNAVAILABLE_COPY = 'Password reset is unavailable on this server.'

export function ForgotPasswordPage() {
  const [email, setEmail] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitted, setSubmitted] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()
  const { email_enabled } = useAppFeatures()

  const submitDisabled = !isValidEmail(email) || isSubmitting

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setIsSubmitting(true)
    try {
      await api.requestPasswordReset(email)
      // Anti-enumeration: server always returns 200, so success copy is shown
      // regardless of whether the email is registered.
      setSubmitted(true)
    } catch {
      // Only network/transport failures land here. Surface a generic retry
      // prompt — do not leak whether the email exists.
      setError('Could not send the reset link. Please try again.')
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
      {isSubmitting && <div className="spinner-overlay"><div>Loading...</div></div>}

      <img src={appIcon} alt="Maze" width={100} height={100} className="auth-logo" />
      <h1 className="auth-title auth-title--solo">Forgot Password</h1>

      {!email_enabled ? (
        <div className="auth-form">
          <p role="status" className="success-msg">{UNAVAILABLE_COPY}</p>
          <button type="button" onClick={() => navigate('/login')} className="btn-link">
            Back to sign in
          </button>
        </div>
      ) : submitted ? (
        <div className="auth-form">
          <p role="status" className="success-msg">{SUCCESS_COPY}</p>
          <button type="button" onClick={() => navigate('/login')} className="btn-link">
            Back to sign in
          </button>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="auth-form">
          <p className="auth-subtitle">Enter the email associated with your account and we'll send you a reset link.</p>

          <label htmlFor="fp-email">Email</label>
          <input
            id="fp-email"
            type="email"
            value={email}
            onChange={e => setEmail(e.target.value)}
            disabled={isSubmitting}
            autoComplete="email"
          />

          {error && <p role="alert" className="error-msg">{error}</p>}

          <button type="submit" disabled={submitDisabled} className="btn-submit">
            Send Reset Link
          </button>
          <button type="button" onClick={() => navigate('/login')} disabled={isSubmitting} className="btn-link">
            Back to sign in
          </button>
        </form>
      )}
    </div>
  )
}
