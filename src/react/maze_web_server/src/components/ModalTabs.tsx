import type { KeyboardEvent } from 'react'
import type { ModalTab } from '../utils/modalTabs'

// Shared tab scaffolding for the modal dialogs. Extracted from the game-settings
// and generate-maze modals, which each inlined an identical WAI-ARIA `tablist`
// (Left/Right arrow navigation, wrapping) plus matching `tabpanel` wiring. The
// dual-mode editor shell builds on the same primitives. The `ModalTab` type and
// the `modalTabPanelProps` panel-wiring helper live in `utils/modalTabs`.

interface ModalTabStripProps<T extends string> {
  tabs: readonly ModalTab<T>[]
  activeTab: T
  onSelect: (id: T) => void
  /** Namespace for the generated tab/panel ids, e.g. "launch" → `launch-tab-scene`. */
  idPrefix: string
  /** Accessible name for the tablist. */
  ariaLabel: string
}

// A WAI-ARIA `tablist`: Left/Right move between tabs (wrapping at the ends), the
// active tab is the only tab-stop, and each tab controls its paired panel. Pair
// each panel with `modalTabPanelProps` so the ids and `hidden` toggle line up.
export function ModalTabStrip<T extends string>({ tabs, activeTab, onSelect, idPrefix, ariaLabel }: ModalTabStripProps<T>) {
  function handleKeyDown(e: KeyboardEvent, index: number) {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return
    e.preventDefault()
    const delta = e.key === 'ArrowRight' ? 1 : -1
    const next = (index + delta + tabs.length) % tabs.length
    onSelect(tabs[next].id)
  }

  return (
    <div className="modal-tabs" role="tablist" aria-label={ariaLabel}>
      {tabs.map((tab, index) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          id={`${idPrefix}-tab-${tab.id}`}
          aria-selected={activeTab === tab.id}
          aria-controls={`${idPrefix}-panel-${tab.id}`}
          tabIndex={activeTab === tab.id ? 0 : -1}
          className="modal-tab"
          onClick={() => onSelect(tab.id)}
          onKeyDown={e => handleKeyDown(e, index)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  )
}
