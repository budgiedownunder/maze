import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { AdvancedFields, type AdvancedFieldsValue } from '../../src/components/AdvancedFields'

const BASE: AdvancedFieldsValue = {
  maxHp: '3',
  enemyMovePeriodMs: '1500',
  minimapCellPx: '10',
  minimapRadius: '5',
  title: '',
  mode: '',
}

function renderFields(over: Partial<AdvancedFieldsValue> = {}, namePlaceholder = 'Tower') {
  const onChange = vi.fn<(patch: Partial<AdvancedFieldsValue>) => void>()
  render(<AdvancedFields value={{ ...BASE, ...over }} onChange={onChange} namePlaceholder={namePlaceholder} />)
  return { onChange }
}

const group = (name: string) => within(screen.getByRole('group', { name }))

describe('AdvancedFields', () => {
  it('groups the knobs into Health & Enemies / Minimap / Titles with the renamed labels', () => {
    renderFields()
    const he = group('Health & Enemies')
    expect(he.getByLabelText('Max HP')).toHaveValue(3)
    expect(he.getByLabelText('Enemy move period (ms)')).toHaveValue(1500)

    const minimap = group('Minimap')
    expect(minimap.getByLabelText('Cell size (px)')).toHaveValue(10)
    expect(minimap.getByLabelText('Radius (cells)')).toHaveValue(5)

    const titles = group('Titles')
    expect(titles.getByLabelText('Splash')).toHaveValue('')
    expect(titles.getByLabelText('Status-bar')).toHaveValue('')
  })

  it('does not carry the time limit (that lives on the General tab)', () => {
    renderFields()
    expect(screen.queryByLabelText('Time limit (seconds)')).toBeNull()
  })

  it('uses descriptive labels, not the raw config keys or old verbose ones', () => {
    renderFields()
    expect(screen.queryByLabelText('title')).toBeNull()
    expect(screen.queryByLabelText('minimapCellPx')).toBeNull()
    // The old pre-grouping labels are gone.
    expect(screen.queryByLabelText('Minimap cell size (px)')).toBeNull()
    expect(screen.queryByLabelText('Splash title')).toBeNull()
  })

  it('shows the game name as the placeholder for both overrides', () => {
    renderFields({}, 'Tower')
    expect(group('Titles').getByLabelText('Splash')).toHaveAttribute('placeholder', 'Tower')
    expect(group('Titles').getByLabelText('Status-bar')).toHaveAttribute('placeholder', 'Tower')
  })

  it('falls back to a generic placeholder when the name is blank', () => {
    renderFields({}, '   ')
    expect(group('Titles').getByLabelText('Splash')).toHaveAttribute('placeholder', "The game's name")
  })

  it('reports a numeric patch as a raw string', () => {
    const { onChange } = renderFields()
    fireEvent.change(group('Health & Enemies').getByLabelText('Max HP'), { target: { value: '7' } })
    expect(onChange).toHaveBeenCalledWith({ maxHp: '7' })
    fireEvent.change(group('Minimap').getByLabelText('Cell size (px)'), { target: { value: '14' } })
    expect(onChange).toHaveBeenCalledWith({ minimapCellPx: '14' })
  })

  it('reports the override text patches', () => {
    const { onChange } = renderFields()
    const titles = group('Titles')
    fireEvent.change(titles.getByLabelText('Splash'), { target: { value: 'Ascend!' } })
    expect(onChange).toHaveBeenCalledWith({ title: 'Ascend!' })
    fireEvent.change(titles.getByLabelText('Status-bar'), { target: { value: 'Endless' } })
    expect(onChange).toHaveBeenCalledWith({ mode: 'Endless' })
  })

  it('hints a minimum on the numeric inputs', () => {
    renderFields()
    expect(group('Health & Enemies').getByLabelText('Max HP')).toHaveAttribute('min', '1')
    expect(group('Minimap').getByLabelText('Radius (cells)')).toHaveAttribute('min', '1')
  })

  it('slots a supplied levelsGroup between Health & Enemies and Minimap', () => {
    const onChange = vi.fn()
    render(
      <AdvancedFields
        value={BASE}
        onChange={onChange}
        namePlaceholder="Tower"
        levelsGroup={<div role="group" aria-label="Levels" />}
      />,
    )
    // The accessible name of each group is either its aria-label (the slot) or
    // its heading text (the FieldGroups). Assert the order.
    const ordered = screen.getAllByRole('group').map(
      g => g.getAttribute('aria-label') ?? g.querySelector('.field-group-title')?.textContent,
    )
    expect(ordered).toEqual(['Health & Enemies', 'Levels', 'Minimap', 'Titles'])
  })

  it('renders nothing for a false levelsGroup (single-level)', () => {
    const onChange = vi.fn()
    render(<AdvancedFields value={BASE} onChange={onChange} namePlaceholder="Tower" levelsGroup={false} />)
    expect(screen.queryByRole('group', { name: 'Levels' })).toBeNull()
  })
})
