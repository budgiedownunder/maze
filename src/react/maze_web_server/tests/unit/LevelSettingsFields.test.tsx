import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LevelSettingsFields } from '../../src/components/LevelSettingsFields'
import { DEFINITION_DEFAULTS, type DefinitionLevelsFormValue } from '../../src/utils/definitionConfig'

const BASE: DefinitionLevelsFormValue = DEFINITION_DEFAULTS.levels

function renderFields(over: Partial<DefinitionLevelsFormValue> = {}) {
  const onChange = vi.fn<(patch: Partial<DefinitionLevelsFormValue>) => void>()
  render(<LevelSettingsFields value={{ ...BASE, ...over }} onChange={onChange} />)
  return { onChange }
}

describe('LevelSettingsFields', () => {
  it('renders the three per-level toggles', () => {
    renderFields()
    for (const name of ['Reset item bag each level', 'Randomise perimeter each level', 'Hide cleared-level enemies']) {
      expect(screen.getByRole('checkbox', { name })).toBeInTheDocument()
    }
  })

  it('does not carry the finish rig or progression fields (those live elsewhere)', () => {
    renderFields()
    expect(screen.queryByLabelText('Finish Cell')).toBeNull()
    expect(screen.queryByLabelText('Difficulty Change')).toBeNull()
  })

  it('reports each toggle change against its config field', async () => {
    const { onChange } = renderFields()
    await userEvent.click(screen.getByRole('checkbox', { name: 'Reset item bag each level' }))
    expect(onChange).toHaveBeenCalledWith({ resetBag: false })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Randomise perimeter each level' }))
    expect(onChange).toHaveBeenCalledWith({ perimeterRandom: true })
    await userEvent.click(screen.getByRole('checkbox', { name: 'Hide cleared-level enemies' }))
    expect(onChange).toHaveBeenCalledWith({ hideCompletedEnemies: true })
  })
})
