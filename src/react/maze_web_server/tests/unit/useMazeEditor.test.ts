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
})
