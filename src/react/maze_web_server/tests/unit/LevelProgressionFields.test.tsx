import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LevelProgressionFields } from '../../src/components/LevelProgressionFields'
import { DEFINITION_DEFAULTS, type DefinitionLevelsFormValue } from '../../src/utils/definitionConfig'

const BASE: DefinitionLevelsFormValue = DEFINITION_DEFAULTS.levels

function renderFields(over: Partial<DefinitionLevelsFormValue> = {}) {
  const onChange = vi.fn<(patch: Partial<DefinitionLevelsFormValue>) => void>()
  render(<LevelProgressionFields value={{ ...BASE, ...over }} onChange={onChange} />)
  return { onChange }
}

describe('LevelProgressionFields', () => {
  it('renders Difficulty Change, Alignment (renamed), and Taper (renamed)', () => {
    renderFields()
    expect(screen.getByLabelText('Difficulty Change')).toHaveValue('easier')
    expect(screen.getByLabelText('Alignment')).toHaveValue('edge')
    expect(screen.getByRole('checkbox', { name: 'Taper' })).toBeInTheDocument()
    // The old verbose labels are gone.
    expect(screen.queryByLabelText('Level Alignment')).toBeNull()
    expect(screen.queryByRole('checkbox', { name: 'Taper upper levels' })).toBeNull()
  })

  it('reports the enum selections with their wire values', () => {
    const { onChange } = renderFields()
    fireEvent.change(screen.getByLabelText('Difficulty Change'), { target: { value: 'harder' } })
    expect(onChange).toHaveBeenCalledWith({ difficultyChange: 'harder' })
    fireEvent.change(screen.getByLabelText('Alignment'), { target: { value: 'random_base' } })
    expect(onChange).toHaveBeenCalledWith({ alignment: 'random_base' })
  })

  it('reports the taper toggle', async () => {
    const { onChange } = renderFields()
    await userEvent.click(screen.getByRole('checkbox', { name: 'Taper' }))
    expect(onChange).toHaveBeenCalledWith({ taper: true })
  })
})
