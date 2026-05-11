import { createContext, useContext } from 'react'
import type { AppFeatures } from '../types/api'

// Fail-open defaults: if the features endpoint is unreachable we still let
// the user attempt to sign up / sign in. OAuth providers fail closed though
// because we'd have no display names to render — better to hide the buttons
// than to render half-broken ones. email_enabled also fails closed: if we
// can't reach the server, hide the email-dependent surfaces (verification
// banners, password reset) rather than promise behaviour we can't deliver.
// max_maze_cells fails open as `null` — no client-side cap until the server
// reports one; the server will still reject over-cap requests with HTTP 422.
export const APP_FEATURES_DEFAULTS: AppFeatures = { allow_signup: true, oauth_providers: [], email_enabled: false, max_maze_cells: null }

export const AppFeaturesContext = createContext<AppFeatures>(APP_FEATURES_DEFAULTS)

export function useAppFeatures(): AppFeatures {
  return useContext(AppFeaturesContext)
}
