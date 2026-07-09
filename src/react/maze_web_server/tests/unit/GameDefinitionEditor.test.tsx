import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
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
  it('renders the five tabs (General first) with the General fields; no Levels or Decor tab', () => {
    renderEditor()
    for (const label of ['General', 'Scene', 'Layout', 'Objects', 'Advanced']) {
      expect(screen.getByRole('tab', { name: label })).toBeInTheDocument()
    }
    // There is no Levels tab — the multi-level settings are distributed across
    // the other tabs and revealed by the count. Decor was folded into Scene.
    expect(screen.queryByRole('tab', { name: 'Levels' })).toBeNull()
    expect(screen.queryByRole('tab', { name: 'Decor' })).toBeNull()
    expect(screen.getByRole('tab', { name: 'General' })).toHaveAttribute('aria-current', 'step')
    // The count + time limit live on General alongside name/description.
    for (const label of ['Name', 'Description', 'Levels', 'Time limit (seconds)']) {
      expect(screen.getByLabelText(label)).toBeVisible()
    }
  })

  it('shows the Grid group on the Layout step', async () => {
    renderEditor()
    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    // Single-level default: the group is just "Grid" and there is no Levels group.
    const grid = within(screen.getByRole('group', { name: 'Grid' }))
    expect(grid.getByLabelText('Rows')).toBeVisible()
    expect(grid.getByLabelText('Min Start to Finish Distance')).toBeVisible()
    expect(screen.queryByRole('group', { name: 'Levels' })).toBeNull()
  })

  it('relabels the Grid group and adds the Levels group when the game is multi-level', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    // Raise the count on General, then look at the Layout tab.
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '3' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))

    // The grid group is now captioned for the ground floor…
    expect(screen.queryByRole('group', { name: 'Grid' })).toBeNull()
    expect(within(screen.getByRole('group', { name: 'Ground Floor Grid' })).getByLabelText('Rows')).toBeVisible()

    // …and the Levels progression group appears with the moved (renamed) fields.
    const levels = within(screen.getByRole('group', { name: 'Levels' }))
    expect(levels.getByLabelText('Difficulty Change')).toBeVisible()
    expect(levels.getByLabelText('Alignment')).toBeVisible()
    expect(levels.getByRole('checkbox', { name: 'Taper' })).toBeVisible()
  })

  it('shows the grouped object fields on the Objects step (count next to style)', async () => {
    renderEditor()
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    const doors = within(screen.getByRole('group', { name: 'Doors' }))
    expect(doors.getByLabelText('Count')).toBeVisible()
    expect(doors.getByLabelText('Spares')).toBeVisible()
    expect(doors.getByLabelText('Style')).toBeVisible()
    expect(within(screen.getByRole('group', { name: 'Treasure' })).getByLabelText('Count')).toBeVisible()
  })

  it('reveals the Finish Type at the bottom of the Objects tab only when multi-level', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    expect(screen.queryByLabelText('Finish Type')).toBeNull()

    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '3' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    expect(screen.getByLabelText('Finish Type')).toBeVisible()

    // Back to single-level hides it again.
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '1' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    expect(screen.queryByLabelText('Finish Type')).toBeNull()
  })

  it('groups the Advanced tab and reveals its Levels group only when multi-level', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))

    // The always-present groups with their renamed fields.
    expect(within(screen.getByRole('group', { name: 'Health & Enemies' })).getByLabelText('Max HP')).toBeVisible()
    expect(within(screen.getByRole('group', { name: 'Minimap' })).getByLabelText('Cell size (px)')).toBeVisible()
    expect(within(screen.getByRole('group', { name: 'Titles' })).getByLabelText('Splash')).toBeVisible()
    // No Levels group at count 1.
    expect(screen.queryByRole('group', { name: 'Levels' })).toBeNull()

    // Raising the count reveals the Advanced Levels group with the per-level toggles.
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '2' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    const levels = within(screen.getByRole('group', { name: 'Levels' }))
    expect(levels.getByRole('checkbox', { name: 'Reset item bag each level' })).toBeVisible()
    expect(levels.getByRole('checkbox', { name: 'Randomise perimeter each level' })).toBeVisible()
    expect(levels.getByRole('checkbox', { name: 'Hide cleared-level enemies' })).toBeVisible()
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
    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '2' } })
    expect(screen.getByRole('alert')).toHaveTextContent('Rows must be a whole number of 3 or more.')
    expect(commitButton()).toBeDisabled()
  })

  it('keeps the generation error visible from another step (pinned footer)', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    fireEvent.change(within(screen.getByRole('group', { name: 'Enemies' })).getByLabelText('Count'), { target: { value: '9' } })
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
    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    fireEvent.change(screen.getByLabelText('Min Start to Finish Distance'), { target: { value: '0' } })
    expect(screen.queryByRole('alert')).toBeNull()
    expect(commitButton()).toBeEnabled()
  })
})

