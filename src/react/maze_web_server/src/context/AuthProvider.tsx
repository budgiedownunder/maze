import { useCallback, useEffect, useRef, useState } from 'react'
import * as api from '../api/client'
import type { UserProfile } from '../types/api'
import {
  AuthContext,
  RENEWAL_CHECK_INTERVAL_MS,
  clearAuthState,
  loadAuthState,
  saveAuthState,
  shouldRenewToken,
  type AuthState,
} from './AuthContext'

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isLoading, setIsLoading] = useState(true)
  const [authState, setAuthState] = useState<AuthState | null>(null)
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const authStateRef = useRef<AuthState | null>(null)
  authStateRef.current = authState

  const stopRenewalInterval = useCallback(() => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }, [])

  const tryRenew = useCallback(async (state: AuthState): Promise<AuthState> => {
    if (shouldRenewToken(state.issuedAt, state.expiry)) {
      const renewed = await api.renewToken(state.token)
      return {
        token: renewed.login_token_id,
        issuedAt: new Date().toISOString(),
        expiry: renewed.login_token_expires_at,
      }
    }
    return state
  }, [])

  const startRenewalInterval = useCallback(() => {
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
  }, [stopRenewalInterval, tryRenew])

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
  }, [tryRenew, startRenewalInterval, stopRenewalInterval])

  const setAuthFromTokenResponse = useCallback(async (token: string, expiresAt: string) => {
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
  }, [startRenewalInterval])

  const login = useCallback(async (email: string, password: string) => {
    const response = await api.login(email, password)
    await setAuthFromTokenResponse(response.login_token_id, response.login_token_expires_at)
  }, [setAuthFromTokenResponse])

  const logout = useCallback(async () => {
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
  }, [authState, stopRenewalInterval])

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
