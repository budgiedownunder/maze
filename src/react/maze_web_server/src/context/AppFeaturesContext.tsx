import { createContext, useContext, useEffect, useState } from 'react'
import * as api from '../api/client'
import type { AppFeatures } from '../types/api'

// Fail-open defaults: if the features endpoint is unreachable we still let
// the user attempt to sign up / sign in. OAuth providers fail closed though
// because we'd have no display names to render — better to hide the buttons
// than to render half-broken ones.
const defaults: AppFeatures = { allow_signup: true, oauth_providers: [] }

export const AppFeaturesContext = createContext<AppFeatures>(defaults)

export function AppFeaturesProvider({ children }: { children: React.ReactNode }) {
  const [features, setFeatures] = useState<AppFeatures>(defaults)

  useEffect(() => {
    api.getFeatures()
      .then(f => setFeatures({ ...defaults, ...f, oauth_providers: f.oauth_providers ?? [] }))
      .catch(() => {
        // fail-open: defaults already set
      })
  }, [])

  return (
    <AppFeaturesContext.Provider value={features}>
      {children}
    </AppFeaturesContext.Provider>
  )
}

export function useAppFeatures(): AppFeatures {
  return useContext(AppFeaturesContext)
}
