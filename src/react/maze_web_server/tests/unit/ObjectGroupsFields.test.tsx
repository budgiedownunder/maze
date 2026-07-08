import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { ObjectGroupsFields } from '../../src/components/ObjectGroupsFields'
import type { MazeGenerationFieldsValue } from '../../src/components/MazeGenerationFields'
import type { ObjectsFieldsValue } from '../../src/components/GameSettingsFields'

const COUNTS: MazeGenerationFieldsValue = {
  rows: '8', cols: '8', minSolutionLength: '1',
  doorCount: '1', spareDoors: '2', spareKeys: '3',
  enemyCount: '4', healthCount: '5', treasureCount: '6',
}

const STYLES: ObjectsFieldsValue = {
  doorStyle: 'swing',
  keyHolder: 'pedestal',
  enemyType: 'goblin',
  healthStyle: 'heart',
}

function renderGroups() {
  const onCountsChange = vi.fn<(p: Partial<MazeGenerationFieldsValue>) => void>()
  const onStylesChange = vi.fn<(p: Partial<ObjectsFieldsValue>) => void>()
  render(
    <ObjectGroupsFields
      counts={COUNTS}
      onCountsChange={onCountsChange}
      styles={STYLES}
      onStylesChange={onStylesChange}
    />,
  )
  return { onCountsChange, onStylesChange }
}

const group = (name: string) => within(screen.getByRole('group', { name }))

describe('ObjectGroupsFields — layout', () => {
  it('renders the five groups', () => {
    renderGroups()
    for (const name of ['Doors', 'Keys', 'Enemies', 'Health', 'Treasure']) {
      expect(screen.getByRole('group', { name })).toBeInTheDocument()
    }
  })

  it('lays out the Doors group as Count / Spares / Style', () => {
    renderGroups()
    const g = group('Doors')
    expect(g.getByLabelText('Count')).toHaveValue(1)
    expect(g.getByLabelText('Spares')).toHaveValue(2)
    expect(g.getByLabelText('Style')).toHaveValue('swing')
  })

  it('lays out the Keys group as Spares / Holder (no count)', () => {
    renderGroups()
    const g = group('Keys')
    expect(g.getByLabelText('Spares')).toHaveValue(3)
    expect(g.getByLabelText('Holder')).toHaveValue('pedestal')
    expect(g.queryByLabelText('Count')).toBeNull()
  })

  it('lays out the Enemies group as Count / Type', () => {
    renderGroups()
    const g = group('Enemies')
    expect(g.getByLabelText('Count')).toHaveValue(4)
    expect(g.getByLabelText('Type')).toHaveValue('goblin')
  })

  it('lays out the Health group as Count / Type', () => {
    renderGroups()
    const g = group('Health')
    expect(g.getByLabelText('Count')).toHaveValue(5)
    expect(g.getByLabelText('Type')).toHaveValue('heart')
  })

  it('lays out the Treasure group as Count only', () => {
    renderGroups()
    const g = group('Treasure')
    expect(g.getByLabelText('Count')).toHaveValue(6)
    expect(g.queryByLabelText('Style')).toBeNull()
    expect(g.queryByLabelText('Type')).toBeNull()
  })

  it('bounds each count with its per-feature max', () => {
    renderGroups()
    expect(group('Doors').getByLabelText('Count')).toHaveAttribute('max', '8')
    expect(group('Doors').getByLabelText('Spares')).toHaveAttribute('max', '8')
    expect(group('Keys').getByLabelText('Spares')).toHaveAttribute('max', '16')
    expect(group('Treasure').getByLabelText('Count')).toHaveAttribute('max', '12')
  })
})

describe('ObjectGroupsFields — patches', () => {
  it('routes a count change to the generation slice, keyed correctly per group', () => {
    const { onCountsChange, onStylesChange } = renderGroups()
    fireEvent.change(group('Doors').getByLabelText('Count'), { target: { value: '7' } })
    expect(onCountsChange).toHaveBeenCalledWith({ doorCount: '7' })
    fireEvent.change(group('Keys').getByLabelText('Spares'), { target: { value: '9' } })
    expect(onCountsChange).toHaveBeenCalledWith({ spareKeys: '9' })
    fireEvent.change(group('Treasure').getByLabelText('Count'), { target: { value: '2' } })
    expect(onCountsChange).toHaveBeenCalledWith({ treasureCount: '2' })
    expect(onStylesChange).not.toHaveBeenCalled()
  })

  it('routes a style change to the objects slice', () => {
    const { onCountsChange, onStylesChange } = renderGroups()
    fireEvent.change(group('Doors').getByLabelText('Style'), { target: { value: 'slide' } })
    expect(onStylesChange).toHaveBeenCalledWith({ doorStyle: 'slide' })
    fireEvent.change(group('Enemies').getByLabelText('Type'), { target: { value: 'ghost' } })
    expect(onStylesChange).toHaveBeenCalledWith({ enemyType: 'ghost' })
    expect(onCountsChange).not.toHaveBeenCalled()
  })
})