describe('GameDefinitionEditor — commit', () => {
  it('builds the request from the edited form on Finish', async () => {
    const { onSubmit } = renderEditor()
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.type(screen.getByLabelText('Description'), 'Climb it')

    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '12' } })

    // The feature counts live on the Objects tab now, grouped by object kind.
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    fireEvent.change(within(screen.getByRole('group', { name: 'Treasure' })).getByLabelText('Count'), { target: { value: '3' } })

    // The decor toggles now live in the Scene tab's Decoration group.
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    await userEvent.click(within(screen.getByRole('group', { name: 'Decoration' })).getByRole('checkbox', { name: 'Floor junctions' }))

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

  it('flows the distributed multi-level controls and the Scene final-level override into the built config', async () => {
    const { onSubmit } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // The count is on General and reveals the multi-level controls across tabs.
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '4' } })

    // Finish Type is at the bottom of the Objects tab.
    await userEvent.click(screen.getByRole('tab', { name: 'Objects' }))
    fireEvent.change(screen.getByLabelText('Finish Type'), { target: { value: 'portal' } })

    // Difficulty Change + Taper are on the Layout tab's Levels group.
    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    const levelsGroup = within(screen.getByRole('group', { name: 'Levels' }))
    fireEvent.change(levelsGroup.getByLabelText('Difficulty Change'), { target: { value: 'harder' } })
    await userEvent.click(levelsGroup.getByRole('checkbox', { name: 'Taper' }))

    // A per-level toggle lives in the Advanced tab's Levels group.
    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    await userEvent.click(within(screen.getByRole('group', { name: 'Levels' })).getByRole('checkbox', { name: 'Hide cleared-level enemies' }))

    // The final-level override is on the Scene tab; its sky select ("Final Level")
    // shows outright (multi-level) in the Sky group — no toggle. A perimeter
    // "Final Level" also exists in Walls.
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    fireEvent.change(within(screen.getByRole('group', { name: 'Sky' })).getByLabelText('Final Level'), { target: { value: 'day' } })

    await userEvent.click(commitButton())

    expect(onSubmit.mock.calls[0][0].config).toMatchObject({
      levels: expect.objectContaining({
        count: 4,
        finishType: 'portal',
        difficultyChange: 'harder',
        taper: true,
        hideCompletedEnemies: true,
        top: { skyType: 'day' },
      }),
    })
  })

  it('groups the Scene tab into Sky / Walls / Decoration with the renamed controls', async () => {
    renderEditor()
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))

    // Sky group: the sky dropdown (single-level → no final-level override yet).
    expect(within(screen.getByRole('group', { name: 'Sky' })).getByLabelText('Sky')).toBeVisible()

    // Walls group: quadrant + renamed Texture / Perimeter.
    const walls = within(screen.getByRole('group', { name: 'Walls' }))
    expect(walls.getByRole('checkbox', { name: 'Quadrant wall types' })).toBeVisible()
    expect(walls.getByLabelText('Texture')).toBeVisible()
    expect(walls.getByRole('checkbox', { name: 'Perimeter' })).toBeVisible()

    // Decoration group: the four renamed toggles (folded-in decor).
    const decoration = within(screen.getByRole('group', { name: 'Decoration' }))
    for (const name of ['Wall tints', 'Wall objects', 'Dead-end objects', 'Floor junctions']) {
      expect(decoration.getByRole('checkbox', { name })).toBeVisible()
    }
  })

  it('shows the Final Level overrides on the Scene tab only for a multi-level game', async () => {
    renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // Single-level: Scene has the sky dropdown but no Final Level overrides.
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    expect(within(screen.getByRole('group', { name: 'Sky' })).getByLabelText('Sky')).toBeVisible()
    expect(screen.queryByLabelText('Final Level')).toBeNull()

    // Raise the count on General → a Final Level select appears in Sky and Walls.
    await userEvent.click(screen.getByRole('tab', { name: 'General' }))
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Levels' }), { target: { value: '2' } })
    await userEvent.click(screen.getByRole('tab', { name: 'Scene' }))
    expect(within(screen.getByRole('group', { name: 'Sky' })).getByLabelText('Final Level')).toBeVisible()
    expect(within(screen.getByRole('group', { name: 'Walls' })).getByLabelText('Final Level')).toBeVisible()
  })

  it('flows the General time limit and edited Advanced controls into the built config', async () => {
    const { onSubmit } = renderEditor({ initialForm: { ...DEFINITION_DEFAULTS, name: 'Tower' } })

    // Time limit is on General; the rest on Advanced.
    fireEvent.change(screen.getByLabelText('Time limit (seconds)'), { target: { value: '90' } })

    await userEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    fireEvent.change(within(screen.getByRole('group', { name: 'Health & Enemies' })).getByLabelText('Max HP'), { target: { value: '5' } })
    fireEvent.change(within(screen.getByRole('group', { name: 'Minimap' })).getByLabelText('Radius (cells)'), { target: { value: '8' } })
    // An explicit splash title overrides the name-seeding; the status-bar label
    // is left blank, so it still falls back to the name.
    await userEvent.type(within(screen.getByRole('group', { name: 'Titles' })).getByLabelText('Splash'), 'Ascend!')

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
