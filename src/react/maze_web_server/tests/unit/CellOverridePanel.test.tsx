import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CellOverridePanel } from '../../src/components/CellOverridePanel'
import type { CellEntity } from '../../src/types/cellEntities'

function setup(cellType: 'E' | 'H' | 'K' | 'D' | 'W', override?: CellEntity) {
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

  it('renders the wall Type + Texture selects, with Texture shown under "Wall"', () => {
    setup('W')
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('wall')
    expect(screen.getByRole('combobox', { name: 'Texture' })).toHaveValue('')
  })

  it('applies a special wall type and hides the Texture select', async () => {
    const { onApply } = setup('W')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Type' }), 'water')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'W', wallType: 'water' })
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
  })

  it('applies a solid texture chosen under "Wall"', async () => {
    const { onApply } = setup('W')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Texture' }), 'brick')
    expect(onApply).toHaveBeenLastCalledWith({ type: 'W', wallType: 'brick' })
  })

  it('clears the override when Texture returns to Default under "Wall"', async () => {
    const { onClear } = setup('W', { type: 'W', wallType: 'brick' })
    expect(screen.getByRole('combobox', { name: 'Texture' })).toHaveValue('brick')
    await userEvent.selectOptions(screen.getByRole('combobox', { name: 'Texture' }), '')
    expect(onClear).toHaveBeenCalled()
  })

  it('seeds a special wall type into the Type select (no Texture select)', () => {
    setup('W', { type: 'W', wallType: 'lava' })
    expect(screen.getByRole('combobox', { name: 'Type' })).toHaveValue('lava')
    expect(screen.queryByRole('combobox', { name: 'Texture' })).not.toBeInTheDocument()
  })

  it('seeds a solid wall texture into the Texture select under "Wall"', () => {
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
