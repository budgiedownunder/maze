import { createContext, useContext } from 'react'
import type { UserProfile } from '../types/api'

// Renew when less than 1/6 of the token lifetime remains.
// Scales automatically if the server's login_expiry_hours is changed.
export const RENEWAL_FRACTION = 1 / 6

// How often to check whether a renewal is needed.
// Must be less than the renewal window for any realistic token lifetime.
export const RENEWAL_CHECK_INTERVAL_MS = 60 * 60 * 1000 // 60 minutes

export function shouldRenewToken(issuedAt: string, expiry: string, now: Date = new Date()): boolean {
  const issuedMs = new Date(issuedAt).getTime()
  const expiryMs = new Date(expiry).getTime()
  const nowMs = now.getTime()
  if (nowMs >= expiryMs) return false
  const remaining = expiryMs - nowMs
  const lifetime = expiryMs - issuedMs
  return remaining / lifetime < RENEWAL_FRACTION
}

export interface AuthState {
  // The login_token_id UUID, used as the Bearer token and to identify the session.
  token: string
  // Recorded by the client at login/renew time — the server does not return issued_at.
  issuedAt: string
  // login_token_expires_at from the server response.
  expiry: string
}

export function loadAuthState(): AuthState | null {
  try {
    const raw = sessionStorage.getItem('auth')
    return raw ? (JSON.parse(raw) as AuthState) : null
  } catch {
    return null
  }
}

export function saveAuthState(state: AuthState): void {
  sessionStorage.setItem('auth', JSON.stringify(state))
}

export function clearAuthState(): void {
  sessionStorage.removeItem('auth')
}

export interface AuthContextValue {
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

export const AuthContext = createContext<AuthContextValue | null>(null)

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}

export function useToken(): string | null {
  const stored = loadAuthState()
  return stored?.token ?? null
}
