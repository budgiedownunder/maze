import { useEffect, useState } from 'react'
import * as api from '../api/client'
import type { AppFeatures } from '../types/api'
import { AppFeaturesContext, APP_FEATURES_DEFAULTS } from './AppFeaturesContext'

export function AppFeaturesProvider({ children }: { children: React.ReactNode }) {
  const [features, setFeatures] = useState<AppFeatures>(APP_FEATURES_DEFAULTS)

  useEffect(() => {
    api.getFeatures()
      .then(f => setFeatures({ ...APP_FEATURES_DEFAULTS, ...f, oauth_providers: f.oauth_providers ?? [] }))
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
