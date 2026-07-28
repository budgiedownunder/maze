import { useRef, useState } from 'react'
import { useAppFeatures } from '../context/AppFeaturesContext'
import type { GenerateOptions } from '../types/api'
import {
  validateMazeGenerationFields,
  MAX_DOOR_COUNT, MAX_ENEMY_COUNT, MAX_HEALTH_COUNT, MAX_TREASURE_COUNT,
  MAX_SPARE_DOOR_COUNT, MAX_SPARE_KEY_COUNT,
} from '../utils/validation'
import { ModalTabStrip } from './ModalTabs'
import { modalTabPanelProps, type ModalTab } from '../utils/modalTabs'

// Tab identifiers grouping the generate fields so the dialog reads as a few
// short panels rather than one long scrolling list. The validation error and
// action buttons stay pinned below the panels so an error from any field is
// visible regardless of which tab is showing.
const TABS = [
  { id: 'sizePosition', label: 'Size & Position' },
  { id: 'features', label: 'Features' },
] as const satisfies readonly ModalTab[]
type GenerateTab = (typeof TABS)[number]['id']

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
  // Spare Doors defaults to 0 — the grid alone can't tell us which `'D'` cells
  // were decoys vs real path doors, so the safe default is "no extras".
  // Spare Keys, though, we can infer: a real door places one key, so any keys
  // beyond the door count are spare keys — seed those so regenerating preserves
  // them instead of dropping to 0 (clamped to the field's cap).
  const doors = grid.reduce((n, row) => n + row.filter(c => c === 'D').length, 0)
  const keys = grid.reduce((n, row) => n + row.filter(c => c === 'K').length, 0)
  const enemies = grid.reduce((n, row) => n + row.filter(c => c === 'E').length, 0)
  const healths = grid.reduce((n, row) => n + row.filter(c => c === 'H').length, 0)
  const treasures = grid.reduce((n, row) => n + row.filter(c => c === 'T').length, 0)
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
    spareKeys: String(Math.min(MAX_SPARE_KEY_COUNT, Math.max(0, keys - doors))),
    enemyCount: String(enemies),
    healthCount: String(healths),
    treasureCount: String(treasures),
  }
}

