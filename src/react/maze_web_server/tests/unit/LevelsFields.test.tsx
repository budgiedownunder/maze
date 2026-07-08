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
  it('renders the Finish Type select and the three per-level toggles', () => {
    renderFields()
    expect(screen.getByLabelText('Finish Type')).toHaveValue('ladder')
    for (const name of ['Reset item bag each level', 'Randomise perimeter each level', 'Hide cleared-level enemies']) {
      expect(screen.getByRole('checkbox', { name })).toBeInTheDocument()
    }
  })

  it('does not carry the level count (that lives on the General tab)', () => {
    renderFields()
    expect(screen.queryByLabelText('Levels')).toBeNull()
  })

  it('does not carry the progression fields (those live on the Layout tab)', () => {
    renderFields()
    expect(screen.queryByLabelText('Difficulty Change')).toBeNull()
    expect(screen.queryByLabelText('Alignment')).toBeNull()
    expect(screen.queryByRole('checkbox', { name: 'Taper' })).toBeNull()
  })

  it('does not carry the final-level override (that lives on the Scene tab)', () => {
    renderFields()
    expect(screen.queryByRole('checkbox', { name: 'Override final level appearance' })).toBeNull()
  })
})

describe('LevelsFields — patches', () => {
  it('reports the finish-type selection with its wire value', () => {
    const { onChange } = renderFields()
    fireEvent.change(screen.getByLabelText('Finish Type'), { target: { value: 'random' } })
    expect(onChange).toHaveBeenCalledWith({ finishType: 'random' })
  })

  it('reports a toggle change', async () => {
    const { onChange } = renderFields()
    await userEvent.click(screen.getByRole('checkbox', { name: 'Reset item bag each level' }))
    expect(onChange).toHaveBeenCalledWith({ resetBag: false })
  })
})
