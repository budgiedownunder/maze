import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GenerateMazeModal } from '../../src/components/GenerateMazeModal'
import { AppFeaturesContext, APP_FEATURES_DEFAULTS } from '../../src/context/AppFeaturesContext'

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

function renderModalWithCap(maxMazeCells: number | null, overrides: Partial<React.ComponentProps<typeof GenerateMazeModal>> = {}) {
  return render(
    <AppFeaturesContext.Provider value={{ ...APP_FEATURES_DEFAULTS, max_maze_cells: maxMazeCells }}>
      <GenerateMazeModal
        grid={sampleGrid}
        onGenerate={mockOnGenerate}
        onCancel={mockOnCancel}
        {...overrides}
      />
    </AppFeaturesContext.Provider>
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

// ── Rendering & defaults ─────────────────────────────────────────

describe('GenerateMazeModal rendering and defaults', () => {
  it('renders every labelled field', () => {
    renderModal()
    expect(screen.getByLabelText('Rows')).toBeInTheDocument()
    expect(screen.getByLabelText('Columns')).toBeInTheDocument()
    expect(screen.getByLabelText('Start Row')).toBeInTheDocument()
    expect(screen.getByLabelText('Start Column')).toBeInTheDocument()
    expect(screen.getByLabelText('Finish Row')).toBeInTheDocument()
    expect(screen.getByLabelText('Finish Column')).toBeInTheDocument()
    expect(screen.getByLabelText('Min Solution Length')).toBeInTheDocument()
    expect(screen.getByLabelText('Doors')).toBeInTheDocument()
    expect(screen.getByLabelText('Spare Doors')).toBeInTheDocument()
    expect(screen.getByLabelText('Spare Keys')).toBeInTheDocument()
    expect(screen.getByLabelText('Enemies')).toBeInTheDocument()
    expect(screen.getByLabelText('Health')).toBeInTheDocument()
  })

  it('defaults Doors to 0 when the grid has no doors', () => {
    renderModal()
    expect(screen.getByLabelText('Doors')).toHaveValue(0)
  })

  it('caps the Doors spinner at 0 and the maximum permitted count', () => {
    // min/max attributes bound the native spinner so a user can't click below
    // 0 or above 8. The form's noValidate (asserted below) keeps submit
    // unblocked so the in-modal JS validation still fires for typed
    // out-of-range values.
    renderModal()
    const doors = screen.getByLabelText('Doors')
    expect(doors).toHaveAttribute('min', '0')
    expect(doors).toHaveAttribute('max', '8')
  })

  it('caps each number input spinner at its minimum valid value', () => {
    // Rows/Columns must be ≥ 3 (the smallest valid grid); the rest only need to
    // stay ≥ 0 (the JS validation enforces stricter lower bounds at submit
    // time, but the spinner just shouldn't go negative).
    renderModal()
    expect(screen.getByLabelText('Rows')).toHaveAttribute('min', '3')
    expect(screen.getByLabelText('Columns')).toHaveAttribute('min', '3')
    for (const label of [
      'Start Row',
      'Start Column',
      'Finish Row',
      'Finish Column',
      'Min Solution Length',
      'Doors',
      'Spare Doors',
      'Spare Keys',
      'Enemies',
      'Health',
    ]) {
      expect(screen.getByLabelText(label)).toHaveAttribute('min', '0')
    }
  })

  it('caps the Enemies spinner at 0 and the maximum permitted count', () => {
    renderModal()
    const enemies = screen.getByLabelText('Enemies')
    expect(enemies).toHaveAttribute('min', '0')
    expect(enemies).toHaveAttribute('max', '8')
  })

  it('caps the Health spinner at 0 and the maximum permitted count', () => {
    renderModal()
    const health = screen.getByLabelText('Health')
    expect(health).toHaveAttribute('min', '0')
    expect(health).toHaveAttribute('max', '8')
  })

  it('defaults Enemies to 0 when the grid has no enemies', () => {
    renderModal()
    expect(screen.getByLabelText('Enemies')).toHaveValue(0)
  })

  it('defaults Health to 0 when the grid has no health pickups', () => {
    renderModal()
    expect(screen.getByLabelText('Health')).toHaveValue(0)
  })

  it('initializes Enemies from the number of enemies already in the grid', () => {
    const gridWithEnemies: string[][] = [
      ['S', 'E', ' '],
      [' ', 'W', 'E'],
      [' ', 'E', 'F'],
    ]
    renderModal({ grid: gridWithEnemies })
    expect(screen.getByLabelText('Enemies')).toHaveValue(3)
  })

  it('initializes Health from the number of health pickups already in the grid', () => {
    const gridWithHealth: string[][] = [
      ['S', 'H', ' '],
      [' ', 'W', 'H'],
      [' ', ' ', 'F'],
    ]
    renderModal({ grid: gridWithHealth })
    expect(screen.getByLabelText('Health')).toHaveValue(2)
  })

  it('disables native form validation so the in-modal error message still fires', () => {
    const { container } = renderModal()
    const form = container.querySelector('form')
    expect(form).not.toBeNull()
    expect(form).toHaveAttribute('novalidate')
  })

  it('initializes Doors from the number of doors already in the grid', () => {
    const gridWithDoors: string[][] = [
      ['S', 'D', ' '],
      [' ', 'W', 'D'],
      [' ', ' ', 'F'],
    ]
    renderModal({ grid: gridWithDoors })
    expect(screen.getByLabelText('Doors')).toHaveValue(2)
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

  it('uses initialMinSpineLength prop as the default Min Solution Length', () => {
    renderModal({ initialMinSpineLength: 7 })
    expect(screen.getByLabelText('Min Solution Length')).toHaveValue(7)
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

  it('shows error when Doors exceeds the maximum', async () => {
    await submitWith({ Doors: '9' })
    expect(screen.getByRole('alert')).toHaveTextContent('Doors must be a whole number between 0 and 8.')
  })

  it('shows error when Doors is negative', async () => {
    await submitWith({ Doors: '-1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Doors must be a whole number between 0 and 8.')
  })

  it('shows error when Spare Doors exceeds the maximum', async () => {
    await submitWith({ 'Spare Doors': '9' })
    expect(screen.getByRole('alert')).toHaveTextContent('Spare Doors must be a whole number between 0 and 8.')
  })

  it('shows error when Spare Doors is negative', async () => {
    await submitWith({ 'Spare Doors': '-1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Spare Doors must be a whole number between 0 and 8.')
  })

  it('shows error when Spare Keys exceeds the maximum', async () => {
    await submitWith({ 'Spare Keys': '9' })
    expect(screen.getByRole('alert')).toHaveTextContent('Spare Keys must be a whole number between 0 and 8.')
  })

  it('shows error when Spare Keys is negative', async () => {
    await submitWith({ 'Spare Keys': '-1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Spare Keys must be a whole number between 0 and 8.')
  })

  it('shows error when Enemies exceeds the maximum', async () => {
    await submitWith({ Enemies: '9' })
    expect(screen.getByRole('alert')).toHaveTextContent('Enemies must be a whole number between 0 and 8.')
  })

  it('shows error when Enemies is negative', async () => {
    await submitWith({ Enemies: '-1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Enemies must be a whole number between 0 and 8.')
  })

  it('shows error when Health exceeds the maximum', async () => {
    await submitWith({ Health: '9' })
    expect(screen.getByRole('alert')).toHaveTextContent('Health must be a whole number between 0 and 8.')
  })

  it('shows error when Health is negative', async () => {
    await submitWith({ Health: '-1' })
    expect(screen.getByRole('alert')).toHaveTextContent('Health must be a whole number between 0 and 8.')
  })

  it('passes spare doors + spare keys through on a valid submit', async () => {
    await submitWith({ 'Spare Doors': '2', 'Spare Keys': '1' })
    expect(mockOnGenerate).toHaveBeenCalledWith(
      expect.objectContaining({ spareDoors: 2, spareKeys: 1 }),
    )
  })

  it('passes enemy + health counts through on a valid submit', async () => {
    await submitWith({ Enemies: '3', Health: '2' })
    expect(mockOnGenerate).toHaveBeenCalledWith(
      expect.objectContaining({ enemyCount: 3, healthCount: 2 }),
    )
  })

  it('does not call onGenerate when validation fails', async () => {
    await submitWith({ Rows: '2' })
    expect(mockOnGenerate).not.toHaveBeenCalled()
  })

  // Cross-field K + D budget: each real door brings one key, so the formula
  // is 2 * Doors + Spare Doors + Spare Keys <= MAX_TOTAL_FEATURES (16).
  it('rejects when 2*Doors + Spare Doors + Spare Keys exceeds the K + D budget', async () => {
    // 2*7 + 1 + 2 = 17 > 16 — over the cap. Field-level limits all individually pass.
    await submitWith({ Doors: '7', 'Spare Doors': '1', 'Spare Keys': '2' })
    expect(screen.getByRole('alert')).toHaveTextContent(
      /Total keys \+ doors \(17\) exceeds the limit of 16/,
    )
    expect(mockOnGenerate).not.toHaveBeenCalled()
  })

  it('accepts when 2*Doors + Spare Doors + Spare Keys equals the K + D budget', async () => {
    // 2*8 + 0 + 0 = 16 — exactly at the cap, should submit.
    await submitWith({ Doors: '8', 'Spare Doors': '0', 'Spare Keys': '0' })
    expect(mockOnGenerate).toHaveBeenCalledWith(
      expect.objectContaining({ doorCount: 8, spareDoors: 0, spareKeys: 0 }),
    )
  })
})

// ── max_maze_cells cap ───────────────────────────────────────────

describe('GenerateMazeModal max_maze_cells cap', () => {
  it('rejects rows × cols over the cap with a message that names the cap value', async () => {
    renderModalWithCap(3_600)
    // 61 × 60 = 3,660 cells.
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '61' } })
    fireEvent.change(screen.getByLabelText('Columns'), { target: { value: '60' } })
    fireEvent.change(screen.getByLabelText('Finish Row'), { target: { value: '61' } })
    fireEvent.change(screen.getByLabelText('Finish Column'), { target: { value: '60' } })
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(screen.getByRole('alert')).toHaveTextContent('Total cells (rows × columns) cannot exceed 3600.')
    expect(mockOnGenerate).not.toHaveBeenCalled()
  })

  it('accepts rows × cols exactly at the cap', async () => {
    renderModalWithCap(3_600)
    // 60 × 60 = 3,600 cells.
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '60' } })
    fireEvent.change(screen.getByLabelText('Columns'), { target: { value: '60' } })
    fireEvent.change(screen.getByLabelText('Finish Row'), { target: { value: '60' } })
    fireEvent.change(screen.getByLabelText('Finish Column'), { target: { value: '60' } })
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(mockOnGenerate).toHaveBeenCalledWith(expect.objectContaining({ rowCount: 60, colCount: 60 }))
  })

  it('does not enforce a cap when max_maze_cells is null', async () => {
    renderModalWithCap(null)
    // 200 × 200 = 40,000 cells — well above any practical cap; with cap=null
    // the modal must let it through.
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '200' } })
    fireEvent.change(screen.getByLabelText('Columns'), { target: { value: '200' } })
    fireEvent.change(screen.getByLabelText('Finish Row'), { target: { value: '200' } })
    fireEvent.change(screen.getByLabelText('Finish Column'), { target: { value: '200' } })
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(mockOnGenerate).toHaveBeenCalledWith(expect.objectContaining({ rowCount: 200, colCount: 200 }))
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
      doorCount: 0,
      spareDoors: 0,
      spareKeys: 0,
      enemyCount: 0,
      healthCount: 0,
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

  it('snaps a now-out-of-range start/finish to the corners when Rows shrinks (on blur)', async () => {
    // sampleGrid is 3×3 with S at (1,1), F at (3,3). Push start/finish rows to
    // the bottom of a 3-row grid, then shrink to 2 rows: both rows are now out
    // of range and must snap (start→1, finish→new max) once the Rows edit is
    // committed by blurring the field.
    renderModal()
    fireEvent.change(screen.getByLabelText('Start Row'), { target: { value: '3' } })
    fireEvent.change(screen.getByLabelText('Finish Row'), { target: { value: '3' } })
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '2' } })
    fireEvent.blur(screen.getByLabelText('Rows'))
    expect(screen.getByLabelText('Start Row')).toHaveValue(1)
    expect(screen.getByLabelText('Finish Row')).toHaveValue(2)
  })

  it('does not clamp on each keystroke — only on commit (blur)', async () => {
    // Regression guard: typing a multi-digit dimension must not clamp against an
    // intermediate value. Starting from 3 rows with finish at row 3, typing the
    // "1" then "5" of "15" (simulated as successive change events) must not snap
    // finish to 1 mid-edit; the in-range value survives once committed.
    renderModal()
    const rowsInput = screen.getByLabelText('Rows')
    fireEvent.change(rowsInput, { target: { value: '1' } })
    fireEvent.change(rowsInput, { target: { value: '15' } })
    // No blur yet: start/finish are untouched while typing.
    expect(screen.getByLabelText('Finish Row')).toHaveValue(3)
    fireEvent.blur(rowsInput)
    // After commit, the in-range finish (3 ≤ 15) is preserved, not reset.
    expect(screen.getByLabelText('Start Row')).toHaveValue(1)
    expect(screen.getByLabelText('Finish Row')).toHaveValue(3)
  })

  it('commits the dimension clamp on Enter as well as blur', async () => {
    renderModal()
    fireEvent.change(screen.getByLabelText('Start Row'), { target: { value: '3' } })
    fireEvent.change(screen.getByLabelText('Finish Row'), { target: { value: '3' } })
    fireEvent.change(screen.getByLabelText('Rows'), { target: { value: '2' } })
    fireEvent.keyDown(screen.getByLabelText('Rows'), { key: 'Enter' })
    expect(screen.getByLabelText('Start Row')).toHaveValue(1)
    expect(screen.getByLabelText('Finish Row')).toHaveValue(2)
  })

  it('leaves an in-range start/finish untouched when Columns grows (on blur)', async () => {
    // Growing the grid never invalidates an existing coordinate, so the
    // author's chosen start/finish must be preserved exactly.
    renderModal()
    fireEvent.change(screen.getByLabelText('Start Column'), { target: { value: '2' } })
    fireEvent.change(screen.getByLabelText('Finish Column'), { target: { value: '2' } })
    fireEvent.change(screen.getByLabelText('Columns'), { target: { value: '9' } })
    fireEvent.blur(screen.getByLabelText('Columns'))
    expect(screen.getByLabelText('Start Column')).toHaveValue(2)
    expect(screen.getByLabelText('Finish Column')).toHaveValue(2)
  })

  it('groups fields into Size & Position / Features tabs and switches panels on click', async () => {
    renderModal()
    // Size & Position is the default active tab: its panel holds the dimension
    // and start/finish fields; the Features panel is hidden.
    const sizePanel = screen.getByRole('tabpanel', { name: 'Size & Position' })
    expect(sizePanel).toBeVisible()
    expect(sizePanel).toContainElement(screen.getByLabelText('Rows'))
    expect(sizePanel).toContainElement(screen.getByLabelText('Start Row'))
    expect(screen.queryByRole('tabpanel', { name: 'Features' })).toBeNull()

    await userEvent.click(screen.getByRole('tab', { name: 'Features' }))
    const featuresPanel = screen.getByRole('tabpanel', { name: 'Features' })
    expect(featuresPanel).toBeVisible()
    expect(featuresPanel).toContainElement(screen.getByLabelText('Doors'))
    expect(screen.getByRole('tab', { name: 'Features' })).toHaveAttribute('aria-selected', 'true')
    // Switching tabs hides the Size & Position panel.
    expect(screen.queryByRole('tabpanel', { name: 'Size & Position' })).toBeNull()

    // The action buttons stay reachable regardless of the active tab.
    expect(screen.getByRole('button', { name: 'Generate' })).toBeVisible()
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
