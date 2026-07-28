// Shared vocabulary + wiring for the tabbed modal dialogs. Kept out of the
// component file so it can only export components (Fast Refresh). Pairs with
// `ModalTabStrip` in `components/ModalTabs.tsx`.

import type { ReactNode } from 'react'

export interface ModalTab<T extends string = string> {
  id: T
  label: string
}

// A wizard step is a tab with an optional rail icon (a theme-aware SVG node is
// recommended so it adapts to light/dark).
export interface WizardStep<T extends string = string> extends ModalTab<T> {
  icon?: ReactNode
}

// Root-element props for a tab panel paired with `ModalTabStrip`: the role, the
// id/label wiring back to its tab, and the `hidden` toggle for inactive panels.
// Spread onto the panel's root `<div>`.
export function modalTabPanelProps(idPrefix: string, tabId: string, activeTab: string) {
  return {
    role: 'tabpanel' as const,
    id: `${idPrefix}-panel-${tabId}`,
    'aria-labelledby': `${idPrefix}-tab-${tabId}`,
    hidden: activeTab !== tabId,
  }
}
