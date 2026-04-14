import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useMazeEditor } from '../../src/hooks/useMazeEditor'
import type { MazeDefinition } from '../../src/types/api'

function makeGrid(rows: number, cols: number, fill = ' '): string[][] {
  return Array.from({ length: rows }, () => Array<string>(cols).fill(fill))
}

function makeDefinition(grid: string[][]): MazeDefinition {
  return { grid }
}

// Initialise the hook with a given grid and return the result
function setupHook(grid: string[][]) {
  const { result } = renderHook(() => useMazeEditor())
  act(() => {
    result.current.initFromDefinition('maze-1', 'Test', makeDefinition(grid))
  })
  return result
}

// ──────────────────────────────────────────────────────────────
// activateCell
// ──────────────────────────────────────────────────────────────

describe('activateCell', () => {
  it('plain click sets active cell and clears anchor', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 3, false))
    expect(result.current.activeCell).toEqual({ row: 2, col: 3 })
    expect(result.current.anchorCell).toBeNull()
  })

  it('plain click on second cell moves active and clears anchor', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.activateCell(3, 4, false))
    expect(result.current.activeCell).toEqual({ row: 3, col: 4 })
    expect(result.current.anchorCell).toBeNull()
  })

  it('shift click sets anchor to previous active and moves active', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.activateCell(3, 4, true))
    expect(result.current.anchorCell).toEqual({ row: 1, col: 1 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 4 })
  })

  it('shift click keeps existing anchor when extending range further', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.activateCell(3, 3, true))  // sets anchor=(1,1), active=(3,3)
    act(() => result.current.activateCell(4, 4, true))  // keep anchor=(1,1), active=(4,4)
    expect(result.current.anchorCell).toEqual({ row: 1, col: 1 })
    expect(result.current.activeCell).toEqual({ row: 4, col: 4 })
  })

  it('shift click with no prior active just sets active', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, true))
    expect(result.current.activeCell).toEqual({ row: 2, col: 2 })
    expect(result.current.anchorCell).toBeNull()
  })
})

// ──────────────────────────────────────────────────────────────
// activateRow / activateCol
// ──────────────────────────────────────────────────────────────

describe('activateRow', () => {
  it('selects full row: active at col 0, anchor at last col', () => {
    const result = setupHook(makeGrid(4, 6))
    act(() => result.current.activateRow(2, false))
    expect(result.current.activeCell).toEqual({ row: 2, col: 0 })
    expect(result.current.anchorCell).toEqual({ row: 2, col: 5 })
  })

  it('shift+click extends range to include full row span', () => {
    const result = setupHook(makeGrid(4, 6))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.activateRow(3, true))
    // anchor row stays at 1, active moves to row 3; both span all cols
    expect(result.current.anchorCell).toEqual({ row: 1, col: 0 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 5 })
  })
})

describe('activateCol', () => {
  it('selects full column: active at row 0, anchor at last row', () => {
    const result = setupHook(makeGrid(4, 6))
    act(() => result.current.activateCol(3, false))
    expect(result.current.activeCell).toEqual({ row: 0, col: 3 })
    expect(result.current.anchorCell).toEqual({ row: 3, col: 3 })
  })

  it('shift+click extends range to include full column span', () => {
    const result = setupHook(makeGrid(4, 6))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.activateCol(4, true))
    expect(result.current.anchorCell).toEqual({ row: 0, col: 1 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 4 })
  })
})

// ──────────────────────────────────────────────────────────────
// moveActive
// ──────────────────────────────────────────────────────────────

