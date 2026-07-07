import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { GenerationFields, type GenerationFieldsValue } from '../../src/components/GameGenerationFields'
import { validateGenerationFields } from '../../src/utils/validation'

const VALID: GenerationFieldsValue = {
  rows: '10',
  cols: '10',
  minSolutionLength: '1',
  doorCount: '0',
  spareDoors: '0',
  spareKeys: '0',
  enemyCount: '0',
  healthCount: '0',
  treasureCount: '0',
}

describe('validateGenerationFields', () => {
  it('returns null for valid inputs', () => {
    expect(validateGenerationFields(VALID, null)).toBeNull()
    expect(validateGenerationFields(VALID, 1000)).toBeNull()
  })

  it('rejects rows / cols below 3', () => {
    expect(validateGenerationFields({ ...VALID, rows: '2' }, null)).toBe('Rows must be a whole number of 3 or more.')
    expect(validateGenerationFields({ ...VALID, cols: '2' }, null)).toBe('Columns must be a whole number of 3 or more.')
  })

  it('enforces the cell cap when one is set', () => {
    // 10×10 = 100 cells, cap 50.
    expect(validateGenerationFields(VALID, 50)).toBe('Total cells (rows × columns) cannot exceed 50.')
    // No cap → never rejected on size.
    expect(validateGenerationFields(VALID, null)).toBeNull()
  })

  it('requires a min solution length of at least 1', () => {
    expect(validateGenerationFields({ ...VALID, minSolutionLength: '0' }, null)).toBe(
      'Min Solution Length must be a whole number of 1 or more.',
    )
  })

  it('bounds each feature count to its cap', () => {
    expect(validateGenerationFields({ ...VALID, doorCount: '9' }, null)).toBe('Doors must be a whole number between 0 and 8.')
    expect(validateGenerationFields({ ...VALID, enemyCount: '9' }, null)).toBe('Enemies must be a whole number between 0 and 8.')
    expect(validateGenerationFields({ ...VALID, treasureCount: '13' }, null)).toBe(
      'Treasure must be a whole number between 0 and 12.',
    )
  })

  it('enforces the combined keys + doors budget', () => {
    // 2*8 + 1 = 17 > 16.
    expect(validateGenerationFields({ ...VALID, doorCount: '8', spareDoors: '1' }, null)).toBe(
      'Total keys + doors (17) exceeds the limit of 16. ' +
        'Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.',
    )
  })
})

describe('GenerationFields', () => {
  it('renders the size + count fields and no start/finish positions', () => {
    render(<GenerationFields value={VALID} onChange={vi.fn()} />)
    for (const label of ['Rows', 'Columns', 'Min Solution Length', 'Doors', 'Spare Doors', 'Spare Keys', 'Enemies', 'Health', 'Treasure']) {
      expect(screen.getByLabelText(label)).toBeInTheDocument()
    }
    expect(screen.queryByLabelText(/Start Row/)).toBeNull()
    expect(screen.queryByLabelText(/Finish Row/)).toBeNull()
  })

  it('reports a patch when a field changes', () => {
    const onChange = vi.fn()
    render(<GenerationFields value={VALID} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '15' } })
    expect(onChange).toHaveBeenCalledWith({ rows: '15' })
    fireEvent.change(screen.getByLabelText('Doors'), { target: { value: '5' } })
    expect(onChange).toHaveBeenCalledWith({ doorCount: '5' })
  })
})
