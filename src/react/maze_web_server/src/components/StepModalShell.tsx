import type { ReactNode } from 'react'
import { ModalTabStrip } from './ModalTabs'
import { WizardStepRail } from './WizardStepRail'
import type { WizardStep } from '../utils/modalTabs'

interface StepModalShellProps<T extends string> {
  // 'tabs' (all steps visible, jump freely — the edit/Properties presentation)
  // or 'wizard' (left step-rail + Back/Next/Finish — the create presentation).
  mode: 'tabs' | 'wizard'
  title: string
  steps: readonly WizardStep<T>[]
  activeStep: T
  onStepChange: (id: T) => void
  /** Namespace for the tab/panel ids, shared with the panels' `modalTabPanelProps`. */
  idPrefix: string
  /** Accessible name for the tab strip / step rail. */
  ariaLabel: string
  /** The step panels (each rendered with `modalTabPanelProps`). */
  children: ReactNode
  /** Pinned content above the action buttons — e.g. a validation error. */
  footerNote?: ReactNode
  onCancel: () => void
  onCommit: () => void
  /**
   * Whether the commit action is available. In wizard mode this is the
   * **early-Finish** gate: Finish is offered on every step, enabled as soon as
   * the whole form validates (not gated to the last step).
   */
  canCommit: boolean
  /** Commit-button label; defaults to Finish (wizard) / Save (tabs). */
  commitLabel?: string
  /** Wizard-only: step ids to mark complete in the rail. */
  completed?: readonly T[]
}

// The dual-mode modal shell for the definition editor: a tab strip (edit) or a
// left step-rail wizard (create) around a shared set of step panels, with a
// pinned footer. Tabs mode: Cancel + Save. Wizard mode: Cancel + Back + Next +
// Finish, where Finish honours `canCommit` from any step (early-Finish).
export function StepModalShell<T extends string>({
  mode,
  title,
  steps,
  activeStep,
  onStepChange,
  idPrefix,
  ariaLabel,
  children,
  footerNote,
  onCancel,
  onCommit,
  canCommit,
  commitLabel,
  completed,
}: StepModalShellProps<T>) {
  const activeIndex = steps.findIndex(s => s.id === activeStep)
  const isFirst = activeIndex <= 0
  const isLast = activeIndex === steps.length - 1
  const commit = commitLabel ?? (mode === 'wizard' ? 'Finish' : 'Save')

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-label={title} style={{ zIndex: 1200 }}>
      <div className={`modal modal-with-scroll-body ${mode === 'wizard' ? 'modal-wizard' : 'modal-md'}`}>
        <h2 className="modal-title">{title}</h2>

        {mode === 'wizard' ? (
          <div className="wizard-body">
            <WizardStepRail
              steps={steps}
              activeStep={activeStep}
              onSelect={onStepChange}
              idPrefix={idPrefix}
              ariaLabel={ariaLabel}
              completed={completed}
            />
            <div className="modal-scroll-body wizard-content">{children}</div>
          </div>
        ) : (
          <>
            <ModalTabStrip
              tabs={steps}
              activeTab={activeStep}
              onSelect={onStepChange}
              idPrefix={idPrefix}
              ariaLabel={ariaLabel}
            />
            <div className="modal-scroll-body">{children}</div>
          </>
        )}

        <div className="modal-tab-footer">
          {footerNote}
          <div className="modal-actions-row">
            <button type="button" className="btn-gray" onClick={onCancel}>Cancel</button>
            {mode === 'wizard' && (
              <button type="button" className="btn-gray" onClick={() => onStepChange(steps[activeIndex - 1].id)} disabled={isFirst}>
                Back
              </button>
            )}
            {mode === 'wizard' && !isLast && (
              <button type="button" className="btn-primary" onClick={() => onStepChange(steps[activeIndex + 1].id)}>
                Next
              </button>
            )}
            <button type="button" className="btn-primary" onClick={onCommit} disabled={!canCommit}>
              {commit}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
