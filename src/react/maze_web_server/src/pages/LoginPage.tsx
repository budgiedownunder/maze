import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { PasswordInput } from '../components/PasswordInput'
import appIcon from '../assets/app.png'

export function LoginPage() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const { login, isLoading } = useAuth()
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()
  const { allow_signup } = useAppFeatures()

  const isBusy = isLoading || isSubmitting
  const submitDisabled = !email.trim() || !password || isBusy

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setIsSubmitting(true)
    try {
      await login(email, password)
      navigate('/mazes', { replace: true })
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

        {error && <p role="alert" className="error-msg">{error}</p>}

        <button type="submit" disabled={submitDisabled} className="btn-submit">
          Sign In
        </button>
        {allow_signup && (
          <button type="button" onClick={() => navigate('/signup')} disabled={isBusy} className="btn-link">
            Sign Up
          </button>
        )}
      </form>
    </div>
  )
}
