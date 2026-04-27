import { createContext, useContext, useEffect, useRef, useState } from 'react'
import * as api from '../api/client'
import type { UserProfile } from '../types/api'

// Renew when less than 1/6 of the token lifetime remains.
// Scales automatically if the server's login_expiry_hours is changed.
export const RENEWAL_FRACTION = 1 / 6

// How often to check whether a renewal is needed.
// Must be less than the renewal window for any realistic token lifetime.
const RENEWAL_CHECK_INTERVAL_MS = 60 * 60 * 1000 // 60 minutes

export function shouldRenewToken(issuedAt: string, expiry: string, now: Date = new Date()): boolean {
  const issuedMs = new Date(issuedAt).getTime()
  const expiryMs = new Date(expiry).getTime()
  const nowMs = now.getTime()
  if (nowMs >= expiryMs) return false
  const remaining = expiryMs - nowMs
  const lifetime = expiryMs - issuedMs
  return remaining / lifetime < RENEWAL_FRACTION
}

interface AuthState {
  // The login_token_id UUID, used as the Bearer token and to identify the session.
  token: string
  // Recorded by the client at login/renew time — the server does not return issued_at.
  issuedAt: string
  // login_token_expires_at from the server response.
  expiry: string
}

function loadAuthState(): AuthState | null {
  try {
    const raw = sessionStorage.getItem('auth')
    return raw ? (JSON.parse(raw) as AuthState) : null
  } catch {
    return null
  }
}

function saveAuthState(state: AuthState): void {
  sessionStorage.setItem('auth', JSON.stringify(state))
}

function clearAuthState(): void {
  sessionStorage.removeItem('auth')
}

interface AuthContextValue {
  isLoading: boolean
  isAuthenticated: boolean
  profile: UserProfile | null
  login: (email: string, password: string) => Promise<void>
  /** Ingest a bearer token issued through any auth flow other than password
   *  (today: OAuth callback). Performs the same post-login state transitions
   *  as `login()` — sessionStorage write, profile fetch, renewal interval —
   *  without making the original credential request itself. */
  setAuthFromTokenResponse: (token: string, expiresAt: string) => Promise<void>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isLoading, setIsLoading] = useState(true)
  const [authState, setAuthState] = useState<AuthState | null>(null)
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const authStateRef = useRef<AuthState | null>(null)
  authStateRef.current = authState

  function stopRenewalInterval() {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }

  async function tryRenew(state: AuthState): Promise<AuthState> {
    if (shouldRenewToken(state.issuedAt, state.expiry)) {
      const renewed = await api.renewToken(state.token)
      return {
        token: renewed.login_token_id,
        issuedAt: new Date().toISOString(),
        expiry: renewed.login_token_expires_at,
      }
    }
    return state
  }

  function startRenewalInterval() {
    stopRenewalInterval()
    intervalRef.current = setInterval(async () => {
      const current = authStateRef.current
      if (!current) return
      try {
        const updated = await tryRenew(current)
        if (updated !== current) {
          saveAuthState(updated)
          setAuthState(updated)
        }
      } catch {
        // renewal failed — token will expire naturally
      }
    }, RENEWAL_CHECK_INTERVAL_MS)
  }

  useEffect(() => {
    const stored = loadAuthState()
    if (!stored) {
      setIsLoading(false)
      return
    }

    ;(async () => {
      try {
        const current = await tryRenew(stored)
        if (current !== stored) saveAuthState(current)
        const me = await api.getMe(current.token)
        setAuthState(current)
        setProfile(me)
        startRenewalInterval()
      } catch {
        clearAuthState()
      } finally {
        setIsLoading(false)
      }
    })()

    return stopRenewalInterval
  }, [])

  async function setAuthFromTokenResponse(token: string, expiresAt: string) {
    const state: AuthState = {
      token,
      issuedAt: new Date().toISOString(),
      expiry: expiresAt,
    }
    const me = await api.getMe(state.token)
    saveAuthState(state)
    setAuthState(state)
    setProfile(me)
    startRenewalInterval()
  }

  async function login(email: string, password: string) {
    const response = await api.login(email, password)
    await setAuthFromTokenResponse(response.login_token_id, response.login_token_expires_at)
  }

  async function logout() {
    const state = authState
    clearAuthState()
    setAuthState(null)
    setProfile(null)
    stopRenewalInterval()
    if (state) {
      try {
        await api.logout(state.token)
      } catch {
        // best-effort
      }
    }
  }

  return (
    <AuthContext.Provider
      value={{
        isLoading,
        isAuthenticated: authState !== null,
        profile,
        login,
        setAuthFromTokenResponse,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}

export function useToken(): string | null {
  const stored = loadAuthState()
  return stored?.token ?? null
}
