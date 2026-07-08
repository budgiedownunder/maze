import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GameDefinitionEditor } from '../../src/components/GameDefinitionEditor'
import { AppFeaturesContext } from '../../src/context/AppFeaturesContext'
import { DEFINITION_DEFAULTS, type DefinitionFormState } from '../../src/utils/definitionConfig'
import type { AppFeatures, GameDefinitionRequest } from '../../src/types/api'

const FEATURES: AppFeatures = { allow_signup: true, oauth_providers: [], email_enabled: false, max_maze_cells: null }

function renderEditor(over: { initialForm?: DefinitionFormState; maxMazeCells?: number | null; mode?: 'tabs' | 'wizard' } = {}) {
  const onSubmit = vi.fn<(request: GameDefinitionRequest) => void>()
  const onCancel = vi.fn()
  render(
    <AppFeaturesContext.Provider value={{ ...FEATURES, max_maze_cells: over.maxMazeCells ?? null }}>
      <GameDefinitionEditor
        mode={over.mode ?? 'wizard'}
        title="New game"
        initialForm={over.initialForm ?? DEFINITION_DEFAULTS}
        onSubmit={onSubmit}
        onCancel={onCancel}
      />
    </AppFeaturesContext.Provider>,
  )
  return { onSubmit, onCancel }
}

const commitButton = () => screen.getByRole('button', { name: 'Finish' })

describe('GameDefinitionEditor — steps', () => {
  it('renders General first with its identity + structure fields, no Levels tab at count 1', () => {
    renderEditor()
    for (const label of ['General', 'Generation', 'Scene', 'Objects', 'Decor', 'Advanced']) {
      expect(screen.getByRole('tab', { name: label })).toBeInTheDocument()
    }
    // Single-level by default, so the Levels tab is absent from the strip.
    expect(screen.queryByRole('tab', { name: 'Levels' })).toBeNull()
    expect(screen.getByRole('tab', { name: 'General' })).toHaveAttribute('aria-current', 'step')
    // The count + time limit live on General alongside name/description.
    for (const label of ['Name', 'Description', 'Levels', 'Time limit (seconds)']) {
      expect(screen.getByLabelText(label)).toBeVisible()
    }
  })

  it('shows the generation field-group on the Generation step', async () => {
    renderEditor()
    await userEvent.click(screen.getByRole('tab', { name: 'Generation' }))
    expect(screen.getByLabelText('Rows')).toBeVisible()
    expect(screen.getByLabelText('Treasure')).toBeVisible()
  })

  it('reveals the Levels tab only once the count is raised above 1, and hides it again', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    expect(screen.queryByRole('tab', { name: 'Levels' })).toBeNull()

    fireEvent.change(screen.getByLabelText('Levels'), { target: { value: '3' } })
    expect(screen.getByRole('tab', { name: 'Levels' })).toBeInTheDocument()

    await userEvent.click(screen.getByRole('tab', { name: 'Levels' }))
    expect(screen.getByLabelText('Finish Type')).toBeVisible()

    // Back to single-level: the tab disappears and the view falls back to General.
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByLabelText('Levels'), { target: { value: '1' } })
    expect(screen.queryByRole('tab', { name: 'Levels' })).toBeNull()
    expect(screen.getByRole('tab', { name: 'General' })).toHaveAttribute('aria-current', 'step')
  })

  it('shows the advanced field-group on the Advanced step', async () => {
    renderEditor()
    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    expect(screen.getByLabelText('Max HP')).toBeVisible()
    expect(screen.getByLabelText('Splash title')).toBeVisible()
  })
})

describe('GameDefinitionEditor — canCommit gating', () => {
  it('disables Finish until a non-empty name is entered', async () => {
    renderEditor()
    expect(commitButton()).toBeDisabled()
    await userEvent.type(screen.getByLabelText('Name'), '   ')
    expect(commitButton()).toBeDisabled()
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    expect(commitButton()).toBeEnabled()
  })

  it('offers early Finish from the first step once the form validates', () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    expect(screen.getByRole('tab', { name: 'General' })).toHaveAttribute('aria-current', 'step')
    expect(commitButton()).toBeEnabled()
  })

  it('disables Finish and reports the error when the generation fields are invalid', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Generation' }))
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '2' } })
    expect(screen.getByRole('alert')).toHaveTextContent('Rows must be a whole number of 3 or more.')
    expect(commitButton()).toBeDisabled()
  })

  it('keeps the generation error visible from another step (pinned footer)', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Generation' }))
    fireEvent.change(screen.getByLabelText('Enemies'), { target: { value: '9' } })
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    expect(screen.getByRole('alert')).toHaveTextContent('Enemies must be a whole number between 0 and 8.')
  })

  it('honours the server-reported cell cap', () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' }, maxMazeCells: 50 })
    // The 8×8 defaults are 64 cells, over a cap of 50.
    expect(screen.getByRole('alert')).toHaveTextContent('Total cells (rows × columns) cannot exceed 50.')
    expect(commitButton()).toBeDisabled()
  })

  it('accepts a min solution length of 0 (no minimum)', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Generation' }))
    fireEvent.change(screen.getByLabelText('Min Solution Length'), { target: { value: '0' } })
    expect(screen.queryByRole('alert')).toBeNull()
    expect(commitButton()).toBeEnabled()
  })
})

