import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CellOverridePanel } from '../../src/components/CellOverridePanel'
import type { CellEntity } from '../../src/types/cellEntities'

function setup(cellType: 'E' | 'H' | 'K' | 'D', override?: CellEntity) {
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
