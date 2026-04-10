import { forwardRef, useMemo, Fragment } from 'react'
import type { CellPoint } from '../hooks/useMazeEditor'

export const CELL_SIZE = 32
export const HEADER_SIZE = 24

interface MazeGridProps {
  grid: string[][]
  solution: Array<CellPoint> | null
  activeCell: CellPoint | null
  anchorCell: CellPoint | null
  onCellClick?: (row: number, col: number, shift: boolean) => void
  onRowHeaderClick?: (row: number, shift: boolean) => void
  onColHeaderClick?: (col: number, shift: boolean) => void
  onKeyDown?: (e: React.KeyboardEvent) => void
}

function cellImage(cell: string): { src: string; alt: string } | null {
  if (cell === 'W') return { src: '/images/maze/wall.png', alt: 'Wall' }
  if (cell === 'S') return { src: '/images/maze/start_flag.png', alt: 'Start' }
  if (cell === 'F') return { src: '/images/maze/finish_flag.png', alt: 'Finish' }
  return null
}

const FOOTSTEP_IMAGES: Record<string, string> = {
  up: '/images/maze/footsteps_up.png',
  down: '/images/maze/footsteps_down.png',
  left: '/images/maze/footsteps_left.png',
  right: '/images/maze/footsteps_right.png',
}

function buildSolutionMap(solution: Array<CellPoint>): Map<string, string> {
  const map = new Map<string, string>()
  for (let i = 0; i < solution.length; i++) {
    const curr = solution[i]
    const next = solution[i + 1]
    let dir: string
    if (next) {
      if (next.row < curr.row) dir = 'up'
      else if (next.row > curr.row) dir = 'down'
      else if (next.col < curr.col) dir = 'left'
      else dir = 'right'
    } else {
      const prev = solution[i - 1]
      if (prev) {
        if (curr.row < prev.row) dir = 'up'
        else if (curr.row > prev.row) dir = 'down'
        else if (curr.col < prev.col) dir = 'left'
        else dir = 'right'
      } else {
        dir = 'right'
      }
    }
    map.set(`${curr.row},${curr.col}`, FOOTSTEP_IMAGES[dir])
  }
  return map
}

export const MazeGrid = forwardRef<HTMLDivElement, MazeGridProps>(
  function MazeGrid(
    { grid, solution, activeCell, anchorCell, onCellClick, onRowHeaderClick, onColHeaderClick, onKeyDown },
    ref,
  ) {
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0

    const solutionMap = useMemo(
      () => (solution ? buildSolutionMap(solution) : new Map<string, string>()),
      [solution],
    )

    const selectionRect = useMemo(() => {
      if (!activeCell) return null
      if (!anchorCell) {
        return { minRow: activeCell.row, maxRow: activeCell.row, minCol: activeCell.col, maxCol: activeCell.col }
      }
      return {
        minRow: Math.min(activeCell.row, anchorCell.row),
        maxRow: Math.max(activeCell.row, anchorCell.row),
        minCol: Math.min(activeCell.col, anchorCell.col),
        maxCol: Math.max(activeCell.col, anchorCell.col),
      }
    }, [activeCell, anchorCell])

    const frameStyle = useMemo(() => {
      if (!selectionRect) return null
      return {
        top: HEADER_SIZE + selectionRect.minRow * CELL_SIZE,
        left: HEADER_SIZE + selectionRect.minCol * CELL_SIZE,
        width: (selectionRect.maxCol - selectionRect.minCol + 1) * CELL_SIZE,
        height: (selectionRect.maxRow - selectionRect.minRow + 1) * CELL_SIZE,
      }
    }, [selectionRect])

    function getCellClasses(row: number, col: number): string {
      const classes = ['maze-cell']

      if (activeCell) {
        const isActive = row === activeCell.row && col === activeCell.col
        const isAnchor = anchorCell != null && row === anchorCell.row && col === anchorCell.col

        if (isActive) {
          classes.push('maze-cell--active')
        } else if (isAnchor) {
          classes.push('maze-cell--anchor')
        } else if (
          selectionRect &&
          row >= selectionRect.minRow && row <= selectionRect.maxRow &&
          col >= selectionRect.minCol && col <= selectionRect.maxCol
        ) {
          classes.push('maze-cell--in-range')
        }
      }

      if (solutionMap.has(`${row},${col}`)) {
        classes.push('maze-cell--solution')
      }

      return classes.join(' ')
    }

    return (
      <div
        className="maze-grid-container"
        ref={ref}
        tabIndex={0}
        onKeyDown={onKeyDown}
        aria-label="Maze grid"
      >
        <div
          className="maze-grid"
          style={{ '--cols': cols } as React.CSSProperties}
        >
          {/* Corner */}
          <div className="maze-cell-corner" />

          {/* Column headers */}
          {Array.from({ length: cols }, (_, c) => (
            <div
              key={`col-${c}`}
              className="maze-cell-col-header"
              onClick={e => onColHeaderClick?.(c, e.shiftKey)}
              aria-label={`Column ${c + 1}`}
            >
              {c + 1}
            </div>
          ))}

          {/* Rows */}
          {Array.from({ length: rows }, (_, r) => (
            <Fragment key={`row-${r}`}>
              <div
                className="maze-cell-row-header"
                onClick={e => onRowHeaderClick?.(r, e.shiftKey)}
                aria-label={`Row ${r + 1}`}
              >
                {r + 1}
              </div>
              {Array.from({ length: cols }, (_, c) => {
                const cell = grid[r]?.[c] ?? ' '
                const img = cellImage(cell)
                const solutionImgSrc = solutionMap.get(`${r},${c}`)
                return (
                  <div
                    key={`cell-${r}-${c}`}
                    className={getCellClasses(r, c)}
                    onClick={e => onCellClick?.(r, c, e.shiftKey)}
                    aria-label={`Cell ${r + 1},${c + 1}`}
                  >
                    {img && <img src={img.src} alt={img.alt} />}
                    {solutionImgSrc && (
                      <img src={solutionImgSrc} alt="Solution path" className="maze-cell-solution-img" />
                    )}
                  </div>
                )
              })}
            </Fragment>
          ))}
        </div>

        {frameStyle && (
          <div
            className="maze-selection-frame"
            style={frameStyle}
            aria-hidden="true"
          />
        )}
      </div>
    )
  },
)
