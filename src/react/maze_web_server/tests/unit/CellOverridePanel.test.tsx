import { describe, it, expect, vi } from 'vitest'
import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CellOverridePanel } from '../../src/components/CellOverridePanel'
import type { CellEntity } from '../../src/types/cellEntities'
import { MAZE_GAME_SETTINGS_DEFAULTS, type MazeGameSettings } from '../../src/utils/mazeGameSettings'

function setup(cellType: 'E' | 'H' | 'K' | 'D' | 'W', override?: CellEntity, gameSettings?: MazeGameSettings) {
  const onApply = vi.fn()
  const onClear = vi.fn()
  render(
    <CellOverridePanel
      cellType={cellType}
      row={1}
      col={2}
      override={override}
      onApply={onApply}
      onClear={onClear}
      gameSettings={gameSettings}
    />,
  )
  return { onApply, onClear }
}

describe('CellOverridePanel', () => {
  it('shows the type and 1-based cell coordinates in the title', () => {
    setup('E')
    expect(screen.getByText('Enemy [2,3]')).toBeInTheDocument()
  })

  it('renders the enemy fields with a Default option', () => {
    setup('E')
    expect(screen.getByRole('combobox', { name: 'Type' })).toBeInTheDocument()
    expect(screen.getByRole('spinbutton', { name: 'Damage' })).toBeInTheDocument()
    expect(screen.getByRole('spinbutton', { name: 'Move Interval (ms)' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'Default' })).toBeInTheDocument()
  })

  it('renders health fields for an H cell', () => {
    setup('H')
    expect(screen.getByRole('combobox', { name: 'Style' })).toBeInTheDocument()
    expect(screen.getByRole('spinbutton', { name: 'Heal Amount' })).toBeInTheDocument()
  })

  it('renders only the holder field for a K cell', () => {
    setup('K')
    expect(screen.getByRole('combobox', { name: 'Holder' })).toBeInTheDocument()
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
  })

  it('renders only the style field for a D cell', () => {
    setup('D')
    expect(screen.getByRole('combobox', { name: 'Style' })).toBeInTheDocument()
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
  })

  it('seeds the fields from an existing override', () => {
    setup('E', { type: 'E', enemyType: 'ghost', damage: 2 })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('ghost')
    expect(screen.getByRole('spinbutton', { name: 'Damage' })).toHaveValue(2)
  })

  it('applies an override live when a rig field changes', async () => {
    const { onApply } = setup('E')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'ghost')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'E', enemyType: 'ghost' })
  })

  it('applies a numeric override live as it is typed', async () => {
    const { onApply } = setup('H')
    await userEvent.type(screen.getByRole('spinbutton', { name: 'Heal Amount' }), '3')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'H', healAmount: 3 })
  })

  it('clears the override when the last set field reverts to default', async () => {
    const { onClear } = setup('K', { type: 'K', keyHolder: 'chest' })
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Holder' }), '')
    expect(onClear).toHaveBeenCalled()
  })

  it('Reset to defaults clears the override', async () => {
    const { onClear } = setup('D', { type: 'D', doorStyle: 'portcullis' })
    await userEvent.click(screen.getByRole('button', { name: 'Reset to defaults' }))
    expect(onClear).toHaveBeenCalled()
  })

  it('Reset clears the whole selection via onResetAll when provided', async () => {
    const onClear = vi.fn()
    const onResetAll = vi.fn()
    render(
      <CellOverridePanel
        cellType="W"
        row={1}
        col={2}
        override={{ type: 'W', wallType: 'lava' }}
        onApply={vi.fn()}
        onClear={onClear}
        onResetAll={onResetAll}
        selectionCount={4}
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Reset to defaults' }))
    expect(onResetAll).toHaveBeenCalledTimes(1)
    expect(onClear).not.toHaveBeenCalled()
  })

  it('shows no Apply-to-all link for a single cell', () => {
    setup('W')
    expect(screen.queryByRole('button', { name: /apply to all/i })).not.toBeInTheDocument()
  })

  it('shows an Apply-to-all link with the selection count for a multi-cell selection', async () => {
    const onApplyToAll = vi.fn()
    render(
      <CellOverridePanel
        cellType="W"
        row={1}
        col={2}
        override={undefined}
        onApply={vi.fn()}
        onClear={vi.fn()}
        onApplyToAll={onApplyToAll}
        selectionCount={6}
      />,
    )
    // The title flags the wider selection; the link names the cell count.
    expect(screen.getByRole('heading', { name: /\+5 more/ })).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Apply to all 6 cells' }))
    expect(onApplyToAll).toHaveBeenCalledTimes(1)
  })

  it('shows a sprite preview reflecting the selected enemy variant', () => {
    const { container } = render(
      <CellOverridePanel
        cellType="E" row={0} col={0}
        override={{ type: 'E', enemyType: 'ghost' }}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    expect(container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/ghost.svg')
  })

  it('updates the enemy preview live when the type changes', async () => {
    const { container } = render(
      <CellOverridePanel
        cellType="E" row={0} col={0} override={undefined}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    // Default selection previews the generic (goblin) sprite.
    expect(container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/enemy.svg')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'ghost')
    expect(container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/ghost.svg')
  })

  it('shows a sprite preview reflecting the selected health variant', () => {
    const { container } = render(
      <CellOverridePanel
        cellType="H" row={0} col={0}
        override={{ type: 'H', healthStyle: 'potion' }}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    expect(container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/potion.svg')
  })

  it('defaults a fresh wall cell to the "Default" (inherit) tier-1, with the texture picker shown for a solid maze default', () => {
    // No game settings ⇒ the effective maze default wall is solid (brick), so tier-1
    // "Default" still offers a per-cell texture override (its tier-2 inherit value '').
    setup('W')
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('default')
    expect(screen.getByRole('combobox', { name: 'Texture' })).toHaveValue('')
  })

  it('lists Default, Wall, and the special wall types in tier-1', () => {
    setup('W')
    const type = screen.getByRole('combobox', { name: 'Type' })
    for (const name of ['Default', 'Wall', 'Water', 'Lava', 'Iron Fence']) {
      expect(within(type).getByRole('option', { name })).toBeInTheDocument()
    }
  })

  it('selecting "Default" inherits (clears the override)', async () => {
    const { onClear } = setup('W', { type: 'W', wallType: 'lava' })
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'default')
    expect(onClear).toHaveBeenCalled()
  })

  it('selecting "Wall" forces a solid wall (first texture) and shows the texture picker', async () => {
    const { onApply } = setup('W')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'wall')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'W', wallType: 'brick' })
    expect(screen.getByRole('combobox', { name: 'Texture' })).toBeInTheDocument()
    // "Wall" forces a concrete texture — no inherit option in tier-2.
    expect(within(screen.getByRole('combobox', { name: 'Texture' })).queryByRole('option', { name: 'Default' }))
      .not.toBeInTheDocument()
  })

  it('applies a special wall type and hides the Texture select', async () => {
    const { onApply } = setup('W')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'water')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'W', wallType: 'water' })
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
  })

  it('under "Default" with a solid maze default, picking a texture overrides just this cell; back to Default inherits', async () => {
    const { onApply, onClear } = setup('W')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Texture' }), 'wood')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'W', wallType: 'wood' })
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Texture' }), '')
    expect(onClear).toHaveBeenCalled()
  })

  it('clears the override by switching tier-1 from Wall to Default', async () => {
    const { onClear } = setup('W', { type: 'W', wallType: 'brick' })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('wall')
    expect(screen.getByRole('combobox', { name: 'Texture' })).toHaveValue('brick')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'default')
    expect(onClear).toHaveBeenCalled()
  })

  it('hides the texture picker under "Default" when the maze default wall is special', () => {
    setup('W', undefined, { ...MAZE_GAME_SETTINGS_DEFAULTS, wallType: 'lava' })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('default')
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
  })

  it('shows the texture picker under "Wall" even when the maze default wall is special', async () => {
    setup('W', undefined, { ...MAZE_GAME_SETTINGS_DEFAULTS, wallType: 'lava' })
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'wall')
    expect(screen.getByRole('combobox', { name: 'Texture' })).toBeInTheDocument()
  })

  it('seeds a special wall type into the Type select (no Texture select)', () => {
    setup('W', { type: 'W', wallType: 'lava' })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('lava')
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
  })

  it('seeds a solid wall override as "Wall" + that texture', () => {
    setup('W', { type: 'W', wallType: 'dressed_stone' })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('wall')
    expect(screen.getByRole('combobox', { name: 'Texture' })).toHaveValue('dressed_stone')
  })

  it('shows a sprite preview reflecting the selected special wall type', () => {
    const { container } = render(
      <CellOverridePanel
        cellType="W" row={0} col={0}
        override={{ type: 'W', wallType: 'water' }}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    expect(container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/water.svg')
  })

  it('previews the maze default wall/enemy/health when no per-cell override is set', () => {
    const lavaMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, wallType: 'lava' as const }
    const ghostMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, enemyType: 'ghost' as const }
    const potionMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, healthStyle: 'potion' as const }

    const wall = render(
      <CellOverridePanel cellType="W" row={0} col={0} override={undefined} onApply={vi.fn()} onClear={vi.fn()} gameSettings={lavaMaze} />,
    )
    expect(wall.container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/lava.svg')
    wall.unmount()

    const enemy = render(
      <CellOverridePanel cellType="E" row={0} col={0} override={undefined} onApply={vi.fn()} onClear={vi.fn()} gameSettings={ghostMaze} />,
    )
    expect(enemy.container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/ghost.svg')
    enemy.unmount()

    const health = render(
      <CellOverridePanel cellType="H" row={0} col={0} override={undefined} onApply={vi.fn()} onClear={vi.fn()} gameSettings={potionMaze} />,
    )
    expect(health.container.querySelector('.cell-override-preview')).toHaveAttribute('src', '/images/maze/potion.svg')
    health.unmount()
  })

  it('re-seeds the fields to defaults when the override is cleared externally', () => {
    // Mirrors the toolbar re-stamping the same cell type: the override is dropped
    // while the cell stays selected, so the panel is not remounted.
    const { rerender } = render(
      <CellOverridePanel
        cellType="E" row={1} col={2}
        override={{ type: 'E', enemyType: 'ghost', damage: 2 }}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('ghost')
    expect(screen.getByRole('spinbutton', { name: 'Damage' })).toHaveValue(2)

    rerender(
      <CellOverridePanel
        cellType="E" row={1} col={2}
        override={undefined}
        onApply={vi.fn()} onClear={vi.fn()}
      />,
    )
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('')
    expect(screen.getByRole('spinbutton', { name: 'Damage' })).toHaveValue(null)
  })
})
