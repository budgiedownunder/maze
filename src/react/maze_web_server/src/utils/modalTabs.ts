// Shared vocabulary + wiring for the tabbed modal dialogs. Kept out of the
// component file so it can only export components (Fast Refresh). Pairs with
// `ModalTabStrip` in `components/ModalTabs.tsx`.

export interface ModalTab<T extends string = string> {
  id: T
  label: string
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
