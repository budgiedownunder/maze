import { useState, useCallback, useMemo } from 'react'
import { useAppFeatures } from '../context/AppFeaturesContext'
import type { MazeDefinition } from '../types/api'
import type { CellEntity, CellOverride } from '../types/cellEntities'
import { exceedsMazeCellCap } from '../utils/validation'

export interface CellPoint {
  row: number
  col: number
}

export interface SelectionRect {
  minRow: number
  maxRow: number
  minCol: number
  maxCol: number
}

// ── Per-cell override map helpers ──────────────────────────────
// Overrides are kept in a Map keyed by "row,col" (O(1) lookup for rendering),
// replaced immutably on every change. The char grid stays the source of truth for
// cell types; this map carries only the optional per-cell characteristics, kept in
// lockstep with the grid by the editing/structural operations below.

const cellKey = (row: number, col: number): string => `${row},${col}`

const parseCellKey = (key: string): [number, number] => {
  const comma = key.indexOf(',')
  return [Number(key.slice(0, comma)), Number(key.slice(comma + 1))]
}

// Drops any overrides on cells inside `rect` (returns the same reference when nothing
// changed, so React can skip the re-render). Used when a fill/clear rewrites cells —
// a re-stamped or cleared cell resets to default characteristics.
function dropOverridesInRect(
  prev: Map<string, CellEntity>,
  rect: SelectionRect,
): Map<string, CellEntity> {
  let changed = false
  const next = new Map(prev)
  for (let r = rect.minRow; r <= rect.maxRow; r++) {
    for (let c = rect.minCol; c <= rect.maxCol; c++) {
      if (next.delete(cellKey(r, c))) changed = true
    }
  }
  return changed ? next : prev
}

// Remaps override coordinates when `count` rows/cols are inserted before index `at`:
// any override at/after the insert point shifts along that axis.
function remapOverridesForInsert(
  prev: Map<string, CellEntity>,
  axis: 'row' | 'col',
  at: number,
  count: number,
): Map<string, CellEntity> {
  if (prev.size === 0) return prev
  const next = new Map<string, CellEntity>()
  for (const [key, entity] of prev) {
    let [r, c] = parseCellKey(key)
    if (axis === 'row') { if (r >= at) r += count } else if (c >= at) { c += count }
    next.set(cellKey(r, c), entity)
  }
  return next
}

// Remaps override coordinates when the `count` rows/cols starting at index `at` are
// deleted: overrides inside the deleted band are dropped, those after it shift back.
function remapOverridesForDelete(
  prev: Map<string, CellEntity>,
  axis: 'row' | 'col',
  at: number,
  count: number,
): Map<string, CellEntity> {
  if (prev.size === 0) return prev
  const next = new Map<string, CellEntity>()
  for (const [key, entity] of prev) {
    let [r, c] = parseCellKey(key)
    const v = axis === 'row' ? r : c
    if (v >= at && v < at + count) continue
    if (v >= at + count) { if (axis === 'row') { r -= count } else { c -= count } }
    next.set(cellKey(r, c), entity)
  }
  return next
}

export interface SelectionStatus {
  isSingleCell: boolean
  containsWall: boolean
  containsStart: boolean
  containsFinish: boolean
  isAllWalls: boolean
  isStart: boolean
  isFinish: boolean
  isEmpty: boolean
  allColumnsSelected: boolean
  allRowsSelected: boolean
  hasSolveCells: boolean
  hasSolution: boolean
}