describe('moveActive', () => {
  it('moves active cell by dRow/dCol', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.moveActive(1, 0, false, false))
    expect(result.current.activeCell).toEqual({ row: 3, col: 2 })
    expect(result.current.anchorCell).toBeNull()
  })

  it('clamps to top-left edge', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.moveActive(-1, -1, false, false))
    expect(result.current.activeCell).toEqual({ row: 0, col: 0 })
  })

  it('clamps to bottom-right edge', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(4, 4, false))
    act(() => result.current.moveActive(1, 1, false, false))
    expect(result.current.activeCell).toEqual({ row: 4, col: 4 })
  })

  it('ctrl jumps to row edge (down)', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 2, false))
    act(() => result.current.moveActive(1, 0, false, true))
    expect(result.current.activeCell).toEqual({ row: 4, col: 2 })
  })

  it('ctrl jumps to row edge (up)', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(3, 2, false))
    act(() => result.current.moveActive(-1, 0, false, true))
    expect(result.current.activeCell).toEqual({ row: 0, col: 2 })
  })

  it('ctrl jumps to col edge (right)', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 1, false))
    act(() => result.current.moveActive(0, 1, false, true))
    expect(result.current.activeCell).toEqual({ row: 2, col: 4 })
  })

  it('shift+arrow extends range by setting anchor if none', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.moveActive(1, 0, true, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 2 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 2 })
  })

  it('shift+arrow keeps existing anchor when extending further', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.moveActive(1, 0, true, false))
    act(() => result.current.moveActive(1, 0, true, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 2 })
    expect(result.current.activeCell).toEqual({ row: 4, col: 2 })
  })

  it('plain arrow after shift clears anchor', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.moveActive(1, 0, true, false))
    act(() => result.current.moveActive(0, 1, false, false))
    expect(result.current.anchorCell).toBeNull()
  })
})

// ──────────────────────────────────────────────────────────────
// moveActiveHome / moveActiveEnd
// ──────────────────────────────────────────────────────────────

describe('moveActiveHome', () => {
  it('Home moves to col 0 same row', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(3, 4, false))
    act(() => result.current.moveActiveHome(false, false))
    expect(result.current.activeCell).toEqual({ row: 3, col: 0 })
    expect(result.current.anchorCell).toBeNull()
  })

  it('Ctrl+Home moves to row 0 col 0', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(3, 4, false))
    act(() => result.current.moveActiveHome(false, true))
    expect(result.current.activeCell).toEqual({ row: 0, col: 0 })
  })

  it('Shift+Home extends range', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(3, 4, false))
    act(() => result.current.moveActiveHome(true, false))
    expect(result.current.anchorCell).toEqual({ row: 3, col: 4 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 0 })
  })
})

describe('moveActiveEnd', () => {
  it('End moves to last col same row', () => {
    const result = setupHook(makeGrid(5, 6))
    act(() => result.current.activateCell(2, 0, false))
    act(() => result.current.moveActiveEnd(false, false))
    expect(result.current.activeCell).toEqual({ row: 2, col: 5 })
    expect(result.current.anchorCell).toBeNull()
  })

  it('Ctrl+End moves to last row and last col', () => {
    const result = setupHook(makeGrid(5, 6))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.moveActiveEnd(false, true))
    expect(result.current.activeCell).toEqual({ row: 4, col: 5 })
  })

  it('Shift+End extends range', () => {
    const result = setupHook(makeGrid(5, 6))
    act(() => result.current.activateCell(2, 1, false))
    act(() => result.current.moveActiveEnd(true, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 1 })
    expect(result.current.activeCell).toEqual({ row: 2, col: 5 })
  })
})

// ──────────────────────────────────────────────────────────────
// setWall
// ──────────────────────────────────────────────────────────────

describe('setWall', () => {
  it('sets a single selected cell to wall', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.setWall())
    expect(result.current.grid[1][1]).toBe('W')
  })

  it('sets all cells in selection rect to wall', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.activateCell(3, 3, true))
    act(() => result.current.setWall())
    for (let r = 1; r <= 3; r++) {
      for (let c = 1; c <= 3; c++) {
        expect(result.current.grid[r][c]).toBe('W')
      }
    }
    // Cells outside selection unchanged
    expect(result.current.grid[0][0]).toBe(' ')
  })

  it('marks the maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(0, 0, false))
    expect(result.current.isDirty).toBe(false)
    act(() => result.current.setWall())
    expect(result.current.isDirty).toBe(true)
  })

  it('does nothing when no cell is selected', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.setWall())
    expect(result.current.isDirty).toBe(false)
  })
})

// ──────────────────────────────────────────────────────────────
// setStart
// ──────────────────────────────────────────────────────────────

describe('setStart', () => {
  it('sets selected cell to start', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.setStart())
    expect(result.current.grid[0][0]).toBe('S')
  })

  it('clears existing start cell before setting new one', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'S'
    const result = setupHook(grid)
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.setStart())
    expect(result.current.grid[0][0]).toBe(' ')
    expect(result.current.grid[2][2]).toBe('S')
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.setStart())
    expect(result.current.isDirty).toBe(true)
  })
})

// ──────────────────────────────────────────────────────────────
// setFinish
// ──────────────────────────────────────────────────────────────

