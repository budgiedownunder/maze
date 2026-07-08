import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GameSceneFields } from '../../src/components/GameSceneFields'
import type { SceneFieldsValue, DecorFieldsValue } from '../../src/components/GameSettingsFields'
import type { DefinitionTopLevelConfig } from '../../src/utils/definitionConfig'

const SCENE: SceneFieldsValue = {
  skyType: 'day',
  wallType: 'brick',
  perimeterWalls: false,
  wallTint: false,
  wallMaterialVariation: false,
}
const DECOR: DecorFieldsValue = { deadEndObjects: false, wallDecorations: false, floorAccents: false }

interface Over {
  scene?: Partial<SceneFieldsValue>
  decor?: Partial<DecorFieldsValue>
  top?: DefinitionTopLevelConfig | null
  multiLevel?: boolean
}

function renderScene(over: Over = {}) {
  const onSceneChange = vi.fn<(p: Partial<SceneFieldsValue>) => void>()
  const onDecorChange = vi.fn<(p: Partial<DecorFieldsValue>) => void>()
  const onTopChange = vi.fn<(t: DefinitionTopLevelConfig | null) => void>()
  render(
    <GameSceneFields
      scene={{ ...SCENE, ...over.scene }}
      onSceneChange={onSceneChange}
      decor={{ ...DECOR, ...over.decor }}
      onDecorChange={onDecorChange}
      top={over.top ?? null}
      onTopChange={onTopChange}
      multiLevel={over.multiLevel ?? false}
    />,
  )
  return { onSceneChange, onDecorChange, onTopChange }
}

const group = (name: string) => within(screen.getByRole('group', { name }))

describe('GameSceneFields — layout', () => {
  it('renders the Sky / Walls / Decoration groups with the renamed controls', () => {
    renderScene()
    expect(group('Sky').getByLabelText('Sky')).toBeVisible()

    const walls = group('Walls')
    expect(walls.getByRole('checkbox', { name: 'Quadrant wall types' })).toBeVisible()
    expect(walls.getByLabelText('Texture')).toBeVisible()
    expect(walls.getByRole('checkbox', { name: 'Perimeter' })).toBeVisible()

    const decoration = group('Decoration')
    for (const name of ['Wall tints', 'Wall objects', 'Dead-end objects', 'Floor junctions']) {
      expect(decoration.getByRole('checkbox', { name })).toBeVisible()
    }
  })

  it('has no Final Level controls for a single-level game (and no override toggle)', () => {
    renderScene({ multiLevel: false, top: {} })
    expect(screen.queryByRole('checkbox', { name: 'Override final' })).toBeNull()
    expect(screen.queryByLabelText('Final Level')).toBeNull()
  })

  it('shows a Final Level select in BOTH the Sky and Walls groups for a multi-level game, defaulting to Inherit', () => {
    // No override toggle any more — the selects appear outright, defaulting to Inherit.
    renderScene({ multiLevel: true, top: null })
    expect(screen.queryByRole('checkbox', { name: 'Override final' })).toBeNull()
    // Sky group's Final Level is the sky override (Inherit = '').
    expect(group('Sky').getByLabelText('Final Level')).toHaveValue('')
    // Walls group's Final Level is the perimeter override (tri-state, Inherit).
    expect(group('Walls').getByLabelText('Final Level')).toHaveValue('inherit')
  })
})

describe('GameSceneFields — interlocks', () => {
  it('quadrant wall types disables the Texture select and the Wall tints toggle', () => {
    renderScene({ scene: { wallMaterialVariation: true } })
    expect(group('Walls').getByLabelText('Texture')).toBeDisabled()
    expect(group('Decoration').getByRole('checkbox', { name: 'Wall tints' })).toBeDisabled()
  })

  it('an enclosed sky forces Perimeter on and disables it', () => {
    renderScene({ scene: { skyType: 'dungeon' } })
    const perimeter = group('Walls').getByRole('checkbox', { name: 'Perimeter' })
    expect(perimeter).toBeChecked()
    expect(perimeter).toBeDisabled()
  })
})

describe('GameSceneFields — patches', () => {
  it('routes scene changes to the scene slice', () => {
    const { onSceneChange } = renderScene()
    fireEvent.change(group('Sky').getByLabelText('Sky'), { target: { value: 'night' } })
    expect(onSceneChange).toHaveBeenCalledWith({ skyType: 'night' })
    fireEvent.change(group('Walls').getByLabelText('Texture'), { target: { value: 'wood' } })
    expect(onSceneChange).toHaveBeenCalledWith({ wallType: 'wood' })
  })

  it('routes decor changes to the decor slice, keyed by the original config field', async () => {
    const { onDecorChange, onSceneChange } = renderScene()
    const decoration = group('Decoration')
    await userEvent.click(decoration.getByRole('checkbox', { name: 'Wall objects' }))
    expect(onDecorChange).toHaveBeenCalledWith({ wallDecorations: true })
    await userEvent.click(decoration.getByRole('checkbox', { name: 'Floor junctions' }))
    expect(onDecorChange).toHaveBeenCalledWith({ floorAccents: true })
    expect(onSceneChange).not.toHaveBeenCalled()
  })
})

describe('GameSceneFields — final-level override', () => {
  it('sets the Sky-group Final Level as the sky override (from a null top)', () => {
    const { onTopChange } = renderScene({ multiLevel: true, top: null })
    fireEvent.change(group('Sky').getByLabelText('Final Level'), { target: { value: 'night' } })
    expect(onTopChange).toHaveBeenCalledWith({ skyType: 'night' })
  })

  it('clearing the Sky-group Final Level back to Inherit drops the key', () => {
    const { onTopChange } = renderScene({ multiLevel: true, top: { skyType: 'night' } })
    fireEvent.change(group('Sky').getByLabelText('Final Level'), { target: { value: '' } })
    expect(onTopChange).toHaveBeenCalledWith({})
  })

  it('sets the Walls-group Final Level as the tri-state perimeter override', () => {
    const { onTopChange } = renderScene({ multiLevel: true, top: { skyType: 'day' } })
    const perim = group('Walls').getByLabelText('Final Level')
    fireEvent.change(perim, { target: { value: 'walled' } })
    // The sky override is preserved when only the perimeter changes.
    expect(onTopChange).toHaveBeenCalledWith({ skyType: 'day', perimeterWalls: true })
    fireEvent.change(perim, { target: { value: 'open' } })
    expect(onTopChange).toHaveBeenCalledWith({ skyType: 'day', perimeterWalls: false })
  })
})
