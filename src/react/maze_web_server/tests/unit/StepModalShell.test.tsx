import { describe, it, expect, vi } from 'vitest'
import { useState } from 'react'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { StepModalShell } from '../../src/components/StepModalShell'
import { modalTabPanelProps, type WizardStep } from '../../src/utils/modalTabs'

type StepId = 'a' | 'b' | 'c'
const STEPS = [
  { id: 'a', label: 'Alpha' },
  { id: 'b', label: 'Beta' },
  { id: 'c', label: 'Gamma' },
] as const satisfies readonly WizardStep[]

function panels(active: StepId) {
  return (
    <>
      <div {...modalTabPanelProps('ed', 'a', active)}>Panel A</div>
      <div {...modalTabPanelProps('ed', 'b', active)}>Panel B</div>
      <div {...modalTabPanelProps('ed', 'c', active)}>Panel C</div>
    </>
  )
}

interface Overrides {
  mode?: 'tabs' | 'wizard'
  activeStep?: StepId
  canCommit?: boolean
  onPreview?: () => void
  canPreview?: boolean
}

function renderShell(over: Overrides = {}) {
  const onStepChange = vi.fn()
  const onCommit = vi.fn()
  const onCancel = vi.fn()
  const active = over.activeStep ?? 'a'
  render(
    <StepModalShell
      mode={over.mode ?? 'tabs'}
      title="Editor"
      steps={STEPS}
      activeStep={active}
      onStepChange={onStepChange}
      idPrefix="ed"
      ariaLabel="Editor steps"
      onCancel={onCancel}
      onCommit={onCommit}
      canCommit={over.canCommit ?? true}
      onPreview={over.onPreview}
      canPreview={over.canPreview}
    >
      {panels(active)}
    </StepModalShell>,
  )
  return { onStepChange, onCommit, onCancel }
}

describe('StepModalShell — tabs mode', () => {
  it('renders a horizontal tab strip and a Save action, no Back/Next', () => {
    renderShell({ mode: 'tabs' })
    const tablist = screen.getByRole('tablist', { name: 'Editor steps' })
    expect(tablist).not.toHaveAttribute('aria-orientation', 'vertical')
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Back' })).toBeNull()
    expect(screen.queryByRole('button', { name: 'Next' })).toBeNull()
  })

  it('disables Save when canCommit is false', () => {
    renderShell({ mode: 'tabs', canCommit: false })
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })

  it('fires onCommit when Save is clicked', async () => {
    const { onCommit } = renderShell({ mode: 'tabs', canCommit: true })
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    expect(onCommit).toHaveBeenCalledOnce()
  })

  it('selecting a tab reports the step change', async () => {
    const { onStepChange } = renderShell({ mode: 'tabs' })
    await userEvent.click(screen.getByRole('tab', { name: 'Beta' }))
    expect(onStepChange).toHaveBeenCalledWith('b')
  })
})

describe('StepModalShell — preview action', () => {
  it('shows no Preview button without onPreview', () => {
    renderShell({})
    expect(screen.queryByRole('button', { name: 'Preview' })).toBeNull()
  })

  it('shows a Preview button (in wizard mode too) when onPreview is provided', () => {
    renderShell({ mode: 'wizard', onPreview: vi.fn(), canPreview: true })
    expect(screen.getByRole('button', { name: 'Preview' })).toBeVisible()
  })

  it('gates Preview on canPreview, independently of canCommit', () => {
    const onPreview = vi.fn()
    // Commit is blocked but preview is allowed — distinct gates.
    renderShell({ onPreview, canPreview: true, canCommit: false })
    expect(screen.getByRole('button', { name: 'Preview' })).toBeEnabled()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })

  it('disables Preview when canPreview is false', () => {
    renderShell({ onPreview: vi.fn(), canPreview: false })
    expect(screen.getByRole('button', { name: 'Preview' })).toBeDisabled()
  })

  it('fires onPreview when clicked', async () => {
    const onPreview = vi.fn()
    renderShell({ onPreview, canPreview: true })
    await userEvent.click(screen.getByRole('button', { name: 'Preview' }))
    expect(onPreview).toHaveBeenCalledOnce()
  })
})

