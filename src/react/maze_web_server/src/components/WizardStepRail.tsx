import type { KeyboardEvent } from 'react'
import type { WizardStep } from '../utils/modalTabs'

interface WizardStepRailProps<T extends string> {
  steps: readonly WizardStep<T>[]
  activeStep: T
  onSelect: (id: T) => void
  /** Namespace for the tab/panel ids, shared with the panels' `modalTabPanelProps`. */
  idPrefix: string
  /** Accessible name for the rail. */
  ariaLabel: string
  /** Step ids to mark visually complete (a check in the marker). */
  completed?: readonly T[]
}

// The wizard's left navigation rail. It is semantically a **vertical tablist**
// (Up/Down move between steps, Home/End jump to the ends) — steps are freely
// jumpable because the editor's form state is fully defaulted, so there is no
// linear gating — and it pairs with the same `role="tabpanel"` panels the tabs
// mode uses (matching `id`/`aria-controls` wiring). Visually it is the softer
// stepped rail: a per-step marker (icon / number / done-check) plus a label,
// with current / done / upcoming states.
export function WizardStepRail<T extends string>({ steps, activeStep, onSelect, idPrefix, ariaLabel, completed }: WizardStepRailProps<T>) {
  const completedSet = new Set<T>(completed ?? [])

  function handleKeyDown(e: KeyboardEvent, index: number) {
    let next: number | null = null
    if (e.key === 'ArrowDown') next = (index + 1) % steps.length
    else if (e.key === 'ArrowUp') next = (index - 1 + steps.length) % steps.length
    else if (e.key === 'Home') next = 0
    else if (e.key === 'End') next = steps.length - 1
    if (next === null) return
    e.preventDefault()
    onSelect(steps[next].id)
  }

  return (
    <div className="wizard-rail" role="tablist" aria-orientation="vertical" aria-label={ariaLabel}>
      {steps.map((step, index) => {
        const isActive = step.id === activeStep
        const isDone = completedSet.has(step.id)
        return (
          <button
            key={step.id}
            type="button"
            role="tab"
            id={`${idPrefix}-tab-${step.id}`}
            aria-selected={isActive}
            aria-controls={`${idPrefix}-panel-${step.id}`}
            aria-current={isActive ? 'step' : undefined}
            tabIndex={isActive ? 0 : -1}
            className="wizard-step"
            data-state={isActive ? 'current' : isDone ? 'done' : 'upcoming'}
            onClick={() => onSelect(step.id)}
            onKeyDown={e => handleKeyDown(e, index)}
          >
            <span className="wizard-step-marker" aria-hidden="true">
              {step.icon ?? (isDone ? '✓' : index + 1)}
            </span>
            <span className="wizard-step-label">{step.label}</span>
          </button>
        )
      })}
    </div>
  )
}
