export type MenuVariant = 'hamburger' | 'topbar'

export function useMenuVariant(): MenuVariant {
  // TODO: return 'topbar' for larger screens once top navigation is designed.
  // Tracked: review navigation approach for desktop — hamburger may not be
  // the best use of available screen real estate on wider viewports.
  return 'hamburger'
}