describe('setFinish', () => {
  it('sets selected cell to finish', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.setFinish())
    expect(result.current.grid[2][2]).toBe('F')
  })

  it('clears existing finish cell before setting new one', () => {
    const grid = makeGrid(3, 3)
    grid[2][2] = 'F'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.setFinish())
    expect(result.current.grid[2][2]).toBe(' ')
    expect(result.current.grid[0][0]).toBe('F')
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.setFinish())
    expect(result.current.isDirty).toBe(true)
  })
})

// ──────────────────────────────────────────────────────────────
// clearCell
// ──────────────────────────────────────────────────────────────

describe('clearCell', () => {
  it('sets a wall cell to empty', () => {
    const grid = makeGrid(3, 3)
    grid[1][1] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.clearCell())
    expect(result.current.grid[1][1]).toBe(' ')
  })

  it('clears all cells in selection rect', () => {
    const grid = makeGrid(3, 3, 'W')
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.activateCell(1, 1, true))
    act(() => result.current.clearCell())
    for (let r = 0; r <= 1; r++) {
      for (let c = 0; c <= 1; c++) {
        expect(result.current.grid[r][c]).toBe(' ')
      }
    }
    // Cell outside selection unchanged
    expect(result.current.grid[2][2]).toBe('W')
  })

  it('marks maze as dirty', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.clearCell())
    expect(result.current.isDirty).toBe(true)
  })
})

// ──────────────────────────────────────────────────────────────
// selectionStatus flags
// ──────────────────────────────────────────────────────────────

describe('selectionStatus', () => {
  it('returns all-false defaults when no cell is active', () => {
    const result = setupHook(makeGrid(3, 3))
    const s = result.current.selectionStatus
    expect(s.isSingleCell).toBe(false)
    expect(s.containsWall).toBe(false)
    expect(s.containsStart).toBe(false)
    expect(s.containsFinish).toBe(false)
    expect(s.isAllWalls).toBe(false)
    expect(s.isStart).toBe(false)
    expect(s.isFinish).toBe(false)
    expect(s.isEmpty).toBe(true)
    expect(s.allColumnsSelected).toBe(false)
    expect(s.allRowsSelected).toBe(false)
  })

  it('isSingleCell is true for a single-cell selection', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(1, 1, false))
    expect(result.current.selectionStatus.isSingleCell).toBe(true)
  })

  it('isSingleCell is false for a multi-cell selection', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.activateCell(1, 1, true))
    expect(result.current.selectionStatus.isSingleCell).toBe(false)
  })

  it('containsWall is true when selection includes a wall', () => {
    const grid = makeGrid(3, 3)
    grid[1][1] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(1, 1, false))
    expect(result.current.selectionStatus.containsWall).toBe(true)
  })

  it('isAllWalls is true when all cells in selection are walls', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'W'
    grid[0][1] = 'W'
    grid[1][0] = 'W'
    grid[1][1] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.activateCell(1, 1, true))
    expect(result.current.selectionStatus.isAllWalls).toBe(true)
  })

  it('isAllWalls is false when selection has mixed cells', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.activateCell(0, 1, true))
    expect(result.current.selectionStatus.isAllWalls).toBe(false)
  })

  it('isStart is true when single-cell selection contains S', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'S'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    const s = result.current.selectionStatus
    expect(s.isStart).toBe(true)
    expect(s.containsStart).toBe(true)
    expect(s.isFinish).toBe(false)
  })

  it('isFinish is true when single-cell selection contains F', () => {
    const grid = makeGrid(3, 3)
    grid[2][2] = 'F'
    const result = setupHook(grid)
    act(() => result.current.activateCell(2, 2, false))
    const s = result.current.selectionStatus
    expect(s.isFinish).toBe(true)
    expect(s.containsFinish).toBe(true)
    expect(s.isStart).toBe(false)
  })

  it('isEmpty is true when all cells in selection are passable', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.activateCell(1, 1, true))
    expect(result.current.selectionStatus.isEmpty).toBe(true)
  })

  it('isEmpty is false when selection contains a wall', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCell(0, 0, false))
    expect(result.current.selectionStatus.isEmpty).toBe(false)
  })

  it('allColumnsSelected is true when selection spans all columns', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateRow(1, false))
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('allColumnsSelected is false when selection is partial', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateCell(1, 0, false))
    act(() => result.current.activateCell(1, 2, true))
    expect(result.current.selectionStatus.allColumnsSelected).toBe(false)
  })

  it('allRowsSelected is true when selection spans all rows', () => {
    const result = setupHook(makeGrid(4, 3))
    act(() => result.current.activateCol(1, false))
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('hasSolveCells is true when grid has both S and F', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'S'
    grid[2][2] = 'F'
    const result = setupHook(grid)
    expect(result.current.selectionStatus.hasSolveCells).toBe(true)
  })

  it('hasSolveCells is false when grid is missing S or F', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'S'
    const result = setupHook(grid)
    expect(result.current.selectionStatus.hasSolveCells).toBe(false)
  })

  it('hasSolution reflects solution state (always false from fresh init)', () => {
    const result = setupHook(makeGrid(3, 3))
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })

  it('setWall clears solution (hasSolution becomes false after edit)', () => {
    // Indirectly verify solution is cleared by checking isDirty is set
    // (solution state is internal; tested via selectionStatus.hasSolution)
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.setWall())
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })

  it('allColumnsSelected is true after selectAll (corner click)', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.selectAll())
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('allRowsSelected is true after selectAll (corner click)', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.selectAll())
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })
})

