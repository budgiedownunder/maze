import { useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import * as api from '../api/client'
import { useTheme } from '../context/ThemeContext'
import { PasswordInput } from '../components/PasswordInput'
import { validateSetPasswordForm } from '../utils/passwordValidation'
import appIcon from '../assets/app.png'

export function ResetPasswordPage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()

  const submitDisabled = !newPassword || !confirmPassword || isSubmitting

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const localError = validateSetPasswordForm({ newPassword, confirmPassword })
    if (localError) { setError(localError); return }
    if (!token) { setError('This reset link is invalid.'); return }

    setError(null)
    setIsSubmitting(true)
    try {
      // The token rides in the JSON body — no URL-encoding required.
      // useSearchParams already decoded any percent-escapes from the link.
      await api.confirmPasswordReset(token, newPassword)
      navigate('/login?message=Password+reset+successful.+Sign+in+with+your+new+password.', { replace: true })
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 400
        ? 'This reset link is invalid or has expired. Request a new one.'
        : 'Could not reset your password. Please try again.')
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
      <h1 className="auth-title auth-title--solo">Reset Password</h1>

      {!token ? (
        <div className="auth-form">
          <p role="alert" className="error-msg">This reset link is invalid.</p>
          <button type="button" onClick={() => navigate('/login')} className="btn-link">
            Back to sign in
          </button>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="auth-form">
          <p className="auth-subtitle">Choose a new password for your account.</p>

          <label htmlFor="rp-new">New password</label>
          <PasswordInput id="rp-new" value={newPassword} onChange={setNewPassword} disabled={isSubmitting} />

          <label htmlFor="rp-confirm">Confirm new password</label>
          <PasswordInput id="rp-confirm" value={confirmPassword} onChange={setConfirmPassword} disabled={isSubmitting} />

          {error && <p role="alert" className="error-msg">{error}</p>}

          <button type="submit" disabled={submitDisabled} className="btn-submit">
            Set New Password
          </button>
          <button type="button" onClick={() => navigate('/login')} disabled={isSubmitting} className="btn-link">
            Back to sign in
          </button>
        </form>
      )}
    </div>
  )
}