describe('StepModalShell — wizard mode', () => {
  it('renders a vertical step rail with Cancel / Back / Next / Finish', () => {
    renderShell({ mode: 'wizard', activeStep: 'b' })
    expect(screen.getByRole('tablist', { name: 'Editor steps' })).toHaveAttribute('aria-orientation', 'vertical')
    for (const name of ['Cancel', 'Back', 'Next', 'Finish']) {
      expect(screen.getByRole('button', { name })).toBeInTheDocument()
    }
  })

  it('disables Back on the first step', () => {
    renderShell({ mode: 'wizard', activeStep: 'a' })
    expect(screen.getByRole('button', { name: 'Back' })).toBeDisabled()
  })

  it('hides Next on the last step but still offers Finish', () => {
    renderShell({ mode: 'wizard', activeStep: 'c' })
    expect(screen.queryByRole('button', { name: 'Next' })).toBeNull()
    expect(screen.getByRole('button', { name: 'Finish' })).toBeInTheDocument()
  })

  it('Next / Back move to the adjacent step', async () => {
    const { onStepChange } = renderShell({ mode: 'wizard', activeStep: 'b' })
    await userEvent.click(screen.getByRole('button', { name: 'Next' }))
    expect(onStepChange).toHaveBeenCalledWith('c')
    await userEvent.click(screen.getByRole('button', { name: 'Back' }))
    expect(onStepChange).toHaveBeenCalledWith('a')
  })

  it('offers Finish early on a non-last step, disabled until canCommit', () => {
    // Step 'a' is the first (non-last) step, yet Finish is present — the
    // early-Finish affordance — just disabled until the form validates.
    renderShell({ mode: 'wizard', activeStep: 'a', canCommit: false })
    expect(screen.getByRole('button', { name: 'Finish' })).toBeDisabled()
  })

  it('fires onCommit when Finish is clicked', async () => {
    const { onCommit } = renderShell({ mode: 'wizard', activeStep: 'a', canCommit: true })
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))
    expect(onCommit).toHaveBeenCalledOnce()
  })

  it('the rail reports jumps via click and Down-arrow', async () => {
    const { onStepChange } = renderShell({ mode: 'wizard', activeStep: 'a' })
    await userEvent.click(screen.getByRole('tab', { name: 'Gamma' }))
    expect(onStepChange).toHaveBeenCalledWith('c')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Alpha' }), { key: 'ArrowDown' })
    expect(onStepChange).toHaveBeenCalledWith('b')
  })

  it('marks the active step with aria-current', () => {
    renderShell({ mode: 'wizard', activeStep: 'b' })
    expect(screen.getByRole('tab', { name: 'Beta' })).toHaveAttribute('aria-current', 'step')
    expect(screen.getByRole('tab', { name: 'Alpha' })).not.toHaveAttribute('aria-current')
  })
})

describe('StepModalShell — panel visibility', () => {
  function Harness({ mode }: { mode: 'tabs' | 'wizard' }) {
    const [active, setActive] = useState<StepId>('a')
    return (
      <StepModalShell
        mode={mode}
        title="Editor"
        steps={STEPS}
        activeStep={active}
        onStepChange={setActive}
        idPrefix="ed"
        ariaLabel="Editor steps"
        onCancel={() => {}}
        onCommit={() => {}}
        canCommit
      >
        {panels(active)}
      </StepModalShell>
    )
  }

  it('shows only the active step panel and switches on navigation (wizard)', async () => {
    render(<Harness mode="wizard" />)
    expect(screen.getByText('Panel A')).toBeVisible()
    expect(screen.getByText('Panel B')).not.toBeVisible()

    await userEvent.click(screen.getByRole('button', { name: 'Next' }))
    expect(screen.getByText('Panel B')).toBeVisible()
    expect(screen.getByText('Panel A')).not.toBeVisible()
  })

  it('shows only the active step panel and switches on tab click (tabs)', async () => {
    render(<Harness mode="tabs" />)
    expect(screen.getByText('Panel A')).toBeVisible()
    await userEvent.click(screen.getByRole('tab', { name: 'Gamma' }))
    expect(screen.getByText('Panel C')).toBeVisible()
    expect(screen.getByText('Panel A')).not.toBeVisible()
  })
})