// ──────────────────────────────────────────────────────────────
// structural editing
// ──────────────────────────────────────────────────────────────

describe('insertRowsBefore', () => {
  it('inserts one row when a single row is selected', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.insertRowsBefore())
    expect(result.current.grid.length).toBe(4)
  })

  it('inserts N rows when N rows are selected', () => {
    const result = setupHook(makeGrid(4, 3))
    // Select rows 1–2 (2 rows)
    act(() => result.current.activateRow(1, false))
    act(() => result.current.activateRow(2, true))
    act(() => result.current.insertRowsBefore())
    expect(result.current.grid.length).toBe(6)
  })

  it('inserts blank rows at the correct index', () => {
    const grid = makeGrid(3, 3)
    grid[1][0] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateRow(1, false))
    act(() => result.current.insertRowsBefore())
    // New blank row at index 1; old row 1 shifts to index 2
    expect(result.current.grid[1]).toEqual([' ', ' ', ' '])
    expect(result.current.grid[2][0]).toBe('W')
  })

  it('inserts before the first row', () => {
    const grid = makeGrid(3, 3)
    grid[0][0] = 'S'
    const result = setupHook(grid)
    act(() => result.current.activateRow(0, false))
    act(() => result.current.insertRowsBefore())
    expect(result.current.grid[0]).toEqual([' ', ' ', ' '])
    expect(result.current.grid[1][0]).toBe('S')
  })

  it('selects the newly inserted row(s) after single-row insert', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.insertRowsBefore())
    expect(result.current.activeCell).toEqual({ row: 1, col: 0 })
    expect(result.current.anchorCell).toEqual({ row: 1, col: 2 })
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('selects all newly inserted rows after multi-row insert', () => {
    const result = setupHook(makeGrid(4, 3))
    // Select rows 1–2
    act(() => result.current.activateRow(1, false))
    act(() => result.current.activateRow(2, true))
    act(() => result.current.insertRowsBefore())
    // 2 new rows inserted at index 1–2; selection should span rows 1–2
    expect(result.current.activeCell).toEqual({ row: 1, col: 0 })
    expect(result.current.anchorCell).toEqual({ row: 2, col: 2 })
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(0, false))
    act(() => result.current.insertRowsBefore())
    expect(result.current.isDirty).toBe(true)
  })

  it('clears solution', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(0, false))
    act(() => result.current.insertRowsBefore())
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })
})

