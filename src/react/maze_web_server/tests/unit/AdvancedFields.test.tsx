import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
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

describe('AdvancedFields', () => {
  it('renders the four numeric knobs and the two override text fields', () => {
    renderFields()
    expect(screen.getByLabelText('Max HP')).toHaveValue(3)
    expect(screen.getByLabelText('Enemy move period (ms)')).toHaveValue(1500)
    expect(screen.getByLabelText('Minimap cell size (px)')).toHaveValue(10)
    expect(screen.getByLabelText('Minimap radius (cells)')).toHaveValue(5)
    expect(screen.getByLabelText('Splash title')).toHaveValue('')
    expect(screen.getByLabelText('Status-bar label')).toHaveValue('')
  })

  it('does not carry the time limit (that lives on the General tab)', () => {
    renderFields()
    expect(screen.queryByLabelText('Time limit (seconds)')).toBeNull()
  })

  it('uses descriptive labels, not the raw config keys', () => {
    renderFields()
    expect(screen.queryByLabelText('title')).toBeNull()
    expect(screen.queryByLabelText('mode')).toBeNull()
    expect(screen.queryByLabelText('minimapCellPx')).toBeNull()
  })

  it('shows the game name as the placeholder for both overrides', () => {
    renderFields({}, 'Tower')
    expect(screen.getByLabelText('Splash title')).toHaveAttribute('placeholder', 'Tower')
    expect(screen.getByLabelText('Status-bar label')).toHaveAttribute('placeholder', 'Tower')
  })

  it('falls back to a generic placeholder when the name is blank', () => {
    renderFields({}, '   ')
    expect(screen.getByLabelText('Splash title')).toHaveAttribute('placeholder', "The game's name")
  })

  it('reports a numeric patch as a raw string', () => {
    const { onChange } = renderFields()
    fireEvent.change(screen.getByLabelText('Max HP'), { target: { value: '7' } })
    expect(onChange).toHaveBeenCalledWith({ maxHp: '7' })
  })

  it('reports the override text patches', () => {
    const { onChange } = renderFields()
    fireEvent.change(screen.getByLabelText('Splash title'), { target: { value: 'Ascend!' } })
    expect(onChange).toHaveBeenCalledWith({ title: 'Ascend!' })
    fireEvent.change(screen.getByLabelText('Status-bar label'), { target: { value: 'Endless' } })
    expect(onChange).toHaveBeenCalledWith({ mode: 'Endless' })
  })

  it('hints a minimum on the numeric inputs', () => {
    renderFields()
    expect(screen.getByLabelText('Max HP')).toHaveAttribute('min', '1')
    expect(screen.getByLabelText('Minimap radius (cells)')).toHaveAttribute('min', '1')
  })
})
