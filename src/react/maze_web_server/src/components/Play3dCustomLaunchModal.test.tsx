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
    expect(labels).toEqual(['Night', 'Sunrise', 'Day', 'Sunset'])
    expect(values).toEqual(['night', 'sunrise', 'day', 'sunset'])
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
    await userEvent.click(screen.getByRole('button', { name: /play/i }))
    expect(onPlay).toHaveBeenCalledTimes(1)
    const settings = onPlay.mock.calls[0][0] as Play3dCustomLaunchSettings
    expect(settings.skyType).toBe('sunset')
    expect(settings.timerSeconds).toBe(120)
  })

  it('calls onCancel when the Cancel button is clicked', async () => {
    const onCancel = vi.fn()
    render(<Play3dCustomLaunchModal mazeName="My Maze" onCancel={onCancel} onPlay={() => {}} />)
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })

  it('pre-fills from localStorage when settings have been saved before', () => {
    const stored: Play3dCustomLaunchSettings = {
      skyType: 'day',
      wallType: 'wood',
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
    expect(screen.getByLabelText(/varied wall tints/i)).toBeChecked()
    expect(screen.getByLabelText(/dead-end objects/i)).not.toBeChecked()
    expect(screen.getByLabelText(/sparse wall decorations/i)).not.toBeChecked()
    expect(screen.getByLabelText(/floor junction markers/i)).toBeChecked()
    expect((screen.getByLabelText(/time limit/i) as HTMLInputElement).value).toBe('180')
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

  it('round-trips a save+load', () => {
    const settings: Play3dCustomLaunchSettings = {
      skyType: 'sunset',
      wallType: 'cobblestone',
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