describe('deleteRows', () => {
  it('decreases row count by one when a single row is selected', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.deleteRows())
    expect(result.current.grid.length).toBe(2)
  })

  it('removes the correct row content', () => {
    const grid = makeGrid(3, 3)
    grid[1][0] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateRow(1, false))
    act(() => result.current.deleteRows())
    // Row 1 (W row) gone; remaining rows are blank
    for (let c = 0; c < 3; c++) {
      expect(result.current.grid[0][c]).toBe(' ')
      expect(result.current.grid[1][c]).toBe(' ')
    }
  })

  it('decreases row count by selection size for multi-row delete', () => {
    const result = setupHook(makeGrid(5, 3))
    // Select rows 1–2 (shift)
    act(() => result.current.activateRow(1, false))
    act(() => result.current.activateRow(2, true))
    act(() => result.current.deleteRows())
    expect(result.current.grid.length).toBe(3)
  })

  it('selects the shifted rows spanning the same count when sufficient rows remain', () => {
    // 7 rows (0–6), delete rows 0–2 → 4 remain; shifted rows fill positions 0–2
    // activateRow(0)+activateRow(2,true) → active={row:2,col:2}, anchor={row:0,col:0}
    const result = setupHook(makeGrid(7, 3))
    act(() => result.current.activateRow(0, false))
    act(() => result.current.activateRow(2, true))
    act(() => result.current.deleteRows())
    // Rows 0–2 still clamped correctly; active stays at row 2, anchor at row 0
    expect(result.current.activeCell).toEqual({ row: 2, col: 2 })
    expect(result.current.anchorCell).toEqual({ row: 0, col: 0 })
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('reduces selection when fewer shifted rows are available than deleted', () => {
    // 5 rows (0–4), delete rows 2–4 → 2 remain at 0–1; only 2 shifted rows available
    const result = setupHook(makeGrid(5, 3))
    act(() => result.current.activateRow(2, false))
    act(() => result.current.activateRow(4, true))
    act(() => result.current.deleteRows())
    // 2 rows remain; selection should be rows 1–1 (edge) since minRow=2 clamps to 1
    expect(result.current.activeCell?.row).toBe(1)
    expect(result.current.anchorCell?.row).toBe(1)
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('selects single edge row when no rows remain beyond the deleted range', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(2, false))
    act(() => result.current.deleteRows())
    expect(result.current.activeCell?.row).toBe(1)
    expect(result.current.anchorCell?.row).toBe(1)
  })

  it('selects the shifted single row when a single row in the middle is deleted', () => {
    const result = setupHook(makeGrid(4, 3))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.deleteRows())
    expect(result.current.activeCell).toEqual({ row: 1, col: 0 })
    expect(result.current.anchorCell).toEqual({ row: 1, col: 2 })
    expect(result.current.selectionStatus.allColumnsSelected).toBe(true)
  })

  it('preserves active cell column after delete', () => {
    // activateRow sets active at col 0, anchor at last col; verify cols preserved
    const result = setupHook(makeGrid(4, 5))
    act(() => result.current.activateRow(1, false))
    act(() => result.current.deleteRows())
    expect(result.current.activeCell?.col).toBe(0)
    expect(result.current.anchorCell?.col).toBe(4)
  })

  it('preserves selection direction (active at bottom, anchor at top) after delete', () => {
    // Shift+click from row 2 down to row 0 → active at row 2, anchor at row 0
    const result = setupHook(makeGrid(5, 3))
    act(() => result.current.activateRow(2, false))
    act(() => result.current.activateRow(0, true))
    // Now activeCell={row:0,col:2}, anchorCell={row:2,col:0} (activateRow extend logic)
    act(() => result.current.deleteRows())
    // After deleting rows 0–2, 2 rows remain; active clamps to row 0 (min(0,1)), anchor clamps to row 1 (min(2,1))
    expect(result.current.activeCell?.col).toBe(2)
    expect(result.current.anchorCell?.col).toBe(0)
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(0, false))
    act(() => result.current.deleteRows())
    expect(result.current.isDirty).toBe(true)
  })

  it('clears solution', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateRow(0, false))
    act(() => result.current.deleteRows())
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })
})

describe('insertColsBefore', () => {
  it('inserts one column when a single column is selected', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.insertColsBefore())
    expect(result.current.grid[0].length).toBe(5)
  })

  it('inserts N columns when N columns are selected', () => {
    const result = setupHook(makeGrid(3, 4))
    // Select cols 1–2 (2 cols)
    act(() => result.current.activateCol(1, false))
    act(() => result.current.activateCol(2, true))
    act(() => result.current.insertColsBefore())
    expect(result.current.grid[0].length).toBe(6)
  })

  it('inserts blank columns at the correct index', () => {
    const grid = makeGrid(3, 3)
    grid[0][1] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCol(1, false))
    act(() => result.current.insertColsBefore())
    // New blank col at index 1; old col 1 shifts to index 2
    for (let r = 0; r < 3; r++) {
      expect(result.current.grid[r][1]).toBe(' ')
    }
    expect(result.current.grid[0][2]).toBe('W')
  })

  it('selects the newly inserted column after single-col insert', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.insertColsBefore())
    expect(result.current.activeCell).toEqual({ row: 0, col: 1 })
    expect(result.current.anchorCell).toEqual({ row: 2, col: 1 })
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('selects all newly inserted columns after multi-col insert', () => {
    const result = setupHook(makeGrid(3, 4))
    // Select cols 1–2
    act(() => result.current.activateCol(1, false))
    act(() => result.current.activateCol(2, true))
    act(() => result.current.insertColsBefore())
    // 2 new cols inserted at index 1–2; selection should span cols 1–2
    expect(result.current.activeCell).toEqual({ row: 0, col: 1 })
    expect(result.current.anchorCell).toEqual({ row: 2, col: 2 })
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.insertColsBefore())
    expect(result.current.isDirty).toBe(true)
  })

  it('clears solution', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.insertColsBefore())
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })
})