export function GenerateMazeModal({ grid, initialMinSpineLength, isLoading = false, error, onGenerate, onCancel }: Props) {
  const { max_maze_cells } = useAppFeatures()
  const defaults = defaultsFromGrid(grid)
  const [activeTab, setActiveTab] = useState<GenerateTab>('sizePosition')
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
  const [treasureCount, setTreasureCount] = useState(defaults.treasureCount)
  const [validationError, setValidationError] = useState<string | null>(null)

  // Whether a 1-based coordinate string sits within 1..max (inclusive).
  function inRange(coord: string, max: number) {
    const n = parseInt(coord, 10)
    return Number.isInteger(n) && n >= 1 && n <= max
  }

  // Last dimension value the clamp ran against — so a blur that didn't actually
  // change Rows/Columns is a no-op (we must not "fix" a deliberately out-of-range
  // start/finish the user is about to submit; that's validation's job).
  const lastClampedRows = useRef(defaults.rows)
  const lastClampedCols = useRef(defaults.cols)

  // Re-clamping start/finish to the new bounds runs only when the user commits
  // an actual dimension change (blur or Enter), not on every keystroke —
  // otherwise typing "15" would clamp against the intermediate "1" before the
  // "5" is typed. A committed change nudges a start/finish coordinate only if it
  // would now fall outside the new bounds (start→top/left corner, finish→new far
  // edge); in-range coordinates are left exactly as the author set them. Fields
  // are 1-based, so the dimension value is itself the far-edge coordinate.
  function commitRows() {
    if (rows === lastClampedRows.current) return
    lastClampedRows.current = rows
    const r = parseInt(rows, 10)
    if (!Number.isInteger(r) || r < 1) return
    if (!inRange(startRow, r)) setStartRow('1')
    if (!inRange(finishRow, r)) setFinishRow(String(r))
  }

  function commitCols() {
    if (cols === lastClampedCols.current) return
    lastClampedCols.current = cols
    const c = parseInt(cols, 10)
    if (!Number.isInteger(c) || c < 1) return
    if (!inRange(startCol, c)) setStartCol('1')
    if (!inRange(finishCol, c)) setFinishCol(String(c))
  }

  // Enter commits the dimension edit without leaving the field (mirrors the
  // blur behaviour) so keyboard users get the same clamp as click/tab-away.
  function commitOnEnter(e: React.KeyboardEvent, commit: () => void) {
    if (e.key === 'Enter') commit()
  }

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
    const treasures = parseInt(treasureCount, 10)

    // Shared with the game-definition editor: the same caps + feature-budget
    // rule, here with `'maze'` so the authored start/finish positions are also
    // validated. The re-clamp behaviour on dimension change stays local (it is
    // a UX nicety, not validation).
    const error = validateMazeGenerationFields(
      {
        rows,
        cols,
        minSolutionLength: minSpineLength,
        startRow,
        startCol,
        finishRow,
        finishCol,
        doorCount,
        spareDoors,
        spareKeys,
        enemyCount,
        healthCount,
        treasureCount,
      },
      max_maze_cells,
      'maze',
    )
    if (error) {
      setValidationError(error)
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
      treasureCount: treasures,
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
          <ModalTabStrip
            tabs={TABS}
            activeTab={activeTab}
            onSelect={setActiveTab}
            idPrefix="generate"
            ariaLabel="Generate settings"
          />

          {/* Scrollable middle region: only the active tab's fields scroll when
              the viewport is too short; the title, the pinned error + action
              buttons stay outside this box. */}
          <div className="modal-scroll-body">
            <div {...modalTabPanelProps('generate', 'sizePosition', activeTab)}>
              <label>
                Rows
                <input type="number" className="input" value={rows} min={3} autoFocus
                  onChange={e => { setRows(e.target.value); setValidationError(null) }}
                  onBlur={commitRows}
                  onKeyDown={e => commitOnEnter(e, commitRows)} />
              </label>
              <label>
                Columns
                <input type="number" className="input" value={cols} min={3}
                  onChange={e => { setCols(e.target.value); setValidationError(null) }}
                  onBlur={commitCols}
                  onKeyDown={e => commitOnEnter(e, commitCols)} />
              </label>
              <label>
                Min Start to Finish Distance
                <input type="number" className="input" value={minSpineLength} min={0}
                  onChange={e => { setMinSpineLength(e.target.value); setValidationError(null) }} />
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
            </div>

            <div {...modalTabPanelProps('generate', 'features', activeTab)}>
              <label>
                Doors
                <input type="number" className="input" value={doorCount} min={0} max={MAX_DOOR_COUNT}
                  onChange={e => { setDoorCount(e.target.value); setValidationError(null) }} />
              </label>
              <label>
                Spare Doors
                <input type="number" className="input" value={spareDoors} min={0} max={MAX_SPARE_DOOR_COUNT}
                  onChange={e => { setSpareDoors(e.target.value); setValidationError(null) }} />
              </label>
              <label>
                Spare Keys
                <input type="number" className="input" value={spareKeys} min={0} max={MAX_SPARE_KEY_COUNT}
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
              <label>
                Treasure
                <input type="number" className="input" value={treasureCount} min={0} max={MAX_TREASURE_COUNT}
                  onChange={e => { setTreasureCount(e.target.value); setValidationError(null) }} />
              </label>
            </div>
          </div>

          {/* Pinned below the tab panels: the validation error and action
              buttons stay visible regardless of the active tab. */}
          <div className="modal-tab-footer">
            {displayError && <p role="alert" className="error-msg">{displayError}</p>}
            <div className="modal-actions-row">
              <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
              <button type="submit" className="btn-primary" disabled={isLoading}>Generate</button>
            </div>
          </div>
        </form>
      </div>
    </div>
  )
}
