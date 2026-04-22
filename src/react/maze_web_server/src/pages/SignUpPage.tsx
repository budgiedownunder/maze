import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useTheme } from '../context/ThemeContext'
import { PasswordInput } from '../components/PasswordInput'
import appIcon from '../assets/app.png'

export function validateSignupForm(fields: {
  email: string
  password: string
  confirmPassword: string
}): string | null {
  if (!fields.email.trim() || !fields.password || !fields.confirmPassword) {
    return 'All fields are required'
  }
  if (fields.password !== fields.confirmPassword) {
    return 'Passwords do not match'
  }
  if (fields.password.length < 8) {
    return 'Password must be at least 8 characters'
  }
  if (!/[A-Z]/.test(fields.password)) return 'Password must contain an uppercase letter'
  if (!/[a-z]/.test(fields.password)) return 'Password must contain a lowercase letter'
  if (!/[0-9]/.test(fields.password)) return 'Password must contain a digit'
  if (!/[^A-Za-z0-9]/.test(fields.password)) return 'Password must contain a special character'
  return null
}

export function SignUpPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()

  const allFilled = email.trim() && password && confirmPassword
  const submitDisabled = !allFilled || isLoading

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const err = validateSignupForm({ email, password, confirmPassword })
    if (err) { setError(err); return }

    setIsLoading(true)
    setError(null)
    try {
      await api.signup(email, password)
      navigate('/login', { replace: true })
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 409 ? 'Email already in use' : 'Sign up failed. Please try again.')
    } finally {
      setIsLoading(false)
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
      {isLoading && <div className="spinner-overlay"><div>Loading...</div></div>}

      <img src={appIcon} alt="Maze" width={100} height={100} className="auth-logo" />
      <h1 className="auth-title auth-title--solo">Create Account</h1>

      <form onSubmit={handleSubmit} className="auth-form">
        <label htmlFor="su-email">Email</label>
        <input id="su-email" type="email" value={email} onChange={e => setEmail(e.target.value)} disabled={isLoading} autoComplete="email" />

        <label htmlFor="su-password">Password</label>
        <PasswordInput id="su-password" value={password} onChange={setPassword} disabled={isLoading} />

        <label htmlFor="su-confirm">Confirm Password</label>
        <PasswordInput id="su-confirm" value={confirmPassword} onChange={setConfirmPassword} disabled={isLoading} />

        {error && <p role="alert" className="error-msg">{error}</p>}

        <button type="submit" disabled={submitDisabled} className="btn-submit">
          Sign Up
        </button>
        <button type="button" onClick={() => navigate('/login')} disabled={isLoading} className="btn-link">
          Back
        </button>
      </form>
    </div>
  )
}
