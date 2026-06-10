import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { MazeGameSettingsModal } from './MazeGameSettingsModal'
import {
  MAZE_GAME_SETTINGS_STORAGE_KEY,
  MAZE_GAME_SETTINGS_DEFAULTS,
  normalizeMazeGameSettings,
  saveMazeGameSettings,
  type MazeGameSettings,
} from '../utils/mazeGameSettings'

describe('MazeGameSettingsModal', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.restoreAllMocks()
  })

  it('renders sky options with Title-Cased labels but lowercase wire values', () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const sky = screen.getByLabelText(/sky/i) as HTMLSelectElement
    const labels = Array.from(sky.options).map(o => o.textContent)
    const values = Array.from(sky.options).map(o => o.value)
    expect(labels).toEqual(['Night', 'Sunrise', 'Day', 'Sunset', 'Dungeon', 'Chamber'])
    expect(values).toEqual(['night', 'sunrise', 'day', 'sunset', 'dungeon', 'chamber'])
  })

  it('renders wall texture options Title-Cased with underscore replaced', () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const wall = screen.getByLabelText(/wall texture/i) as HTMLSelectElement
    const labels = Array.from(wall.options).map(o => o.textContent)
    const values = Array.from(wall.options).map(o => o.value)
    // The four solid textures plus the three non-occluding types.
    expect(labels).toEqual([
      'Brick',
      'Dressed Stone',
      'Wood',
      'Cobblestone',
      'Water',
      'Lava',
      'Iron Fence',
    ])
    expect(values).toEqual([
      'brick',
      'dressed_stone',
      'wood',
      'cobblestone',
      'water',
      'lava',
      'iron_fence',
    ])
  })

  it('disables wall texture and wall tint when material variation is on', async () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const materialVariation = screen.getByLabelText(/quadrant wall types/i)
    expect(materialVariation).not.toBeChecked()
    const wallTexture = screen.getByLabelText(/wall texture/i) as HTMLSelectElement
    const wallTint = screen.getByLabelText(/varied wall tints/i) as HTMLInputElement
    expect(wallTexture).not.toBeDisabled()
    expect(wallTint).not.toBeDisabled()

    await userEvent.click(materialVariation)
    expect(wallTexture).toBeDisabled()
    expect(wallTint).toBeDisabled()
  })

  it('forces and disables the perimeter-walls checkbox under an enclosed sky', async () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const perimeter = screen.getByLabelText(/perimeter walls/i) as HTMLInputElement
    // Default open sky (night): on by default and editable.
    expect(perimeter).toBeChecked()
    expect(perimeter).not.toBeDisabled()
    // Enclosed sky always walls the perimeter → forced on and disabled.
    await userEvent.selectOptions(screen.getByLabelText(/sky/i) as HTMLSelectElement, 'dungeon')
    expect(perimeter).toBeChecked()
    expect(perimeter).toBeDisabled()
  })

  it('rejects submission with non-positive time limit and shows an inline error', async () => {
    const onSubmit = vi.fn()
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={onSubmit} />)
    const timer = screen.getByLabelText(/time limit/i) as HTMLInputElement
    await userEvent.clear(timer)
    await userEvent.type(timer, '0')
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onSubmit).not.toHaveBeenCalled()
    expect(screen.getByRole('alert')).toHaveTextContent(/time limit must be a positive/i)
  })

  it('calls onSubmit with the form values on valid submission', async () => {
    const onSubmit = vi.fn()
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={onSubmit} />)
    const timer = screen.getByLabelText(/time limit/i) as HTMLInputElement
    await userEvent.clear(timer)
    await userEvent.type(timer, '120')
    const sky = screen.getByLabelText(/sky/i) as HTMLSelectElement
    await userEvent.selectOptions(sky, 'sunset')
    // Door style and key holder live on the Objects tab — activate it before
    // interacting with their (otherwise hidden) controls.
    await userEvent.click(screen.getByRole('tab', { name: /objects/i }))
    const door = screen.getByLabelText(/door style/i) as HTMLSelectElement
    await userEvent.selectOptions(door, 'portcullis')
    const keyHolder = screen.getByLabelText(/key holder/i) as HTMLSelectElement
    await userEvent.selectOptions(keyHolder, 'chest')
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onSubmit).toHaveBeenCalledTimes(1)
    const settings = onSubmit.mock.calls[0][0] as MazeGameSettings
    expect(settings.skyType).toBe('sunset')
    expect(settings.doorStyle).toBe('portcullis')
    expect(settings.keyHolder).toBe('chest')
    expect(settings.timerSeconds).toBe(120)
  })

  it('calls onCancel when the Cancel button is clicked', async () => {
    const onCancel = vi.fn()
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={onCancel} onSubmit={() => {}} />)
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })

  it('groups fields into Scene / Objects / Decor tabs and switches panels on click', async () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    // Scene is the default active tab: its panel is visible, the others hidden.
    const scenePanel = screen.getByRole('tabpanel', { name: /scene/i })
    expect(scenePanel).toBeVisible()
    expect(scenePanel).toContainElement(screen.getByLabelText(/sky/i))
    // The Objects panel exists but is hidden until its tab is selected.
    expect(screen.getByRole('tab', { name: /objects/i })).toHaveAttribute('aria-selected', 'false')
    // getByRole excludes hidden tabpanels by default, so the Objects panel is
    // not queryable while Scene is active — proving only one panel shows.
    expect(screen.queryByRole('tabpanel', { name: /objects/i })).toBeNull()

    await userEvent.click(screen.getByRole('tab', { name: /objects/i }))
    const objectsPanel = screen.getByRole('tabpanel', { name: /objects/i })
    expect(objectsPanel).toBeVisible()
    expect(objectsPanel).toContainElement(screen.getByLabelText(/door style/i))
    expect(screen.getByRole('tab', { name: /objects/i })).toHaveAttribute('aria-selected', 'true')
    // Switching tabs hides the Scene panel.
    expect(screen.queryByRole('tabpanel', { name: /scene/i })).toBeNull()

    // The time limit + actions stay reachable regardless of the active tab.
    expect(screen.getByLabelText(/time limit/i)).toBeVisible()
    expect(screen.getByRole('button', { name: /play/i })).toBeVisible()
  })

  it('pre-fills all fields from initialSettings', () => {
    const stored: MazeGameSettings = {
      skyType: 'day',
      wallType: 'wood',
      perimeterWalls: false,
      doorStyle: 'dissolve',
      keyHolder: 'floating_key',
      enemyType: 'ghost',
      healthStyle: 'potion',
      wallTint: true,
      wallMaterialVariation: false,
      deadEndObjects: false,
      wallDecorations: false,
      floorAccents: true,
      timerSeconds: 180,
    }
    render(
      <MazeGameSettingsModal
        mazeName="My Maze"
        initialSettings={stored}
        onCancel={() => {}}
        onSubmit={() => {}}
      />,
    )
    expect((screen.getByLabelText(/sky/i) as HTMLSelectElement).value).toBe('day')
    expect((screen.getByLabelText(/wall texture/i) as HTMLSelectElement).value).toBe('wood')
    // Open sky (day) + stored false → the perimeter-walls box reflects it.
    expect(screen.getByLabelText(/perimeter walls/i)).not.toBeChecked()
    expect((screen.getByLabelText(/door style/i) as HTMLSelectElement).value).toBe('dissolve')
    expect((screen.getByLabelText(/key holder/i) as HTMLSelectElement).value).toBe('floating_key')
    expect((screen.getByLabelText(/enemy type/i) as HTMLSelectElement).value).toBe('ghost')
    expect((screen.getByLabelText(/health style/i) as HTMLSelectElement).value).toBe('potion')
    expect(screen.getByLabelText(/varied wall tints/i)).toBeChecked()
    expect(screen.getByLabelText(/dead-end objects/i)).not.toBeChecked()
    expect(screen.getByLabelText(/sparse wall decorations/i)).not.toBeChecked()
    expect(screen.getByLabelText(/floor junction markers/i)).toBeChecked()
    expect((screen.getByLabelText(/time limit/i) as HTMLInputElement).value).toBe('180')
  })

  it('renders enemy type options Title-Cased with lowercase wire values', () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const enemy = screen.getByLabelText(/enemy type/i) as HTMLSelectElement
    const labels = Array.from(enemy.options).map(o => o.textContent)
    const values = Array.from(enemy.options).map(o => o.value)
    expect(labels).toEqual(['Goblin', 'Ghost'])
    expect(values).toEqual(['goblin', 'ghost'])
  })

  it('renders health style options Title-Cased with lowercase wire values', () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    const health = screen.getByLabelText(/health style/i) as HTMLSelectElement
    const labels = Array.from(health.options).map(o => o.textContent)
    const values = Array.from(health.options).map(o => o.value)
    expect(labels).toEqual(['Heart', 'Potion'])
    expect(values).toEqual(['heart', 'potion'])
  })

  it('defaults Enemy type and Health style to goblin / heart', () => {
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={() => {}} />)
    expect((screen.getByLabelText(/enemy type/i) as HTMLSelectElement).value).toBe('goblin')
    expect((screen.getByLabelText(/health style/i) as HTMLSelectElement).value).toBe('heart')
  })

  it('passes enemyType and healthStyle through onSubmit on submit', async () => {
    const onSubmit = vi.fn()
    render(<MazeGameSettingsModal mazeName="My Maze" onCancel={() => {}} onSubmit={onSubmit} />)
    // Enemy type and health style live on the Objects tab.
    await userEvent.click(screen.getByRole('tab', { name: /objects/i }))
    const enemy = screen.getByLabelText(/enemy type/i) as HTMLSelectElement
    await userEvent.selectOptions(enemy, 'ghost')
    const health = screen.getByLabelText(/health style/i) as HTMLSelectElement
    await userEvent.selectOptions(health, 'potion')
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onSubmit).toHaveBeenCalledTimes(1)
    const settings = onSubmit.mock.calls[0][0] as MazeGameSettings
    expect(settings.enemyType).toBe('ghost')
    expect(settings.healthStyle).toBe('potion')
  })

  it('renders a custom title and submit label (settings-editor mode)', () => {
    render(
      <MazeGameSettingsModal
        mazeName="My Maze"
        title="Game settings — My Maze"
        submitLabel="Save"
        onCancel={() => {}}
        onSubmit={() => {}}
      />,
    )
    expect(screen.getByRole('heading', { name: /game settings — my maze/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument()
    // The default "Play" label is absent in editor mode.
    expect(screen.queryByRole('button', { name: /^play$/i })).toBeNull()
  })
})

describe('normalizeMazeGameSettings', () => {
  // Casts an untrusted/partial bag (as if from a persisted maze or storage)
  // into the Partial the validator accepts, without TypeScript narrowing the
  // deliberately-invalid values away.
  const partial = (o: Record<string, unknown>): Partial<MazeGameSettings> =>
    o as Partial<MazeGameSettings>

  it('returns the defaults for an empty object', () => {
    expect(normalizeMazeGameSettings({})).toEqual(MAZE_GAME_SETTINGS_DEFAULTS)
  })

  it('falls back to default skyType when the value is unknown', () => {
    expect(normalizeMazeGameSettings(partial({ skyType: 'banana' })).skyType).toBe(
      MAZE_GAME_SETTINGS_DEFAULTS.skyType,
    )
  })

  it('falls back to default timerSeconds when the value is non-positive', () => {
    expect(normalizeMazeGameSettings({ timerSeconds: -5 }).timerSeconds).toBe(
      MAZE_GAME_SETTINGS_DEFAULTS.timerSeconds,
    )
  })

  it('falls back to default enemyType when the value is unknown', () => {
    expect(normalizeMazeGameSettings(partial({ enemyType: 'dragon' })).enemyType).toBe(
      MAZE_GAME_SETTINGS_DEFAULTS.enemyType,
    )
  })

  it('falls back to default healthStyle when the value is unknown', () => {
    expect(normalizeMazeGameSettings(partial({ healthStyle: 'shield' })).healthStyle).toBe(
      MAZE_GAME_SETTINGS_DEFAULTS.healthStyle,
    )
  })

  it('returns a complete valid object unchanged', () => {
    const settings: MazeGameSettings = {
      skyType: 'sunset',
      wallType: 'cobblestone',
      perimeterWalls: false,
      doorStyle: 'slide',
      keyHolder: 'chest',
      enemyType: 'ghost',
      healthStyle: 'potion',
      wallTint: true,
      wallMaterialVariation: true,
      deadEndObjects: true,
      wallDecorations: false,
      floorAccents: true,
      timerSeconds: 240,
    }
    expect(normalizeMazeGameSettings(settings)).toEqual(settings)
  })
})

describe('saveMazeGameSettings', () => {
  beforeEach(() => localStorage.clear())

  it('writes the settings to localStorage as the launch handoff', () => {
    const settings: MazeGameSettings = {
      skyType: 'sunset',
      wallType: 'cobblestone',
      perimeterWalls: false,
      doorStyle: 'slide',
      keyHolder: 'chest',
      enemyType: 'ghost',
      healthStyle: 'potion',
      wallTint: true,
      wallMaterialVariation: true,
      deadEndObjects: true,
      wallDecorations: false,
      floorAccents: true,
      timerSeconds: 240,
    }
    saveMazeGameSettings(settings)
    expect(JSON.parse(localStorage.getItem(MAZE_GAME_SETTINGS_STORAGE_KEY)!)).toEqual(settings)
  })
})
