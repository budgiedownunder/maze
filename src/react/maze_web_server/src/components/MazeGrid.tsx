import { forwardRef, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import type { CellPoint } from '../hooks/useMazeEditor'

export const CELL_SIZE = 32
export const HEADER_SIZE = 24

interface MazeGridProps {
  grid: string[][]
  solution: Array<CellPoint> | null
  activeCell: CellPoint | null
  anchorCell: CellPoint | null
  isRangeMode?: boolean
  onCellClick?: (row: number, col: number, shift: boolean) => void
  onCellDoubleClick?: (row: number, col: number) => void
  onRowHeaderClick?: (row: number, shift: boolean) => void
  onColHeaderClick?: (col: number, shift: boolean) => void
  onCornerClick?: () => void
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
    { grid, solution, activeCell, anchorCell, isRangeMode = false, onCellClick, onCellDoubleClick, onRowHeaderClick, onColHeaderClick, onCornerClick, onKeyDown },
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

    // Internal ref for DOM measurement; merged with the forwarded ref below.
    const containerRef = useRef<HTMLDivElement>(null)
    const setContainerRef = useCallback(
      (node: HTMLDivElement | null) => {
        (containerRef as React.RefObject<HTMLDivElement | null>).current = node
        if (typeof ref === 'function') ref(node)
        else if (ref) (ref as React.RefObject<HTMLDivElement | null>).current = node
      },
      [ref],
    )

    // Frame position is measured from the DOM so it stays correct at any zoom level.
    const [frameStyle, setFrameStyle] = useState<{ top: number; left: number; width: number; height: number } | null>(null)

    // Keep a ref so the stable measureFrame callback can read the latest selectionRect
    // without being recreated every time it changes.
    const selectionRectRef = useRef(selectionRect)
    selectionRectRef.current = selectionRect

    // Stable measurement function — no deps, safe to subscribe to window events.
    const measureFrame = useCallback(() => {
      const sr = selectionRectRef.current
      if (!sr || !containerRef.current) {
        setFrameStyle(null)
        return
      }

      const container = containerRef.current
      const containerRect = container.getBoundingClientRect()

      // JSDOM (unit tests) always returns zero-sized rects — fall back to calculated values.
      if (containerRect.width === 0) {
        setFrameStyle({
          top: HEADER_SIZE + sr.minRow * CELL_SIZE,
          left: HEADER_SIZE + sr.minCol * CELL_SIZE,
          width: (sr.maxCol - sr.minCol + 1) * CELL_SIZE,
          height: (sr.maxRow - sr.minRow + 1) * CELL_SIZE,
        })
        return
      }

      const tlCell = container.querySelector<HTMLElement>(
        `td[aria-label="Cell ${sr.minRow + 1},${sr.minCol + 1}"]`,
      )
      const brCell = container.querySelector<HTMLElement>(
        `td[aria-label="Cell ${sr.maxRow + 1},${sr.maxCol + 1}"]`,
      )
      if (!tlCell || !brCell) return

      const tlRect = tlCell.getBoundingClientRect()
      const brRect = brCell.getBoundingClientRect()

      // getBoundingClientRect is viewport-relative (accounts for current scroll), but
      // position:absolute is relative to the container's padding edge (inside its border,
      // before scroll). Subtract border widths and add scroll to convert correctly.
      setFrameStyle({
        top: tlRect.top - containerRect.top - container.clientTop + container.scrollTop,
        left: tlRect.left - containerRect.left - container.clientLeft + container.scrollLeft,
        width: brRect.right - tlRect.left,
        height: brRect.bottom - tlRect.top,
      })
    }, []) // stable — reads selectionRect via ref

    // Re-measure whenever the selection changes.
    useLayoutEffect(() => {
      measureFrame()
    }, [selectionRect, measureFrame])

    // Re-measure on window resize (browser zoom fires resize), so the frame stays
    // aligned after the user zooms without needing to reselect.
    useEffect(() => {
      window.addEventListener('resize', measureFrame)
      return () => window.removeEventListener('resize', measureFrame)
    }, [measureFrame])

    // Scroll the active cell into view whenever it changes (arrow keys, Home/End, etc.).
    //
    // UP/LEFT — computed integers: scrollTop = cellTop - HEADER_SIZE.
    //   "cellTop" is an exact integer (HEADER_SIZE + row * CELL_SIZE), so there is no
    //   floating-point drift. scrollIntoView cannot be used here because it measures the
    //   actual sub-pixel cell position (e.g. 24.5px at 110% zoom) and subtracts the
    //   scroll-margin, producing a fractional scrollTop (e.g. 0.5 → rounds to 1).
    //   That 1px offset puts the top ants line under the sticky header.
    //
    // DOWN/RIGHT — scrollIntoView({ block/inline: 'end' }): the browser uses the actual
    //   rendered cell bottom/right, which accounts for sub-pixel accumulation across many
    //   rows/cols at non-integer zoom levels. CSS scroll-margin-bottom/right: 2px ensures
    //   the frame's ants line is never flush with (and clipped by) the container edge.
    useLayoutEffect(() => {
      if (!activeCell || !containerRef.current) return
      const container = containerRef.current
      if (container.clientWidth === 0) return // JSDOM — no layout

      const cellTop  = HEADER_SIZE + activeCell.row * CELL_SIZE
      const cellLeft = HEADER_SIZE + activeCell.col * CELL_SIZE

      if (cellTop  < container.scrollTop  + HEADER_SIZE) container.scrollTop  = cellTop  - HEADER_SIZE
      if (cellLeft < container.scrollLeft + HEADER_SIZE) container.scrollLeft = cellLeft - HEADER_SIZE

      const needDown  = cellTop  + CELL_SIZE > container.scrollTop  + container.clientHeight
      const needRight = cellLeft + CELL_SIZE > container.scrollLeft + container.clientWidth
      if (needDown || needRight) {
        container
          .querySelector<HTMLElement>(`td[aria-label="Cell ${activeCell.row + 1},${activeCell.col + 1}"]`)
          ?.scrollIntoView({
            block:  needDown  ? 'end' : 'nearest',
            inline: needRight ? 'end' : 'nearest',
          })
      }
    }, [activeCell])

    function getCellClasses(row: number, col: number): string {
      const classes = ['maze-cell']
      const inSolution = solutionMap.has(`${row},${col}`)

      // Only apply selection highlights to cells not in the solution path —
      // solution styling takes visual precedence over the active/range highlight.
      if (activeCell && !inSolution) {
        const isActive = row === activeCell.row && col === activeCell.col
        const isAnchor = anchorCell != null && row === anchorCell.row && col === anchorCell.col

        if (isAnchor) {
          classes.push('maze-cell--anchor')
        } else if (isActive) {
          // No anchor = single selection (acts as anchor origin) → yellow
          // With anchor = moving end of range → in-range color
          classes.push(anchorCell === null ? 'maze-cell--anchor' : 'maze-cell--active')
        } else if (
          selectionRect &&
          row >= selectionRect.minRow && row <= selectionRect.maxRow &&
          col >= selectionRect.minCol && col <= selectionRect.maxCol
        ) {
          classes.push('maze-cell--in-range')
        }
      }

      if (inSolution && grid[row][col] !== 'S' && grid[row][col] !== 'F') {
        classes.push('maze-cell--solution')
      }

      return classes.join(' ')
    }

    const isAllSelected =
      selectionRect !== null &&
      selectionRect.minRow === 0 && selectionRect.maxRow === rows - 1 &&
      selectionRect.minCol === 0 && selectionRect.maxCol === cols - 1

    return (
      <div
        className="maze-grid-container"
        ref={setContainerRef}
        tabIndex={0}
        onKeyDown={onKeyDown}
        aria-label="Maze grid"
      >
        <table className="maze-grid">
          <thead>
            <tr>
              <th
                className={`maze-cell-corner${isAllSelected ? ' maze-cell-corner--selected' : ''}`}
                onClick={() => onCornerClick?.()}
                aria-label="Select all"
              />
              {Array.from({ length: cols }, (_, c) => {
                const isColSelected = selectionRect !== null &&
                  c >= selectionRect.minCol && c <= selectionRect.maxCol
                return (
                <th
                  key={`col-${c}`}
                  scope="col"
                  className={`maze-cell-col-header${isColSelected ? ' maze-cell-col-header--selected' : ''}`}
                  onClick={e => onColHeaderClick?.(c, e.shiftKey)}
                  aria-label={`Column ${c + 1}`}
                >
                  {c + 1}
                </th>
              )})}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: rows }, (_, r) => {
              const isRowSelected = selectionRect !== null &&
                r >= selectionRect.minRow && r <= selectionRect.maxRow
              return (
              <tr key={`row-${r}`}>
                <th
                  scope="row"
                  className={`maze-cell-row-header${isRowSelected ? ' maze-cell-row-header--selected' : ''}`}
                  onClick={e => onRowHeaderClick?.(r, e.shiftKey)}
                  aria-label={`Row ${r + 1}`}
                >
                  {r + 1}
                </th>
                {Array.from({ length: cols }, (_, c) => {
                  const cell = grid[r]?.[c] ?? ' '
                  const img = cellImage(cell)
                  const solutionImgSrc = solutionMap.get(`${r},${c}`)
                  return (
                    <td
                      key={`cell-${r}-${c}`}
                      className={getCellClasses(r, c)}
                      onClick={e => onCellClick?.(r, c, e.shiftKey)}
                      onDoubleClick={() => onCellDoubleClick?.(r, c)}
                      aria-label={`Cell ${r + 1},${c + 1}`}
                    >
                      {img && <img src={img.src} alt={img.alt} />}
                      {solutionImgSrc && cell !== 'S' && cell !== 'F' && (
                        <img src={solutionImgSrc} alt="Solution path" className="maze-cell-solution-img" />
                      )}
                    </td>
                  )
                })}
              </tr>
              )
            })}
          </tbody>
        </table>

        {frameStyle && (
          <div
            className={`maze-selection-frame${
              !isRangeMode &&
              selectionRect !== null &&
              selectionRect.minRow === selectionRect.maxRow &&
              selectionRect.minCol === selectionRect.maxCol
                ? ' maze-selection-frame--single'
                : ''
            }`}
            style={frameStyle}
            aria-hidden="true"
          />
        )}
      </div>
    )
  },
)