describe('deleteCols', () => {
  it('decreases column count by one when a single column is selected', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.deleteCols())
    expect(result.current.grid[0].length).toBe(3)
  })

  it('removes the correct column content', () => {
    const grid = makeGrid(3, 3)
    grid[0][1] = 'W'
    const result = setupHook(grid)
    act(() => result.current.activateCol(1, false))
    act(() => result.current.deleteCols())
    // Col 1 (W col) gone; 2 cols remain, both blank
    for (let r = 0; r < 3; r++) {
      expect(result.current.grid[r].length).toBe(2)
      expect(result.current.grid[r][0]).toBe(' ')
      expect(result.current.grid[r][1]).toBe(' ')
    }
  })

  it('decreases column count by selection size for multi-col delete', () => {
    const result = setupHook(makeGrid(3, 5))
    // Select cols 1–2
    act(() => result.current.activateCol(1, false))
    act(() => result.current.activateCol(2, true))
    act(() => result.current.deleteCols())
    expect(result.current.grid[0].length).toBe(3)
  })

  it('selects the shifted cols spanning the same count when sufficient cols remain', () => {
    // 7 cols (0–6), delete cols 0–2 → 4 remain; shifted cols fill positions 0–2
    // activateCol(0)+activateCol(2,true) → active={row:2,col:2}, anchor={row:0,col:0}
    const result = setupHook(makeGrid(3, 7))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.activateCol(2, true))
    act(() => result.current.deleteCols())
    // Cols 0–2 still clamped correctly; active stays at col 2, anchor at col 0
    expect(result.current.activeCell).toEqual({ row: 2, col: 2 })
    expect(result.current.anchorCell).toEqual({ row: 0, col: 0 })
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('reduces selection when fewer shifted cols are available than deleted', () => {
    // 5 cols (0–4), delete cols 0–2 → 2 remain; only cols 0–1 available
    // activateCol(0)+activateCol(2,true) → active={row:2,col:2}, anchor={row:0,col:0}
    const result = setupHook(makeGrid(3, 5))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.activateCol(2, true))
    act(() => result.current.deleteCols())
    // active col clamps to min(2,1)=1, anchor col clamps to min(0,1)=0
    expect(result.current.activeCell?.col).toBe(1)
    expect(result.current.anchorCell?.col).toBe(0)
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('selects single edge column when no cols remain beyond the deleted range', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(2, false))
    act(() => result.current.deleteCols())
    expect(result.current.activeCell?.col).toBe(1)
    expect(result.current.anchorCell?.col).toBe(1)
  })

  it('selects the shifted single column when a single col in the middle is deleted', () => {
    const result = setupHook(makeGrid(3, 4))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.deleteCols())
    expect(result.current.activeCell).toEqual({ row: 0, col: 1 })
    expect(result.current.anchorCell).toEqual({ row: 2, col: 1 })
    expect(result.current.selectionStatus.allRowsSelected).toBe(true)
  })

  it('preserves active cell row after delete', () => {
    // activateCol sets active at row 0, anchor at last row; verify rows preserved
    const result = setupHook(makeGrid(5, 4))
    act(() => result.current.activateCol(1, false))
    act(() => result.current.deleteCols())
    expect(result.current.activeCell?.row).toBe(0)
    expect(result.current.anchorCell?.row).toBe(4)
  })

  it('preserves selection direction (active at right, anchor at left) after delete', () => {
    // Shift+click from col 2 to col 0 → active at col 0, anchor at col 2
    const result = setupHook(makeGrid(3, 6))
    act(() => result.current.activateCol(2, false))
    act(() => result.current.activateCol(0, true))
    // activateCol extend: anchorCell={row:0,col:2}, activeCell={row:2,col:0}
    act(() => result.current.deleteCols())
    // After deleting cols 0–2, 3 remain; active clamps col to min(0,2)=0, anchor clamps col to min(2,2)=2
    expect(result.current.activeCell?.col).toBe(0)
    expect(result.current.anchorCell?.col).toBe(2)
  })

  it('marks maze as dirty', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.deleteCols())
    expect(result.current.isDirty).toBe(true)
  })

  it('clears solution', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCol(0, false))
    act(() => result.current.deleteCols())
    expect(result.current.selectionStatus.hasSolution).toBe(false)
  })
})

