import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { FinalLevelOverrideFields } from '../../src/components/FinalLevelOverrideFields'
import type { DefinitionTopLevelConfig } from '../../src/utils/definitionConfig'

function renderFields(value: DefinitionTopLevelConfig | null) {
  const onChange = vi.fn<(top: DefinitionTopLevelConfig | null) => void>()
  render(<FinalLevelOverrideFields value={value} onChange={onChange} />)
  return { onChange }
}

describe('FinalLevelOverrideFields', () => {
  it('shows only the toggle (off) when there is no override', () => {
    renderFields(null)
    expect(screen.getByRole('checkbox', { name: 'Override final level appearance' })).not.toBeChecked()
    expect(screen.queryByLabelText('Final Level Sky')).toBeNull()
    expect(screen.queryByLabelText('Final Level Perimeter Walls')).toBeNull()
  })

  it('enabling the override seeds an empty (all-inherit) top object', async () => {
    const { onChange } = renderFields(null)
    await userEvent.click(screen.getByRole('checkbox', { name: 'Override final level appearance' }))
    expect(onChange).toHaveBeenCalledWith({})
  })

  it('disabling the override clears it back to null', async () => {
    const { onChange } = renderFields({ skyType: 'day' })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Override final level appearance' }))
    expect(onChange).toHaveBeenCalledWith(null)
  })

  it('shows the sky + perimeter selects when the override is on, defaulting to Inherit', () => {
    renderFields({})
    expect(screen.getByRole('checkbox', { name: 'Override final level appearance' })).toBeChecked()
    expect(screen.getByLabelText('Final Level Sky')).toHaveValue('')
    expect(screen.getByLabelText('Final Level Perimeter Walls')).toHaveValue('inherit')
  })

  it('sets a sky override, and clears it back to inherit', () => {
    const onChange = vi.fn<(top: DefinitionTopLevelConfig | null) => void>()
    const { rerender } = render(<FinalLevelOverrideFields value={{}} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Final Level Sky'), { target: { value: 'night' } })
    expect(onChange).toHaveBeenCalledWith({ skyType: 'night' })

    onChange.mockClear()
    rerender(<FinalLevelOverrideFields value={{ skyType: 'night' }} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Final Level Sky'), { target: { value: '' } })
    // Clearing to Inherit drops the key, leaving an empty override.
    expect(onChange).toHaveBeenCalledWith({})
  })

  it('maps the perimeter select to a tri-state (inherit / walled / open)', () => {
    const { onChange } = renderFields({})
    const perim = screen.getByLabelText('Final Level Perimeter Walls')
    fireEvent.change(perim, { target: { value: 'walled' } })
    expect(onChange).toHaveBeenCalledWith({ perimeterWalls: true })
    fireEvent.change(perim, { target: { value: 'open' } })
    expect(onChange).toHaveBeenCalledWith({ perimeterWalls: false })
  })

  it('preserves the sky override when only the perimeter changes', () => {
    const { onChange } = renderFields({ skyType: 'day' })
    fireEvent.change(screen.getByLabelText('Final Level Perimeter Walls'), { target: { value: 'open' } })
    expect(onChange).toHaveBeenCalledWith({ skyType: 'day', perimeterWalls: false })
  })
})
