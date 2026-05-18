import { useEffect, useRef, useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import * as api from '../api/client'
import { useAuth } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import appIcon from '../assets/app.png'

type VerifyState = 'pending' | 'success' | 'invalid'

export function VerifyEmailPage() {
  const [searchParams] = useSearchParams()
  const token = searchParams.get('token')
  const [state, setState] = useState<VerifyState>(token ? 'pending' : 'invalid')
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()
  const { isAuthenticated } = useAuth()
  const submittedRef = useRef(false)

  useEffect(() => {
    if (!token || submittedRef.current) return
    submittedRef.current = true
    api.confirmEmailVerification(token)
      .then(() => setState('success'))
      .catch(() => setState('invalid'))
  }, [token])

  const continueDestination = isAuthenticated ? '/' : '/login'
  const continueLabel = isAuthenticated ? 'Continue to your account' : 'Sign in'

  return (
    <div className="auth-page">
      <button
        className="theme-toggle auth-theme-toggle"
        onClick={toggleTheme}
        aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
      >
        {theme === 'dark' ? '☀' : '☾'}
      </button>
      {state === 'pending' && <div className="spinner-overlay"><div>Loading...</div></div>}

      <img src={appIcon} alt="Maze" width={100} height={100} className="auth-logo" />
      <h1 className="auth-title auth-title--solo">Verify Email</h1>

      <div className="auth-form">
        {state === 'pending' && (
          <p role="status">Verifying your email...</p>
        )}
        {state === 'success' && (
          <>
            <p role="status" className="success-msg">
              Email verified! You can close this tab and return to the app.
            </p>
            <button type="button" onClick={() => navigate(continueDestination)} className="btn-link">
              {continueLabel}
            </button>
          </>
        )}
        {state === 'invalid' && (
          <>
            <p role="alert" className="error-msg">
              This verification link is invalid or has already been used.
            </p>
            <button type="button" onClick={() => navigate('/login')} className="btn-link">
              Back to sign in
            </button>
          </>
        )}
      </div>
    </div>
  )
}
