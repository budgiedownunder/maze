import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MazeGenerationFields, type MazeGenerationFieldsValue } from '../../src/components/MazeGenerationFields'
import { validateMazeGenerationFields } from '../../src/utils/validation'

const VALID: MazeGenerationFieldsValue = {
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

describe('validateMazeGenerationFields (game subject — no positions)', () => {
  it('returns null for valid inputs', () => {
    expect(validateMazeGenerationFields(VALID, null, 'game')).toBeNull()
    expect(validateMazeGenerationFields(VALID, 1000, 'game')).toBeNull()
  })

  it('rejects rows / cols below 3', () => {
    expect(validateMazeGenerationFields({ ...VALID, rows: '2' }, null, 'game')).toBe('Rows must be a whole number of 3 or more.')
    expect(validateMazeGenerationFields({ ...VALID, cols: '2' }, null, 'game')).toBe('Columns must be a whole number of 3 or more.')
  })

  it('enforces the cell cap when one is set', () => {
    // 10×10 = 100 cells, cap 50.
    expect(validateMazeGenerationFields(VALID, 50, 'game')).toBe('Total cells (rows × columns) cannot exceed 50.')
    // No cap → never rejected on size.
    expect(validateMazeGenerationFields(VALID, null, 'game')).toBeNull()
  })

  it('allows a min solution length of 0 (no minimum) but rejects a negative one', () => {
    expect(validateMazeGenerationFields({ ...VALID, minSolutionLength: '0' }, null, 'game')).toBeNull()
    expect(validateMazeGenerationFields({ ...VALID, minSolutionLength: '-1' }, null, 'game')).toBe(
      'Min Solution Length must be a whole number of 0 or more.',
    )
    expect(validateMazeGenerationFields({ ...VALID, minSolutionLength: '' }, null, 'game')).toBe(
      'Min Solution Length must be a whole number of 0 or more.',
    )
  })

  it('bounds each feature count to its cap', () => {
    expect(validateMazeGenerationFields({ ...VALID, doorCount: '9' }, null, 'game')).toBe('Doors must be a whole number between 0 and 8.')
    expect(validateMazeGenerationFields({ ...VALID, enemyCount: '9' }, null, 'game')).toBe('Enemies must be a whole number between 0 and 8.')
    expect(validateMazeGenerationFields({ ...VALID, treasureCount: '13' }, null, 'game')).toBe(
      'Treasure must be a whole number between 0 and 12.',
    )
  })

  it('bounds spare doors to the door cap (8) and spare keys to the whole budget (16)', () => {
    expect(validateMazeGenerationFields({ ...VALID, spareDoors: '9' }, null, 'game')).toBe(
      'Spare Doors must be a whole number between 0 and 8.',
    )
    expect(validateMazeGenerationFields({ ...VALID, spareKeys: '17' }, null, 'game')).toBe(
      'Spare Keys must be a whole number between 0 and 16.',
    )
    // 16 spare keys with no doors uses the whole budget exactly → valid.
    expect(validateMazeGenerationFields({ ...VALID, spareKeys: '16' }, null, 'game')).toBeNull()
  })

  it('enforces the combined keys + doors budget', () => {
    // 2*8 + 1 = 17 > 16.
    expect(validateMazeGenerationFields({ ...VALID, doorCount: '8', spareDoors: '1' }, null, 'game')).toBe(
      'Total keys + doors (17) exceeds the limit of 16. ' +
        'Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.',
    )
  })

  it('ignores start/finish positions entirely, even when present and invalid', () => {
    expect(
      validateMazeGenerationFields({ ...VALID, startRow: '999', startCol: '0', finishRow: '', finishCol: '' }, null, 'game'),
    ).toBeNull()
  })
})

describe('validateMazeGenerationFields (maze subject — positions checked)', () => {
  // rows/cols = 10; start (1,1), finish (10,10).
  const MAZE_VALID = { ...VALID, startRow: '1', startCol: '1', finishRow: '10', finishCol: '10' }

  it('returns null for valid positions', () => {
    expect(validateMazeGenerationFields(MAZE_VALID, null, 'maze')).toBeNull()
  })

  it('bounds each coordinate to the grid', () => {
    expect(validateMazeGenerationFields({ ...MAZE_VALID, startRow: '11' }, null, 'maze')).toBe('Start Row must be between 1 and 10.')
    expect(validateMazeGenerationFields({ ...MAZE_VALID, startCol: '0' }, null, 'maze')).toBe('Start Column must be between 1 and 10.')
    expect(validateMazeGenerationFields({ ...MAZE_VALID, finishRow: '11' }, null, 'maze')).toBe('Finish Row must be between 1 and 10.')
    expect(validateMazeGenerationFields({ ...MAZE_VALID, finishCol: '0' }, null, 'maze')).toBe('Finish Column must be between 1 and 10.')
  })

  it('requires start and finish to differ', () => {
    expect(validateMazeGenerationFields({ ...MAZE_VALID, finishRow: '1', finishCol: '1' }, null, 'maze')).toBe(
      'Start and Finish cells must be different.',
    )
  })

  it('rejects missing positions (the game path leaves them undefined)', () => {
    expect(validateMazeGenerationFields(VALID, null, 'maze')).toBe('Start Row must be between 1 and 10.')
  })

  it('still requires a min solution length of at least 1', () => {
    expect(validateMazeGenerationFields({ ...MAZE_VALID, minSolutionLength: '0' }, null, 'maze')).toBe(
      'Min Solution Length must be a whole number of 1 or more.',
    )
  })
})

describe('MazeGenerationFields', () => {
  it('renders the size + count fields and no start/finish positions', () => {
    render(<MazeGenerationFields value={VALID} onChange={vi.fn()} />)
    for (const label of ['Rows', 'Columns', 'Min Solution Length', 'Doors', 'Spare Doors', 'Spare Keys', 'Enemies', 'Health', 'Treasure']) {
      expect(screen.getByLabelText(label)).toBeInTheDocument()
    }
    expect(screen.queryByLabelText(/Start Row/)).toBeNull()
    expect(screen.queryByLabelText(/Finish Row/)).toBeNull()
  })

  it('reports a patch when a field changes', () => {
    const onChange = vi.fn()
    render(<MazeGenerationFields value={VALID} onChange={onChange} />)
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '15' } })
    expect(onChange).toHaveBeenCalledWith({ rows: '15' })
    fireEvent.change(screen.getByLabelText('Doors'), { target: { value: '5' } })
    expect(onChange).toHaveBeenCalledWith({ doorCount: '5' })
  })
})
