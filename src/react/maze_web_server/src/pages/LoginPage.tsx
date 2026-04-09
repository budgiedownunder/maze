import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { PasswordInput } from '../components/PasswordInput'
import appIcon from '/app.png'

export function LoginPage() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const { login, isLoading } = useAuth()
  const navigate = useNavigate()

  const submitDisabled = !username.trim() || !password || isLoading

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    try {
      await login(username, password)
      navigate('/mazes', { replace: true })
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 401 ? 'Invalid username or password' : 'Login failed. Please try again.')
    }
  }

  return (
    <div style={pageStyle}>
      {isLoading && <div style={spinnerOverlayStyle}><div>Loading...</div></div>}

      <img src={appIcon} alt="Maze" width={100} height={100} style={{ borderRadius: '50%' }} />
      <h1 style={{ margin: '0.75rem 0 0.25rem' }}>Maze</h1>
      <p style={{ margin: '0 0 1.5rem', color: '#555' }}>Sign in to your account</p>

      <form onSubmit={handleSubmit} style={formStyle}>
        <label htmlFor="username">Username</label>
        <input
          id="username"
          value={username}
          onChange={e => setUsername(e.target.value)}
          disabled={isLoading}
          autoComplete="username"
        />

        <label htmlFor="password">Password</label>
        <PasswordInput id="password" value={password} onChange={setPassword} disabled={isLoading} />

        {error && <p role="alert" style={{ color: 'red', margin: 0 }}>{error}</p>}

        <button type="submit" disabled={submitDisabled} style={{ marginTop: '0.5rem' }}>
          Sign In
        </button>
        <button type="button" onClick={() => navigate('/signup')} disabled={isLoading} style={{ background: 'none', border: '1px solid #ccc' }}>
          Sign Up
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
