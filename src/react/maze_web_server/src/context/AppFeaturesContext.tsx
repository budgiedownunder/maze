import { createContext, useContext, useEffect, useState } from 'react'
import * as api from '../api/client'
import type { AppFeatures } from '../types/api'

const defaults: AppFeatures = { allow_signup: true }

export const AppFeaturesContext = createContext<AppFeatures>(defaults)

export function AppFeaturesProvider({ children }: { children: React.ReactNode }) {
  const [features, setFeatures] = useState<AppFeatures>(defaults)

  useEffect(() => {
    api.getFeatures()
      .then(setFeatures)
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