describe('GameDefinitionEditor — commit', () => {
  it('builds the request from the edited form on Finish', async () => {
    const { onSubmit } = renderEditor()
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.type(screen.getByLabelText('Description'), 'Climb it')

    await userEvent.click(screen.getByRole('tab', { name: 'Generation' }))
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '12' } })
    fireEvent.change(screen.getByLabelText('Treasure'), { target: { value: '3' } })

    await userEvent.click(screen.getByRole('tab', { name: 'Decor' }))
    await userEvent.click(screen.getByRole('checkbox', { name: 'Floor junction markers' }))

    await userEvent.click(commitButton())

    expect(onSubmit).toHaveBeenCalledOnce()
    const request = onSubmit.mock.calls[0][0]
    expect(request.name).toBe('Tower')
    expect(request.description).toBe('Climb it')
    expect(request.visibility).toBe('private')
    expect(request.rotation).toBe('static')
    expect(request.config).toMatchObject({
      rows: 12,
      cols: 8,
      treasureCount: 3,
      seed: 0,
      landmarks: expect.objectContaining({ floorAccents: !DEFINITION_DEFAULTS.decor.floorAccents }),
      levels: expect.objectContaining({ count: 1 }),
    })
  })

  it('flows the edited Levels controls and the Scene final-level override into the built config', async () => {
    const { onSubmit } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // The count is on General and reveals the Levels tab once above 1.
    fireEvent.change(screen.getByLabelText('Levels'), { target: { value: '4' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Levels' }))
    fireEvent.change(screen.getByLabelText('Finish Type'), { target: { value: 'portal' } })
    fireEvent.change(screen.getByLabelText('Difficulty Change'), { target: { value: 'harder' } })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Taper upper levels' }))

    // The final-level override now lives on the Scene tab.
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    await userEvent.click(screen.getByRole('checkbox', { name: 'Override final level appearance' }))
    fireEvent.change(screen.getByLabelText('Final Level Sky'), { target: { value: 'day' } })

    await userEvent.click(commitButton())

    expect(onSubmit.mock.calls[0][0].config).toMatchObject({
      levels: expect.objectContaining({
        count: 4,
        finishType: 'portal',
        difficultyChange: 'harder',
        taper: true,
        top: { skyType: 'day' },
      }),
    })
  })

  it('shows the final-level override on the Scene tab only for a multi-level game', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // Single-level: Scene has the scene fields but no final-level override.
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    expect(screen.getByLabelText('Sky')).toBeVisible()
    expect(screen.queryByRole('checkbox', { name: 'Override final level appearance' })).toBeNull()

    // Raise the count on General → the override appears on Scene.
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByLabelText('Levels'), { target: { value: '2' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    expect(screen.getByRole('checkbox', { name: 'Override final level appearance' })).toBeVisible()
  })

  it('flows the General time limit and edited Advanced controls into the built config', async () => {
    const { onSubmit } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // Time limit is on General; the rest on Advanced.
    fireEvent.change(screen.getByLabelText('Time limit (seconds)'), { target: { value: '90' } })

    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    fireEvent.change(screen.getByLabelText('Max HP'), { target: { value: '5' } })
    fireEvent.change(screen.getByLabelText('Minimap radius (cells)'), { target: { value: '8' } })
    // An explicit splash title overrides the name-seeding; the status-bar label
    // is left blank, so it still falls back to the name.
    await userEvent.type(screen.getByLabelText('Splash title'), 'Ascend!')

    await userEvent.click(commitButton())

    expect(onSubmit.mock.calls[0][0].config).toMatchObject({
      timerSeconds: 90,
      maxHp: 5,
      minimapRadius: 8,
      title: 'Ascend!',
      mode: 'Tower',
    })
  })

  it('trims the name and seeds title + mode from it when both are blank', async () => {
    const { onSubmit } = renderEditor()
    await userEvent.type(screen.getByLabelText('Name'), '  Tower  ')
    await userEvent.click(commitButton())

    const request = onSubmit.mock.calls[0][0]
    expect(request.name).toBe('Tower')
    expect(request.config).toMatchObject({ title: 'Tower', mode: 'Tower' })
  })

  it('leaves an explicit title / mode untouched', async () => {
    const { onSubmit } = renderEditor({
      initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower', title: 'Ascend!', mode: 'Endless' },
    })
    await userEvent.click(commitButton())
    expect(onSubmit.mock.calls[0][0].config).toMatchObject({ title: 'Ascend!', mode: 'Endless' })
  })

  it('stores an empty description as unset', async () => {
    const { onSubmit } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(commitButton())
    expect(onSubmit.mock.calls[0][0].description).toBeNull()
  })

  it('echoes the pass-through visibility / rotation / seed unchanged', async () => {
    const { onSubmit } = renderEditor({
      mode: 'tabs',
      initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower', visibility: 'public', rotation: 'daily', seed: 99 },
    })
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const request = onSubmit.mock.calls[0][0]
    expect(request.visibility).toBe('public')
    expect(request.rotation).toBe('daily')
    expect(request.config).toMatchObject({ seed: 99 })
  })

  it('cancels without submitting', async () => {
    const { onSubmit, onCancel } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(onCancel).toHaveBeenCalledOnce()
    expect(onSubmit).not.toHaveBeenCalled()
  })
})
