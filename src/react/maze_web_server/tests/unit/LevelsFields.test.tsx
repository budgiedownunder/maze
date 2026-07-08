import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LevelsFields } from '../../src/components/LevelsFields'
import { DEFINITION_DEFAULTS, type DefinitionLevelsFormValue } from '../../src/utils/definitionConfig'

const BASE: DefinitionLevelsFormValue = DEFINITION_DEFAULTS.levels

function renderFields(over: Partial<DefinitionLevelsFormValue> = {}) {
  const onChange = vi.fn<(patch: Partial<DefinitionLevelsFormValue>) => void>()
  render(<LevelsFields value={{ ...BASE, ...over }} onChange={onChange} />)
  return { onChange }
}

describe('LevelsFields — rendering', () => {
  it('renders the three enum selects and the four toggles', () => {
    renderFields()
    expect(screen.getByLabelText('Finish Type')).toHaveValue('ladder')
    expect(screen.getByLabelText('Difficulty Change')).toHaveValue('easier')
    expect(screen.getByLabelText('Level Alignment')).toHaveValue('edge')
    for (const name of ['Reset item bag each level', 'Taper upper levels', 'Randomise perimeter each level', 'Hide cleared-level enemies']) {
      expect(screen.getByRole('checkbox', { name })).toBeInTheDocument()
    }
  })

  it('does not carry the level count (that lives on the General tab)', () => {
    renderFields()
    expect(screen.queryByLabelText('Levels')).toBeNull()
  })

  it('hides the final-level override controls until the override is enabled', () => {
    renderFields()
    expect(screen.queryByLabelText('Final Level Sky')).toBeNull()
    expect(screen.queryByLabelText('Final Level Perimeter Walls')).toBeNull()
  })
})

describe('LevelsFields — patches', () => {
  it('reports enum selections with their wire values', () => {
    const { onChange } = renderFields()
    fireEvent.change(screen.getByLabelText('Finish Type'), { target: { value: 'random' } })
    expect(onChange).toHaveBeenCalledWith({ finishType: 'random' })
    fireEvent.change(screen.getByLabelText('Level Alignment'), { target: { value: 'random_base' } })
    expect(onChange).toHaveBeenCalledWith({ alignment: 'random_base' })
  })

  it('reports a toggle change', async () => {
    const { onChange } = renderFields()
    await userEvent.click(screen.getByRole('checkbox', { name: 'Taper upper levels' }))
    expect(onChange).toHaveBeenCalledWith({ taper: true })
  })
})

describe('LevelsFields — final-level override', () => {
  it('enabling the override seeds an empty (all-inherit) top object', async () => {
    const { onChange } = renderFields({ top: null })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Override final level appearance' }))
    expect(onChange).toHaveBeenCalledWith({ top: {} })
  })

  it('disabling the override clears top back to null', async () => {
    const { onChange } = renderFields({ top: { skyType: 'day' } })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Override final level appearance' }))
    expect(onChange).toHaveBeenCalledWith({ top: null })
  })

  it('shows the sky + perimeter selects when the override is on, defaulting to Inherit', () => {
    renderFields({ top: {} })
    expect(screen.getByLabelText('Final Level Sky')).toHaveValue('')
    expect(screen.getByLabelText('Final Level Perimeter Walls')).toHaveValue('inherit')
  })

  it('sets a sky override, and clears it back to inherit', () => {
    const onChange = vi.fn<(patch: Partial<DefinitionLevelsFormValue>) => void>()
    const { rerender } = render(<LevelsFields value={{ ...BASE, top: {} }} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Final Level Sky'), { target: { value: 'night' } })
    expect(onChange).toHaveBeenCalledWith({ top: { skyType: 'night' } })

    onChange.mockClear()
    rerender(<LevelsFields value={{ ...BASE, top: { skyType: 'night' } }} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Final Level Sky'), { target: { value: '' } })
    // Clearing to Inherit drops the key, leaving an empty override.
    expect(onChange).toHaveBeenCalledWith({ top: {} })
  })

  it('maps the perimeter select to a tri-state (inherit / walled / open)', () => {
    const { onChange } = renderFields({ top: {} })
    const perim = screen.getByLabelText('Final Level Perimeter Walls')
    fireEvent.change(perim, { target: { value: 'walled' } })
    expect(onChange).toHaveBeenCalledWith({ top: { perimeterWalls: true } })
    fireEvent.change(perim, { target: { value: 'open' } })
    expect(onChange).toHaveBeenCalledWith({ top: { perimeterWalls: false } })
  })

  it('preserves the sky override when only the perimeter changes', () => {
    const { onChange } = renderFields({ top: { skyType: 'day' } })
    fireEvent.change(screen.getByLabelText('Final Level Perimeter Walls'), { target: { value: 'open' } })
    expect(onChange).toHaveBeenCalledWith({ top: { skyType: 'day', perimeterWalls: false } })
  })
})