export function useMazeEditor() {
  const { max_maze_cells } = useAppFeatures()
  const [grid, setGrid] = useState<string[][]>([])
  const [mazeName, setMazeName] = useState('')
  const [mazeId, setMazeId] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  const [activeCell, setActiveCell] = useState<CellPoint | null>(null)
  const [anchorCell, setAnchorCell] = useState<CellPoint | null>(null)
  const [solution, setSolutionState] = useState<Array<CellPoint> | null>(null)
  const [isRangeMode, setIsRangeMode] = useState(false)
  // Per-cell overrides, parallel to the char grid (see helpers above). Sparse: only
  // cells carrying a non-default characteristic appear.
  const [overrides, setOverrides] = useState<Map<string, CellEntity>>(new Map())

  const initFromDefinition = useCallback(
    (
      id: string | null,
      name: string,
      definition: MazeDefinition,
      cellOverrides: CellOverride[] = [],
    ) => {
      setMazeId(id)
      setMazeName(name)
      setGrid(definition.grid)
      setOverrides(new Map(cellOverrides.map(o => [cellKey(o.row, o.col), o.entity])))
      setIsDirty(false)
      setActiveCell(null)
      setAnchorCell(null)
      setSolutionState(null)
    },
    [],
  )

  const markSaved = useCallback((id: string, name: string) => {
    setMazeId(id)
    setMazeName(name)
    setIsDirty(false)
  }, [])

  const applyGenerated = useCallback((grid: string[][], overrides: CellOverride[] = []) => {
    setGrid(grid)
    // The generator can emit per-cell overrides (e.g. a treasure's style), so
    // seed the override map from them rather than clearing it.
    setOverrides(new Map(overrides.map(o => [cellKey(o.row, o.col), o.entity])))
    setActiveCell(null)
    setAnchorCell(null)
    setSolutionState(null)
    setIsDirty(true)
  }, [])

  // ── Per-cell overrides ───────────────────────────────────────

  const getOverride = useCallback(
    (row: number, col: number): CellEntity | undefined => overrides.get(cellKey(row, col)),
    [overrides],
  )

  // Sets (replaces) the override on a single cell. Caller ensures the entity's `type`
  // matches the cell's char; persistence validates it again at the WASM boundary.
  const setOverride = useCallback((row: number, col: number, entity: CellEntity) => {
    setOverrides(prev => {
      const next = new Map(prev)
      next.set(cellKey(row, col), entity)
      return next
    })
    setIsDirty(true)
  }, [])

  const clearOverride = useCallback((row: number, col: number) => {
    if (!overrides.has(cellKey(row, col))) return  // no-op: don't dirty the maze
    setOverrides(prev => {
      const next = new Map(prev)
      next.delete(cellKey(row, col))
      return next
    })
    setIsDirty(true)
  }, [overrides])

  // Snapshot of the current overrides as a list, for persistence (the WASM codec
  // takes `CellOverride[]`).
  const getOverridesList = useCallback((): CellOverride[] => {
    return Array.from(overrides, ([key, entity]) => {
      const [row, col] = parseCellKey(key)
      return { row, col, entity }
    })
  }, [overrides])

  const applySolution = useCallback((path: Array<CellPoint>) => {
    if (anchorCell !== null) setActiveCell(anchorCell)
    setAnchorCell(null)
    setIsRangeMode(false)
    setSolutionState(path)
  }, [anchorCell])

  const clearSolution = useCallback(() => {
    setSolutionState(null)
  }, [])

  // ── Derived selection rect ───────────────────────────────────

  const selectionRect = useMemo((): SelectionRect | null => {
    if (!activeCell) return null
    if (!anchorCell) {
      return {
        minRow: activeCell.row, maxRow: activeCell.row,
        minCol: activeCell.col, maxCol: activeCell.col,
      }
    }
    return {
      minRow: Math.min(activeCell.row, anchorCell.row),
      maxRow: Math.max(activeCell.row, anchorCell.row),
      minCol: Math.min(activeCell.col, anchorCell.col),
      maxCol: Math.max(activeCell.col, anchorCell.col),
    }
  }, [activeCell, anchorCell])

  // ── Derived selection status ─────────────────────────────────

  const selectionStatus = useMemo((): SelectionStatus => {
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    const hasSolveCells =
      grid.some(r => r.includes('S')) && grid.some(r => r.includes('F'))
    const hasSolution = solution !== null

    if (!selectionRect || rows === 0) {
      return {
        isSingleCell: false, containsWall: false, containsStart: false,
        containsFinish: false, isAllWalls: false, isStart: false, isFinish: false,
        isEmpty: true, allColumnsSelected: false, allRowsSelected: false,
        hasSolveCells, hasSolution,
      }
    }

    let wallCount = 0
    let totalCells = 0
    let containsWall = false
    let containsStart = false
    let containsFinish = false
    let containsKey = false
    let containsDoor = false
    let containsEnemy = false
    let containsHealth = false
    let containsTreasure = false

    for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
      for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
        const cell = grid[r]?.[c] ?? ' '
        totalCells++
        if (cell === 'W') { containsWall = true; wallCount++ }
        else if (cell === 'S') containsStart = true
        else if (cell === 'F') containsFinish = true
        else if (cell === 'K') containsKey = true
        else if (cell === 'D') containsDoor = true
        else if (cell === 'E') containsEnemy = true
        else if (cell === 'H') containsHealth = true
        else if (cell === 'T') containsTreasure = true
      }
    }

    const isSingleCell =
      selectionRect.minRow === selectionRect.maxRow &&
      selectionRect.minCol === selectionRect.maxCol
    const isAllWalls = totalCells > 0 && wallCount === totalCells
    const isEmpty =
      !containsWall && !containsStart && !containsFinish && !containsKey && !containsDoor && !containsEnemy && !containsHealth && !containsTreasure
    const isStart = isSingleCell && containsStart
    const isFinish = isSingleCell && containsFinish
    const allColumnsSelected =
      selectionRect.minCol === 0 && selectionRect.maxCol === cols - 1
    const allRowsSelected =
      selectionRect.minRow === 0 && selectionRect.maxRow === rows - 1

    return {
      isSingleCell, containsWall, containsStart, containsFinish,
      isAllWalls, isStart, isFinish, isEmpty,
      allColumnsSelected, allRowsSelected, hasSolveCells, hasSolution,
    }
  }, [grid, selectionRect, solution])

  // ── Range mode ───────────────────────────────────────────────

  // Single unified test: multi-cell selection is active when isRangeMode is true.
  // On desktop the Shift key feeds into the `shift` parameter of each navigation
  // function; on mobile the Select/Done toolbar buttons set isRangeMode directly.
  // All navigation functions use `effectiveShift = shift || isRangeMode` so both
  // mechanisms drive identical behaviour.

  const enableRangeMode = useCallback(() => {
    setIsRangeMode(true)
  }, [])

  const disableRangeMode = useCallback(() => {
    setIsRangeMode(false)
    setAnchorCell(null)
  }, [])

  // ── Navigation ───────────────────────────────────────────────

  const activateCell = useCallback((row: number, col: number, shift: boolean) => {
    const effectiveShift = shift || isRangeMode
    if (!effectiveShift) {
      setActiveCell({ row, col })
      setAnchorCell(null)
    } else {
      // If no anchor yet, fix the current active as anchor; then move active
      setAnchorCell(prev => prev ?? activeCell)
      setActiveCell({ row, col })
    }
  }, [activeCell, isRangeMode])

  // Select all cells (used by corner header click)
  const selectAll = useCallback(() => {
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    if (rows === 0 || cols === 0) return
    setActiveCell({ row: 0, col: 0 })
    setAnchorCell({ row: rows - 1, col: cols - 1 })
  }, [grid])

  // Full-row selection (used by row header clicks)
  const activateRow = useCallback((row: number, shift: boolean) => {
    const cols = grid.length > 0 ? grid[0].length : 0
    if (cols === 0) return
    const effectiveShift = shift || isRangeMode
    if (!effectiveShift || activeCell === null) {
      setActiveCell({ row, col: 0 })
      setAnchorCell({ row, col: cols - 1 })
    } else {
      // Extend: keep anchor row position, expand cols to full width, extend row range
      const anchor = anchorCell ?? activeCell
      setAnchorCell({ row: anchor.row, col: 0 })
      setActiveCell({ row, col: cols - 1 })
    }
  }, [grid, activeCell, anchorCell, isRangeMode])

  // Full-column selection (used by column header clicks)
  const activateCol = useCallback((col: number, shift: boolean) => {
    const rows = grid.length
    if (rows === 0) return
    const effectiveShift = shift || isRangeMode
    if (!effectiveShift || activeCell === null) {
      setActiveCell({ row: 0, col })
      setAnchorCell({ row: rows - 1, col })
    } else {
      // Extend: keep anchor col position, expand rows to full height, extend col range
      const anchor = anchorCell ?? activeCell
      setAnchorCell({ row: 0, col: anchor.col })
      setActiveCell({ row: rows - 1, col })
    }
  }, [grid, activeCell, anchorCell, isRangeMode])

  const moveActive = useCallback((
    dRow: number, dCol: number, shift: boolean, ctrl: boolean,
  ) => {
    if (!activeCell) return
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    if (rows === 0) return

    let newRow: number
    let newCol: number

    if (ctrl) {
      newRow = dRow < 0 ? 0 : dRow > 0 ? rows - 1 : activeCell.row
      newCol = dCol < 0 ? 0 : dCol > 0 ? cols - 1 : activeCell.col
    } else {
      newRow = Math.max(0, Math.min(rows - 1, activeCell.row + dRow))
      newCol = Math.max(0, Math.min(cols - 1, activeCell.col + dCol))
    }

    const effectiveShift = shift || isRangeMode
    if (!effectiveShift) {
      setActiveCell({ row: newRow, col: newCol })
      setAnchorCell(null)
    } else {
      if (anchorCell === null) setAnchorCell(activeCell)
      setActiveCell({ row: newRow, col: newCol })
    }
  }, [activeCell, anchorCell, grid, isRangeMode])

  const moveActiveHome = useCallback((shift: boolean, ctrl: boolean) => {
    if (!activeCell) return
    const newRow = ctrl ? 0 : activeCell.row
    const newCol = 0

    const effectiveShift = shift || isRangeMode
    if (!effectiveShift) {
      setActiveCell({ row: newRow, col: newCol })
      setAnchorCell(null)
    } else {
      if (anchorCell === null) setAnchorCell(activeCell)
      setActiveCell({ row: newRow, col: newCol })
    }
  }, [activeCell, anchorCell, isRangeMode])

  const moveActiveEnd = useCallback((shift: boolean, ctrl: boolean) => {
    if (!activeCell) return
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    const newRow = ctrl ? rows - 1 : activeCell.row
    const newCol = cols - 1

    const effectiveShift = shift || isRangeMode
    if (!effectiveShift) {
      setActiveCell({ row: newRow, col: newCol })
      setAnchorCell(null)
    } else {
      if (anchorCell === null) setAnchorCell(activeCell)
      setActiveCell({ row: newRow, col: newCol })
    }
  }, [activeCell, anchorCell, grid, isRangeMode])

  // ── Cell editing ─────────────────────────────────────────────

  // Sets every cell in the current selection to `char`, clears any displayed
  // solution, and marks the maze dirty. No-op when nothing is selected.
  const fillSelection = useCallback((char: string) => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = char
        }
      }
      return next
    })
    setOverrides(prev => dropOverridesInRect(prev, selectionRect))
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  // Like `fillSelection`, but first clears any existing occurrence of `char`
  // elsewhere in the grid — for cells limited to a single instance (start, finish).
  const setUniqueCell = useCallback((char: string) => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      for (let r = 0; r < next.length; r++) {
        for (let c = 0; c < next[r].length; c++) {
          if (next[r][c] === char) next[r][c] = ' '
        }
      }
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = char
        }
      }
      return next
    })
    // Start/Finish are not overridable, and any cell rewritten into one loses its
    // override; the cells cleared elsewhere were S/F (never overridden), so dropping
    // overrides across the written selection is sufficient.
    setOverrides(prev => dropOverridesInRect(prev, selectionRect))
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  const setWall = useCallback(() => fillSelection('W'), [fillSelection])
  const setStart = useCallback(() => setUniqueCell('S'), [setUniqueCell])
  const setFinish = useCallback(() => setUniqueCell('F'), [setUniqueCell])
  const setKey = useCallback(() => fillSelection('K'), [fillSelection])
  const setDoor = useCallback(() => fillSelection('D'), [fillSelection])
  const setEnemy = useCallback(() => fillSelection('E'), [fillSelection])
  const setHealth = useCallback(() => fillSelection('H'), [fillSelection])
  const setTreasure = useCallback(() => fillSelection('T'), [fillSelection])
  const clearCell = useCallback(() => fillSelection(' '), [fillSelection])

  // ── Structural editing ───────────────────────────────────────

  const canInsertRows = useMemo((): boolean => {
    if (!selectionRect) return true
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    const insertCount = selectionRect.maxRow - selectionRect.minRow + 1
    return !exceedsMazeCellCap(rows + insertCount, cols, max_maze_cells)
  }, [selectionRect, grid, max_maze_cells])

  const canInsertColumns = useMemo((): boolean => {
    if (!selectionRect) return true
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0
    const insertCount = selectionRect.maxCol - selectionRect.minCol + 1
    return !exceedsMazeCellCap(rows, cols + insertCount, max_maze_cells)
  }, [selectionRect, grid, max_maze_cells])

  const insertRowsBefore = useCallback(() => {
    if (!selectionRect) return
    const cols = grid.length > 0 ? grid[0].length : 0
    const insertAt = selectionRect.minRow
    const insertCount = selectionRect.maxRow - selectionRect.minRow + 1
    setGrid(prev => {
      const next = [...prev]
      const newRows = Array.from({ length: insertCount }, () => Array<string>(cols).fill(' '))
      next.splice(insertAt, 0, ...newRows)
      return next
    })
    setOverrides(prev => remapOverridesForInsert(prev, 'row', insertAt, insertCount))
    setActiveCell({ row: insertAt, col: 0 })
    setAnchorCell({ row: insertAt + insertCount - 1, col: cols - 1 })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect, grid])

  const deleteRows = useCallback(() => {
    if (!selectionRect) return
    const { minRow, maxRow } = selectionRect
    const deleteCount = maxRow - minRow + 1
    const newRowCount = grid.length - deleteCount
    setGrid(prev => {
      const next = [...prev]
      next.splice(minRow, deleteCount)
      return next
    })
    setOverrides(prev => remapOverridesForDelete(prev, 'row', minRow, deleteCount))
    if (newRowCount > 0) {
      // Clamp each end of the selection to the new grid bounds, preserving columns and
      // which end is active vs anchor (direction the user built the selection from).
      const newActiveRow = Math.min(activeCell!.row, newRowCount - 1)
      setActiveCell({ row: newActiveRow, col: activeCell!.col })
      if (anchorCell) {
        const newAnchorRow = Math.min(anchorCell.row, newRowCount - 1)
        setAnchorCell({ row: newAnchorRow, col: anchorCell.col })
      } else {
        setAnchorCell(null)
      }
    } else {
      setActiveCell(null)
      setAnchorCell(null)
    }
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect, grid, activeCell, anchorCell])

  const insertColsBefore = useCallback(() => {
    if (!selectionRect) return
    const rows = grid.length
    const insertAt = selectionRect.minCol
    const insertCount = selectionRect.maxCol - selectionRect.minCol + 1
    setGrid(prev =>
      prev.map(row => {
        const next = [...row]
        next.splice(insertAt, 0, ...Array<string>(insertCount).fill(' '))
        return next
      })
    )
    setOverrides(prev => remapOverridesForInsert(prev, 'col', insertAt, insertCount))
    setActiveCell({ row: 0, col: insertAt })
    setAnchorCell({ row: rows - 1, col: insertAt + insertCount - 1 })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect, grid])

  const deleteCols = useCallback(() => {
    if (!selectionRect) return
    const { minCol, maxCol } = selectionRect
    const deleteCount = maxCol - minCol + 1
    const rows = grid.length
    const newColCount = (rows > 0 ? grid[0].length : 0) - deleteCount
    setGrid(prev =>
      prev.map(row => {
        const next = [...row]
        next.splice(minCol, deleteCount)
        return next
      })
    )
    setOverrides(prev => remapOverridesForDelete(prev, 'col', minCol, deleteCount))
    if (rows > 0 && newColCount > 0) {
      // Clamp each end of the selection to the new grid bounds, preserving rows and
      // which end is active vs anchor (direction the user built the selection from).
      const newActiveCol = Math.min(activeCell!.col, newColCount - 1)
      setActiveCell({ row: activeCell!.row, col: newActiveCol })
      if (anchorCell) {
        const newAnchorCol = Math.min(anchorCell.col, newColCount - 1)
        setAnchorCell({ row: anchorCell.row, col: newAnchorCol })
      } else {
        setAnchorCell(null)
      }
    } else {
      setActiveCell(null)
      setAnchorCell(null)
    }
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect, grid, activeCell, anchorCell])

  return {
    grid,
    overrides,
    mazeName,
    mazeId,
    isDirty,
    activeCell,
    anchorCell,
    solution,
    isRangeMode,
    selectionStatus,
    initFromDefinition,
    markSaved,
    applyGenerated,
    applySolution,
    clearSolution,
    getOverride,
    setOverride,
    clearOverride,
    getOverridesList,
    selectAll,
    activateCell,
    activateRow,
    activateCol,
    moveActive,
    moveActiveHome,
    moveActiveEnd,
    enableRangeMode,
    disableRangeMode,
    setWall,
    setStart,
    setFinish,
    setKey,
    setDoor,
    setEnemy,
    setHealth,
    setTreasure,
    clearCell,
    insertRowsBefore,
    deleteRows,
    insertColsBefore,
    deleteCols,
    canInsertRows,
    canInsertColumns,
  }
}
