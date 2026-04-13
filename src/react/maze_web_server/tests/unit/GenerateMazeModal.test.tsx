import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GenerateMazeModal } from '../../src/components/GenerateMazeModal'

const mockOnGenerate = vi.fn()
const mockOnCancel = vi.fn()

// 3×3 grid: S top-left, W middle, F bottom-right
const sampleGrid: string[][] = [
  ['S', ' ', ' '],
  [' ', 'W', ' '],
  [' ', ' ', 'F'],
]

// 5×5 blank grid (no S or F)
const blankGrid: string[][] = Array.from({ length: 5 }, () => Array<string>(5).fill(' '))

function renderModal(overrides: Partial<React.ComponentProps<typeof GenerateMazeModal>> = {}) {
  return render(
    <GenerateMazeModal
      grid={sampleGrid}
      onGenerate={mockOnGenerate}
      onCancel={mockOnCancel}
      {...overrides}
    />
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

// ── Rendering & defaults ─────────────────────────────────────────

describe('GenerateMazeModal rendering and defaults', () => {
  it('renders all 7 labelled fields', () => {
    renderModal()
    expect(screen.getByLabelText('Rows')).toBeInTheDocument()
    expect(screen.getByLabelText('Columns')).toBeInTheDocument()
    expect(screen.getByLabelText('Start Row')).toBeInTheDocument()
    expect(screen.getByLabelText('Start Column')).toBeInTheDocument()
    expect(screen.getByLabelText('Finish Row')).toBeInTheDocument()
    expect(screen.getByLabelText('Finish Column')).toBeInTheDocument()
    expect(screen.getByLabelText('Min Solution Length')).toBeInTheDocument()
  })

  it('defaults Rows and Columns to the grid dimensions', () => {
    renderModal()
    expect(screen.getByLabelText('Rows')).toHaveValue(3)
    expect(screen.getByLabelText('Columns')).toHaveValue(3)
  })

  it('defaults Start Row/Col to the S position (1-based)', () => {
    renderModal()
    expect(screen.getByLabelText('Start Row')).toHaveValue(1)
    expect(screen.getByLabelText('Start Column')).toHaveValue(1)
  })

  it('defaults Finish Row/Col to the F position (1-based)', () => {
    renderModal()
    expect(screen.getByLabelText('Finish Row')).toHaveValue(3)
    expect(screen.getByLabelText('Finish Column')).toHaveValue(3)
  })

  it('defaults Min Solution Length to 1', () => {
    renderModal()
    expect(screen.getByLabelText('Min Solution Length')).toHaveValue(1)
  })

  it('defaults Start to row 1 col 1 when grid has no S', () => {
    renderModal({ grid: blankGrid })
    expect(screen.getByLabelText('Start Row')).toHaveValue(1)
    expect(screen.getByLabelText('Start Column')).toHaveValue(1)
  })

  it('defaults Finish to last row/col when grid has no F', () => {
    renderModal({ grid: blankGrid })
    expect(screen.getByLabelText('Finish Row')).toHaveValue(5)
    expect(screen.getByLabelText('Finish Column')).toHaveValue(5)
  })

  it('defaults Rows/Cols to 5 when given an empty grid', () => {
    renderModal({ grid: [] })
    expect(screen.getByLabelText('Rows')).toHaveValue(5)
    expect(screen.getByLabelText('Columns')).toHaveValue(5)
  })
})

// ── Validation ───────────────────────────────────────────────────

describe('GenerateMazeModal validation', () => {
  async function submitWith(fields: Record<string, string>) {
    renderModal()
    for (const [label, value] of Object.entries(fields)) {
      fireEvent.change(screen.getByLabelText(label), { target: { value } })
    }
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
  }

  it('shows error when Rows is less than 3', async () => {
    await submitWith({ Rows: '2' })
    expect(screen.getByRole('alert')).toHaveTextContent('Rows must be a whole number of 3 or more.')
  })

  it('shows error when Columns is less than 3', async () => {
    await submitWith({ Columns: '2' })
    expect(screen.getByRole('alert')).toHaveTextContent('Columns must be a whole number of 3 or more.')
  })

  it('shows error when Start Row is out of range', async () => {
    await submitWith({ 'Start Row': '0' })
    expect(screen.getByRole('alert')).toHaveTextContent('Start Row must be between 1 and 3.')
  })

  it('shows error when Start Row exceeds Rows', async () => {
    await submitWith({ 'Start Row': '4' })
    expect(screen.getByRole('alert')).toHaveTextContent('Start Row must be between 1 and 3.')
  })

  it('shows error when Start Column is out of range', async () => {
    await submitWith({ 'Start Column': '0' })
    expect(screen.getByRole('alert')).toHaveTextContent('Start Column must be between 1 and 3.')
  })

  it('shows error when Finish Row is out of range', async () => {
    await submitWith({ 'Finish Row': '4' })
    expect(screen.getByRole('alert')).toHaveTextContent('Finish Row must be between 1 and 3.')
  })

  it('shows error when Finish Column is out of range', async () => {
    await submitWith({ 'Finish Column': '4' })
    expect(screen.getByRole('alert')).toHaveTextContent('Finish Column must be between 1 and 3.')
  })

  it('shows error when Start and Finish are the same cell', async () => {
    // S is at (1,1) and F is at (3,3) in sampleGrid; move F to (1,1)
    await submitWith({ 'Finish Row': '1', 'Finish Column': '1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Start and Finish cells must be different.')
  })

  it('shows error when Min Solution Length is less than 1', async () => {
    await submitWith({ 'Min Solution Length': '0' })
    expect(screen.getByRole('alert')).toHaveTextContent('Min Solution Length must be a whole number of 1 or more.')
  })

  it('does not call onGenerate when validation fails', async () => {
    await submitWith({ Rows: '2' })
    expect(mockOnGenerate).not.toHaveBeenCalled()
  })
})

// ── Happy path ───────────────────────────────────────────────────

describe('GenerateMazeModal happy path', () => {
  it('calls onGenerate with correct 1-based GenerateOptions on valid submit', async () => {
    renderModal()
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(mockOnGenerate).toHaveBeenCalledWith({
      rowCount: 3,
      colCount: 3,
      startRow: 1,
      startCol: 1,
      finishRow: 3,
      finishCol: 3,
      minSpineLength: 1,
    })
  })

  it('calls onCancel when Cancel is clicked', async () => {
    renderModal()
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(mockOnCancel).toHaveBeenCalled()
  })
})

// ── Loading / error props ────────────────────────────────────────

describe('GenerateMazeModal loading and error props', () => {
  it('disables the Generate button when isLoading is true', () => {
    renderModal({ isLoading: true })
    expect(screen.getByRole('button', { name: 'Generate' })).toBeDisabled()
  })

  it('displays the error prop when there is no validation error', async () => {
    renderModal({ error: 'WASM generation failed' })
    expect(screen.getByRole('alert')).toHaveTextContent('WASM generation failed')
  })

  it('validation error takes priority over error prop', async () => {
    renderModal({ error: 'WASM generation failed' })
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '2' } })
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(screen.getByRole('alert')).toHaveTextContent('Rows must be a whole number of 3 or more.')
  })
})