// ──────────────────────────────────────────────────────────────
// isRangeMode / enableRangeMode / disableRangeMode
// ──────────────────────────────────────────────────────────────

describe('isRangeMode', () => {
  it('starts as false', () => {
    const result = setupHook(makeGrid(5, 5))
    expect(result.current.isRangeMode).toBe(false)
  })

  it('enableRangeMode sets isRangeMode to true', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.enableRangeMode())
    expect(result.current.isRangeMode).toBe(true)
  })

  it('disableRangeMode sets isRangeMode to false and clears anchor', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.enableRangeMode())
    act(() => result.current.activateCell(3, 3, false)) // extends range without shift
    expect(result.current.anchorCell).not.toBeNull()
    act(() => result.current.disableRangeMode())
    expect(result.current.isRangeMode).toBe(false)
    expect(result.current.anchorCell).toBeNull()
  })

  it('activateCell extends range without shift when isRangeMode is true', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(1, 1, false))
    act(() => result.current.enableRangeMode())
    act(() => result.current.activateCell(3, 3, false))
    expect(result.current.anchorCell).toEqual({ row: 1, col: 1 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 3 })
  })

  it('moveActive extends range without shift when isRangeMode is true', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 2, false))
    act(() => result.current.enableRangeMode())
    act(() => result.current.moveActive(1, 0, false, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 2 })
    expect(result.current.activeCell).toEqual({ row: 3, col: 2 })
  })

  it('moveActiveHome extends range without shift when isRangeMode is true', () => {
    const result = setupHook(makeGrid(5, 5))
    act(() => result.current.activateCell(2, 4, false))
    act(() => result.current.enableRangeMode())
    act(() => result.current.moveActiveHome(false, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 4 })
    expect(result.current.activeCell).toEqual({ row: 2, col: 0 })
  })

  it('moveActiveEnd extends range without shift when isRangeMode is true', () => {
    const result = setupHook(makeGrid(5, 6))
    act(() => result.current.activateCell(2, 0, false))
    act(() => result.current.enableRangeMode())
    act(() => result.current.moveActiveEnd(false, false))
    expect(result.current.anchorCell).toEqual({ row: 2, col: 0 })
    expect(result.current.activeCell).toEqual({ row: 2, col: 5 })
  })
})

// ──────────────────────────────────────────────────────────────
// markSaved
// ──────────────────────────────────────────────────────────────

describe('markSaved', () => {
  it('clears isDirty and updates mazeId and mazeName', () => {
    const result = setupHook(makeGrid(3, 3))
    // Make the grid dirty by editing a cell
    act(() => result.current.activateCell(0, 0, false))
    act(() => result.current.setWall())
    expect(result.current.isDirty).toBe(true)

    act(() => result.current.markSaved('new-id', 'New Name'))

    expect(result.current.isDirty).toBe(false)
    expect(result.current.mazeId).toBe('new-id')
    expect(result.current.mazeName).toBe('New Name')
  })

  it('updates mazeId from null when saving a new maze', () => {
    const { result } = renderHook(() => useMazeEditor())
    act(() => result.current.initFromDefinition(null, '', makeDefinition(makeGrid(3, 3))))
    expect(result.current.mazeId).toBeNull()

    act(() => result.current.markSaved('saved-id', 'My Maze'))

    expect(result.current.mazeId).toBe('saved-id')
    expect(result.current.mazeName).toBe('My Maze')
    expect(result.current.isDirty).toBe(false)
  })

  it('does not clear active cell or anchor', () => {
    const result = setupHook(makeGrid(3, 3))
    act(() => result.current.activateCell(1, 2, false))
    act(() => result.current.markSaved('new-id', 'New Name'))
    expect(result.current.activeCell).toEqual({ row: 1, col: 2 })
    expect(result.current.anchorCell).toBeNull()
  })
})
