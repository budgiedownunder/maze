import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { Play3dCustomLaunchModal } from './Play3dCustomLaunchModal'
import {
  PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY,
  PLAY3D_CUSTOM_LAUNCH_DEFAULTS,
  loadPlay3dCustomLaunchSettings,
  savePlay3dCustomLaunchSettings,
  type Play3dCustomLaunchSettings,
} from '../utils/play3dCustomLaunchSettings'

describe('Play3dCustomLaunchModal', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.restoreAllMocks()
  })

  it('renders sky options with Title-Cased labels but lowercase wire values', () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    const sky = screen.getByLabelText(/sky/i) as HTMLSelectElement
    const labels = Array.from(sky.options).map(o => o.textContent)
    const values = Array.from(sky.options).map(o => o.value)
    expect(labels).toEqual(['Night', 'Sunrise', 'Day', 'Sunset', 'Dungeon', 'Chamber'])
    expect(values).toEqual(['night', 'sunrise', 'day', 'sunset', 'dungeon', 'chamber'])
  })

  it('renders wall texture options Title-Cased with underscore replaced', () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    const wall = screen.getByLabelText(/wall texture/i) as HTMLSelectElement
    const labels = Array.from(wall.options).map(o => o.textContent)
    expect(labels).toEqual(['Brick', 'Dressed Stone', 'Wood', 'Cobblestone'])
  })

  it('disables wall texture and wall tint when material variation is on', async () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
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

  it('rejects submission with non-positive time limit and shows an inline error', async () => {
    const onPlay = vi.fn()
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={onPlay} />)
    const timer = screen.getByLabelText(/time limit/i) as HTMLInputElement
    await userEvent.clear(timer)
    await userEvent.type(timer, '0')
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onPlay).not.toHaveBeenCalled()
    expect(screen.getByRole('alert')).toHaveTextContent(/time limit must be a positive/i)
  })

  it('calls onPlay with the form values on valid submission', async () => {
    const onPlay = vi.fn()
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={onPlay} />)
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
    expect(onPlay).toHaveBeenCalledTimes(1)
    const settings = onPlay.mock.calls[0][0] as Play3dCustomLaunchSettings
    expect(settings.skyType).toBe('sunset')
    expect(settings.doorStyle).toBe('portcullis')
    expect(settings.keyHolder).toBe('chest')
    expect(settings.timerSeconds).toBe(120)
  })

  it('calls onCancel when the Cancel button is clicked', async () => {
    const onCancel = vi.fn()
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={onCancel} onPlay={() => {}} />)
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })

  it('groups fields into Scene / Objects / Decor tabs and switches panels on click', async () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
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

  it('pre-fills from localStorage when settings have been saved before', () => {
    const stored: Play3dCustomLaunchSettings = {
      skyType: 'day',
      wallType: 'wood',
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
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify(stored))
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    expect((screen.getByLabelText(/sky/i) as HTMLSelectElement).value).toBe('day')
    expect((screen.getByLabelText(/wall texture/i) as HTMLSelectElement).value).toBe('wood')
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
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    const enemy = screen.getByLabelText(/enemy type/i) as HTMLSelectElement
    const labels = Array.from(enemy.options).map(o => o.textContent)
    const values = Array.from(enemy.options).map(o => o.value)
    expect(labels).toEqual(['Goblin', 'Ghost'])
    expect(values).toEqual(['goblin', 'ghost'])
  })

  it('renders health style options Title-Cased with lowercase wire values', () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    const health = screen.getByLabelText(/health style/i) as HTMLSelectElement
    const labels = Array.from(health.options).map(o => o.textContent)
    const values = Array.from(health.options).map(o => o.value)
    expect(labels).toEqual(['Heart', 'Potion'])
    expect(values).toEqual(['heart', 'potion'])
  })

  it('defaults Enemy type and Health style to goblin / heart', () => {
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={() => {}} />)
    expect((screen.getByLabelText(/enemy type/i) as HTMLSelectElement).value).toBe('goblin')
    expect((screen.getByLabelText(/health style/i) as HTMLSelectElement).value).toBe('heart')
  })

  it('passes enemyType and healthStyle through onPlay on submit', async () => {
    const onPlay = vi.fn()
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={() => {}} onPlay={onPlay} />)
    // Enemy type and health style live on the Objects tab.
    await userEvent.click(screen.getByRole('tab', { name: /objects/i }))
    const enemy = screen.getByLabelText(/enemy type/i) as HTMLSelectElement
    await userEvent.selectOptions(enemy, 'ghost')
    const health = screen.getByLabelText(/health style/i) as HTMLSelectElement
    await userEvent.selectOptions(health, 'potion')
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onPlay).toHaveBeenCalledTimes(1)
    const settings = onPlay.mock.calls[0][0] as Play3dCustomLaunchSettings
    expect(settings.enemyType).toBe('ghost')
    expect(settings.healthStyle).toBe('potion')
  })
})

describe('loadPlay3dCustomLaunchSettings', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('returns the defaults when no setting is stored', () => {
    expect(loadPlay3dCustomLaunchSettings()).toEqual(PLAY3D_CUSTOM_LAUNCH_DEFAULTS)
  })

  it('returns the defaults when stored value is malformed JSON', () => {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, '{not json')
    expect(loadPlay3dCustomLaunchSettings()).toEqual(PLAY3D_CUSTOM_LAUNCH_DEFAULTS)
  })

  it('falls back to default skyType when stored value is unknown', () => {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify({ skyType: 'banana' }))
    expect(loadPlay3dCustomLaunchSettings().skyType).toBe(PLAY3D_CUSTOM_LAUNCH_DEFAULTS.skyType)
  })

  it('falls back to default timerSeconds when stored value is non-positive', () => {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify({ timerSeconds: -5 }))
    expect(loadPlay3dCustomLaunchSettings().timerSeconds).toBe(PLAY3D_CUSTOM_LAUNCH_DEFAULTS.timerSeconds)
  })

  it('falls back to default enemyType when stored value is unknown', () => {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify({ enemyType: 'dragon' }))
    expect(loadPlay3dCustomLaunchSettings().enemyType).toBe(PLAY3D_CUSTOM_LAUNCH_DEFAULTS.enemyType)
  })

  it('falls back to default healthStyle when stored value is unknown', () => {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify({ healthStyle: 'shield' }))
    expect(loadPlay3dCustomLaunchSettings().healthStyle).toBe(PLAY3D_CUSTOM_LAUNCH_DEFAULTS.healthStyle)
  })

  it('round-trips a save+load', () => {
    const settings: Play3dCustomLaunchSettings = {
      skyType: 'sunset',
      wallType: 'cobblestone',
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
    savePlay3dCustomLaunchSettings(settings)
    expect(loadPlay3dCustomLaunchSettings()).toEqual(settings)
  })
})
