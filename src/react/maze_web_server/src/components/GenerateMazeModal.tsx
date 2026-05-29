import { useState } from 'react'
import { useAppFeatures } from '../context/AppFeaturesContext'
import type { GenerateOptions } from '../types/api'
import {
  exceedsGenerateFeatureCap, exceedsMazeCellCap,
  MAX_DOOR_COUNT, MAX_ENEMY_COUNT, MAX_HEALTH_COUNT, MAX_TOTAL_FEATURES,
} from '../utils/validation'

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
  // Seed the Doors / Enemies / Health fields with the counts already in the maze
  // (so regenerating preserves the author's content), falling back to 0.
  // Spare Doors and Spare Keys default to 0 — the grid alone can't tell us
  // which `'D'` cells were decoys vs real path doors, so the safe default is
  // "no extras" and let the author opt in.
  const doors = grid.reduce((n, row) => n + row.filter(c => c === 'D').length, 0)
  const enemies = grid.reduce((n, row) => n + row.filter(c => c === 'E').length, 0)
  const healths = grid.reduce((n, row) => n + row.filter(c => c === 'H').length, 0)
  return {
    rows: String(rows),
    cols: String(cols),
    startRow: String((start?.row ?? 0) + 1),
    startCol: String((start?.col ?? 0) + 1),
    finishRow: String((finish?.row ?? rows - 1) + 1),
    finishCol: String((finish?.col ?? cols - 1) + 1),
    minSpineLength: '1',
    doorCount: String(doors),
    spareDoors: '0',
    spareKeys: '0',
    enemyCount: String(enemies),
    healthCount: String(healths),
  }
}

export function GenerateMazeModal({ grid, initialMinSpineLength, isLoading = false, error, onGenerate, onCancel }: Props) {
  const { max_maze_cells } = useAppFeatures()
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
  const [doorCount, setDoorCount] = useState(defaults.doorCount)
  const [spareDoors, setSpareDoors] = useState(defaults.spareDoors)
  const [spareKeys, setSpareKeys] = useState(defaults.spareKeys)
  const [enemyCount, setEnemyCount] = useState(defaults.enemyCount)
  const [healthCount, setHealthCount] = useState(defaults.healthCount)
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
    const doors = parseInt(doorCount, 10)
    const sdoors = parseInt(spareDoors, 10)
    const skeys = parseInt(spareKeys, 10)
    const enemies = parseInt(enemyCount, 10)
    const healths = parseInt(healthCount, 10)

    if (!Number.isInteger(r) || r < 3) {
      setValidationError('Rows must be a whole number of 3 or more.')
      return
    }
    if (!Number.isInteger(c) || c < 3) {
      setValidationError('Columns must be a whole number of 3 or more.')
      return
    }
    if (exceedsMazeCellCap(r, c, max_maze_cells)) {
      setValidationError(`Total cells (rows × columns) cannot exceed ${max_maze_cells}.`)
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
    if (!Number.isInteger(doors) || doors < 0 || doors > MAX_DOOR_COUNT) {
      setValidationError(`Doors must be a whole number between 0 and ${MAX_DOOR_COUNT}.`)
      return
    }
    if (!Number.isInteger(sdoors) || sdoors < 0 || sdoors > MAX_DOOR_COUNT) {
      setValidationError(`Spare Doors must be a whole number between 0 and ${MAX_DOOR_COUNT}.`)
      return
    }
    if (!Number.isInteger(skeys) || skeys < 0 || skeys > MAX_DOOR_COUNT) {
      setValidationError(`Spare Keys must be a whole number between 0 and ${MAX_DOOR_COUNT}.`)
      return
    }
    if (!Number.isInteger(enemies) || enemies < 0 || enemies > MAX_ENEMY_COUNT) {
      setValidationError(`Enemies must be a whole number between 0 and ${MAX_ENEMY_COUNT}.`)
      return
    }
    if (!Number.isInteger(healths) || healths < 0 || healths > MAX_HEALTH_COUNT) {
      setValidationError(`Health must be a whole number between 0 and ${MAX_HEALTH_COUNT}.`)
      return
    }
    // Cross-field budget: each real door contributes one 'K' and one 'D' to
    // the generated grid, so the formula counts doors twice. The cap mirrors
    // the key-aware solver's MAX_TOTAL_FEATURES so a generated maze always
    // has a solvable path the editor can display.
    if (exceedsGenerateFeatureCap(doors, sdoors, skeys)) {
      const total = 2 * doors + sdoors + skeys
      setValidationError(
        `Total keys + doors (${total}) exceeds the limit of ${MAX_TOTAL_FEATURES}. ` +
          `Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.`,
      )
      return
    }

    setValidationError(null)
    onGenerate({
      rowCount: r,
      colCount: c,
      startRow: sr,
      startCol: sc,
      finishRow: fr,
      finishCol: fc,
      minSpineLength: msl,
      doorCount: doors,
      spareDoors: sdoors,
      spareKeys: skeys,
      enemyCount: enemies,
      healthCount: healths,
    })
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label="Generate Maze" className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal modal-sm modal-with-scroll-body">
        <h2 className="modal-title">Generate Maze</h2>
        {/* noValidate keeps the in-modal JS validation as the source of truth —
            otherwise the `min`/`max` attributes on the number inputs below
            would let the browser block form submission for an out-of-range
            typed value, hiding the per-field "must be a whole number…" alerts.
            The attributes are still useful: they cap each input's native
            spinner (so the user can't click past the bound). */}
        <form className="modal-form" noValidate onSubmit={handleSubmit}>
          {/* Scrollable middle region: only the form fields and validation
              error scroll when the viewport is too short; the title above
              and the action buttons below stay pinned. */}
          <div className="modal-scroll-body">
            <label>
              Rows
              <input type="number" className="input" value={rows} min={3} autoFocus
                onChange={e => { setRows(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Columns
              <input type="number" className="input" value={cols} min={3}
                onChange={e => { setCols(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Start Row
              <input type="number" className="input" value={startRow} min={0}
                onChange={e => { setStartRow(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Start Column
              <input type="number" className="input" value={startCol} min={0}
                onChange={e => { setStartCol(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Finish Row
              <input type="number" className="input" value={finishRow} min={0}
                onChange={e => { setFinishRow(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Finish Column
              <input type="number" className="input" value={finishCol} min={0}
                onChange={e => { setFinishCol(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Min Solution Length
              <input type="number" className="input" value={minSpineLength} min={0}
                onChange={e => { setMinSpineLength(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Doors
              <input type="number" className="input" value={doorCount} min={0} max={MAX_DOOR_COUNT}
                onChange={e => { setDoorCount(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Spare Doors
              <input type="number" className="input" value={spareDoors} min={0} max={MAX_DOOR_COUNT}
                onChange={e => { setSpareDoors(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Spare Keys
              <input type="number" className="input" value={spareKeys} min={0} max={MAX_DOOR_COUNT}
                onChange={e => { setSpareKeys(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Enemies
              <input type="number" className="input" value={enemyCount} min={0} max={MAX_ENEMY_COUNT}
                onChange={e => { setEnemyCount(e.target.value); setValidationError(null) }} />
            </label>
            <label>
              Health
              <input type="number" className="input" value={healthCount} min={0} max={MAX_HEALTH_COUNT}
                onChange={e => { setHealthCount(e.target.value); setValidationError(null) }} />
            </label>
            {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          </div>
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary" disabled={isLoading}>Generate</button>
          </div>
        </form>
      </div>
    </div>
  )
}
