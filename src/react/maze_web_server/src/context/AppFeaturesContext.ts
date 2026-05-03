import { createContext, useContext } from 'react'
import type { AppFeatures } from '../types/api'

// Fail-open defaults: if the features endpoint is unreachable we still let
// the user attempt to sign up / sign in. OAuth providers fail closed though
// because we'd have no display names to render — better to hide the buttons
// than to render half-broken ones.
export const APP_FEATURES_DEFAULTS: AppFeatures = { allow_signup: true, oauth_providers: [] }

export const AppFeaturesContext = createContext<AppFeatures>(APP_FEATURES_DEFAULTS)

export function useAppFeatures(): AppFeatures {
  return useContext(AppFeaturesContext)
}
