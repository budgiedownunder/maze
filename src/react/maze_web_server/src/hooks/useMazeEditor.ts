import { useState, useCallback, useMemo } from 'react'
import type { MazeDefinition } from '../types/api'

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
  const [grid, setGrid] = useState<string[][]>([])
  const [mazeName, setMazeName] = useState('')
  const [mazeId, setMazeId] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  const [activeCell, setActiveCell] = useState<CellPoint | null>(null)
  const [anchorCell, setAnchorCell] = useState<CellPoint | null>(null)
  const [solution, setSolutionState] = useState<Array<CellPoint> | null>(null)
  const [isRangeMode, setIsRangeMode] = useState(false)

  const initFromDefinition = useCallback(
    (id: string | null, name: string, definition: MazeDefinition) => {
      setMazeId(id)
      setMazeName(name)
      setGrid(definition.grid)
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

  const applyGenerated = useCallback((definition: MazeDefinition) => {
    setGrid(definition.grid)
    setActiveCell(null)
    setAnchorCell(null)
    setSolutionState(null)
    setIsDirty(true)
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

    for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
      for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
        const cell = grid[r]?.[c] ?? ' '
        totalCells++
        if (cell === 'W') { containsWall = true; wallCount++ }
        else if (cell === 'S') containsStart = true
        else if (cell === 'F') containsFinish = true
      }
    }

    const isSingleCell =
      selectionRect.minRow === selectionRect.maxRow &&
      selectionRect.minCol === selectionRect.maxCol
    const isAllWalls = totalCells > 0 && wallCount === totalCells
    const isEmpty = !containsWall && !containsStart && !containsFinish
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

  const setWall = useCallback(() => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = 'W'
        }
      }
      return next
    })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  const setStart = useCallback(() => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      // Clear any existing start cell
      for (let r = 0; r < next.length; r++) {
        for (let c = 0; c < next[r].length; c++) {
          if (next[r][c] === 'S') next[r][c] = ' '
        }
      }
      // Set the selected cell as start
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = 'S'
        }
      }
      return next
    })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  const setFinish = useCallback(() => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      // Clear any existing finish cell
      for (let r = 0; r < next.length; r++) {
        for (let c = 0; c < next[r].length; c++) {
          if (next[r][c] === 'F') next[r][c] = ' '
        }
      }
      // Set the selected cell as finish
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = 'F'
        }
      }
      return next
    })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  const clearCell = useCallback(() => {
    if (!selectionRect) return
    setGrid(prev => {
      const next = prev.map(r => [...r])
      for (let r = selectionRect.minRow; r <= selectionRect.maxRow; r++) {
        for (let c = selectionRect.minCol; c <= selectionRect.maxCol; c++) {
          next[r][c] = ' '
        }
      }
      return next
    })
    setSolutionState(null)
    setIsDirty(true)
  }, [selectionRect])

  // ── Structural editing ───────────────────────────────────────

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
  }, [selectionRect, grid])

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
  }, [selectionRect, grid])

  return {
    grid,
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
    clearCell,
    insertRowsBefore,
    deleteRows,
    insertColsBefore,
    deleteCols,
  }
}
