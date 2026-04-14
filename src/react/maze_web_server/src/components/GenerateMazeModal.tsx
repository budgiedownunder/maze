import { useState } from 'react'
import type { GenerateOptions } from '../types/api'

interface Props {
  grid: string[][]
  initialMinSpineLength?: number
  isLoading?: boolean
  error?: string | null
  onGenerate: (options: GenerateOptions) => void
  onCancel: () => void
}

function findCell(grid: string[][], value: string): { row: number; col: number } | null {
  for (let r = 0; r < grid.length; r++)
    for (let c = 0; c < (grid[r]?.length ?? 0); c++)
      if (grid[r][c] === value) return { row: r, col: c }
  return null
}

function defaultsFromGrid(grid: string[][]) {
  const rows = grid.length || 5
  const cols = grid[0]?.length || 5
  const start = findCell(grid, 'S')
  const finish = findCell(grid, 'F')
  return {
    rows: String(rows),
    cols: String(cols),
    startRow: String((start?.row ?? 0) + 1),
    startCol: String((start?.col ?? 0) + 1),
    finishRow: String((finish?.row ?? rows - 1) + 1),
    finishCol: String((finish?.col ?? cols - 1) + 1),
    minSpineLength: '1',
  }
}

export function GenerateMazeModal({ grid, initialMinSpineLength, isLoading = false, error, onGenerate, onCancel }: Props) {
  const defaults = defaultsFromGrid(grid)
  const [rows, setRows] = useState(defaults.rows)
  const [cols, setCols] = useState(defaults.cols)
  const [startRow, setStartRow] = useState(defaults.startRow)
  const [startCol, setStartCol] = useState(defaults.startCol)
  const [finishRow, setFinishRow] = useState(defaults.finishRow)
  const [finishCol, setFinishCol] = useState(defaults.finishCol)
  const [minSpineLength, setMinSpineLength] = useState(
    initialMinSpineLength != null ? String(initialMinSpineLength) : defaults.minSpineLength,
  )
  const [validationError, setValidationError] = useState<string | null>(null)

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const r = parseInt(rows, 10)
    const c = parseInt(cols, 10)
    const sr = parseInt(startRow, 10)
    const sc = parseInt(startCol, 10)
    const fr = parseInt(finishRow, 10)
    const fc = parseInt(finishCol, 10)
    const msl = parseInt(minSpineLength, 10)

    if (!Number.isInteger(r) || r < 3) {
      setValidationError('Rows must be a whole number of 3 or more.')
      return
    }
    if (!Number.isInteger(c) || c < 3) {
      setValidationError('Columns must be a whole number of 3 or more.')
      return
    }
    if (!Number.isInteger(sr) || sr < 1 || sr > r) {
      setValidationError(`Start Row must be between 1 and ${r}.`)
      return
    }
    if (!Number.isInteger(sc) || sc < 1 || sc > c) {
      setValidationError(`Start Column must be between 1 and ${c}.`)
      return
    }
    if (!Number.isInteger(fr) || fr < 1 || fr > r) {
      setValidationError(`Finish Row must be between 1 and ${r}.`)
      return
    }
    if (!Number.isInteger(fc) || fc < 1 || fc > c) {
      setValidationError(`Finish Column must be between 1 and ${c}.`)
      return
    }
    if (sr === fr && sc === fc) {
      setValidationError('Start and Finish cells must be different.')
      return
    }
    if (!Number.isInteger(msl) || msl < 1) {
      setValidationError('Min Solution Length must be a whole number of 1 or more.')
      return
    }

    setValidationError(null)
    onGenerate({ rowCount: r, colCount: c, startRow: sr, startCol: sc, finishRow: fr, finishCol: fc, minSpineLength: msl })
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label="Generate Maze" className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">Generate Maze</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <label>
            Rows
            <input type="number" className="input" value={rows} autoFocus
              onChange={e => { setRows(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Columns
            <input type="number" className="input" value={cols}
              onChange={e => { setCols(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Start Row
            <input type="number" className="input" value={startRow}
              onChange={e => { setStartRow(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Start Column
            <input type="number" className="input" value={startCol}
              onChange={e => { setStartCol(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Finish Row
            <input type="number" className="input" value={finishRow}
              onChange={e => { setFinishRow(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Finish Column
            <input type="number" className="input" value={finishCol}
              onChange={e => { setFinishCol(e.target.value); setValidationError(null) }} />
          </label>
          <label>
            Min Solution Length
            <input type="number" className="input" value={minSpineLength}
              onChange={e => { setMinSpineLength(e.target.value); setValidationError(null) }} />
          </label>
          {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary" disabled={isLoading}>Generate</button>
          </div>
        </form>
      </div>
    </div>
  )
}
