import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { PasswordInput } from '../components/PasswordInput'

export function validateSignupForm(fields: {
  username: string
  fullName: string
  email: string
  password: string
  confirmPassword: string
}): string | null {
  if (!fields.username.trim() || !fields.fullName.trim() || !fields.email.trim() || !fields.password || !fields.confirmPassword) {
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
  const [username, setUsername] = useState('')
  const [fullName, setFullName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const navigate = useNavigate()

  const allFilled = username.trim() && fullName.trim() && email.trim() && password && confirmPassword
  const submitDisabled = !allFilled || isLoading

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const err = validateSignupForm({ username, fullName, email, password, confirmPassword })
    if (err) { setError(err); return }

    setIsLoading(true)
    setError(null)
    try {
      await api.signup(username, fullName, email, password)
      navigate('/login', { replace: true })
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 409 ? 'Username or email already in use' : 'Sign up failed. Please try again.')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div style={pageStyle}>
      {isLoading && <div style={spinnerOverlayStyle}><div>Loading...</div></div>}

      <h1 style={{ marginBottom: '1.5rem' }}>Create Account</h1>

      <form onSubmit={handleSubmit} style={formStyle}>
        <label htmlFor="su-username">Username</label>
        <input id="su-username" value={username} onChange={e => setUsername(e.target.value)} disabled={isLoading} autoComplete="username" />

        <label htmlFor="su-fullname">Full Name</label>
        <input id="su-fullname" value={fullName} onChange={e => setFullName(e.target.value)} disabled={isLoading} autoComplete="name" />

        <label htmlFor="su-email">Email</label>
        <input id="su-email" type="email" value={email} onChange={e => setEmail(e.target.value)} disabled={isLoading} autoComplete="email" />

        <label htmlFor="su-password">Password</label>
        <PasswordInput id="su-password" value={password} onChange={setPassword} disabled={isLoading} />

        <label htmlFor="su-confirm">Confirm Password</label>
        <PasswordInput id="su-confirm" value={confirmPassword} onChange={setConfirmPassword} disabled={isLoading} />

        {error && <p role="alert" style={{ color: 'red', margin: 0 }}>{error}</p>}

        <button type="submit" disabled={submitDisabled} style={{ marginTop: '0.5rem' }}>
          Sign Up
        </button>
        <button type="button" onClick={() => navigate('/login')} disabled={isLoading} style={{ background: 'none', border: '1px solid #ccc' }}>
          Back
        </button>
      </form>
    </div>
  )
}

const pageStyle: React.CSSProperties = {
  display: 'flex', flexDirection: 'column', alignItems: 'center',
  justifyContent: 'center', minHeight: '100vh', padding: '2rem',
}

const formStyle: React.CSSProperties = {
  display: 'flex', flexDirection: 'column', gap: '0.5rem',
  width: '100%', maxWidth: '360px',
}

const spinnerOverlayStyle: React.CSSProperties = {
  position: 'fixed', inset: 0, background: 'rgba(255,255,255,0.7)',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  zIndex: 2000,
}
