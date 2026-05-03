import { forwardRef, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import type { CellPoint } from '../hooks/useMazeEditor'
import type { WalkState } from '../hooks/useWalkAnimation'
import type { MazeGameWasm } from 'maze_wasm'
import { MazeGameDirection } from '../wasm/mazeWasm'

export const CELL_SIZE = 32
export const HEADER_SIZE = 24

interface MazeGridProps {
  grid: string[][]
  solution: Array<CellPoint> | null
  walkState?: WalkState | null
  activeCell: CellPoint | null
  anchorCell: CellPoint | null
  isRangeMode?: boolean
  onCellClick?: (row: number, col: number, shift: boolean) => void
  onCellDoubleClick?: (row: number, col: number) => void
  onRowHeaderClick?: (row: number, shift: boolean) => void
  onColHeaderClick?: (col: number, shift: boolean) => void
  onCornerClick?: () => void
  onKeyDown?: (e: React.KeyboardEvent) => void
  game?: MazeGameWasm | null
  version?: number
  cellSize?: number
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

const WALKER_IMAGES: Record<string, string> = {
  up: '/images/maze/walker_up.gif',
  down: '/images/maze/walker_down.gif',
  left: '/images/maze/walker_left.gif',
  right: '/images/maze/walker_right.gif',
}
const WALKER_CELEBRATE_IMG = '/images/maze/walker_celebrate.gif'

const VISITED_DOT_IMG = '/images/maze/visited_dot.png'

const GAME_WALKER_IMAGES: Record<number, string> = {
  [MazeGameDirection.None]:  '/images/maze/walker_down.gif',
  [MazeGameDirection.Up]:    '/images/maze/walker_up.gif',
  [MazeGameDirection.Down]:  '/images/maze/walker_down.gif',
  [MazeGameDirection.Left]:  '/images/maze/walker_left.gif',
  [MazeGameDirection.Right]: '/images/maze/walker_right.gif',
}

function gameWalkerImg(dir: MazeGameDirection, isComplete: boolean): string {
  if (isComplete) return WALKER_CELEBRATE_IMG
  return GAME_WALKER_IMAGES[dir] ?? GAME_WALKER_IMAGES[MazeGameDirection.Down]
}

function directionBetween(from: CellPoint, to: CellPoint): string {
  if (to.row < from.row) return 'up'
  if (to.row > from.row) return 'down'
  if (to.col < from.col) return 'left'
  return 'right'
}

interface WalkInfo {
  walkerKey: string
  walkerImg: string
  walkedMap: Map<string, string>
}

function buildWalkInfo(walkState: WalkState): WalkInfo {
  const { path, currentIndex, isComplete } = walkState
  const current = path[currentIndex]
  const walkerKey = `${current.row},${current.col}`

  let walkerImg: string
  if (isComplete) {
    walkerImg = WALKER_CELEBRATE_IMG
  } else {
    const next = path[currentIndex + 1]
    walkerImg = WALKER_IMAGES[directionBetween(current, next)]
  }

  const walkedMap = buildSolutionMap(path.slice(0, currentIndex))
  return { walkerKey, walkerImg, walkedMap }
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
    { grid, solution, walkState, activeCell, anchorCell, isRangeMode = false, onCellClick, onCellDoubleClick, onRowHeaderClick, onColHeaderClick, onCornerClick, onKeyDown, game, version, cellSize = CELL_SIZE },
    ref,
  ) {
    const rows = grid.length
    const cols = rows > 0 ? grid[0].length : 0

    const playerRow  = game?.player_row()       ?? -1
    const playerCol  = game?.player_col()       ?? -1
    const playerDir  = game?.player_direction() ?? MazeGameDirection.None
    const isComplete = game?.is_complete()      ?? false

    const visitedSet = useMemo(() => {
      if (!game) return null
      const cells = game.visited_cells() as unknown as Array<{ row: number; col: number }>
      return new Set(cells.map(c => `${c.row},${c.col}`))
    }, [version, game]) // eslint-disable-line react-hooks/exhaustive-deps

    const solutionMap = useMemo(
      () => (solution ? buildSolutionMap(solution) : new Map<string, string>()),
      [solution],
    )

    const walkInfo = useMemo(
      () => (walkState ? buildWalkInfo(walkState) : null),
      [walkState],
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

    const measureFrame = useCallback(() => {
      if (!selectionRect || !containerRef.current) {
        setFrameStyle(null)
        return
      }

      const container = containerRef.current
      const containerRect = container.getBoundingClientRect()

      // JSDOM (unit tests) always returns zero-sized rects — fall back to calculated values.
      if (containerRect.width === 0) {
        setFrameStyle({
          top: HEADER_SIZE + selectionRect.minRow * cellSize,
          left: HEADER_SIZE + selectionRect.minCol * cellSize,
          width: (selectionRect.maxCol - selectionRect.minCol + 1) * cellSize,
          height: (selectionRect.maxRow - selectionRect.minRow + 1) * cellSize,
        })
        return
      }

      const tlCell = container.querySelector<HTMLElement>(
        `td[aria-label="Cell ${selectionRect.minRow + 1},${selectionRect.minCol + 1}"]`,
      )
      const brCell = container.querySelector<HTMLElement>(
        `td[aria-label="Cell ${selectionRect.maxRow + 1},${selectionRect.maxCol + 1}"]`,
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
    }, [selectionRect, cellSize])

    // Re-measure whenever measureFrame's inputs change (selection, cell size).
    // Canonical "measure DOM post-layout, then setState" pattern — the setState
    // inside measureFrame is required because frameStyle depends on values only
    // available after the browser has laid out the cells.
    useLayoutEffect(() => {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      measureFrame()
    }, [measureFrame])

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

      const cellTop  = HEADER_SIZE + activeCell.row * cellSize
      const cellLeft = HEADER_SIZE + activeCell.col * cellSize

      if (cellTop  < container.scrollTop  + HEADER_SIZE) container.scrollTop  = cellTop  - HEADER_SIZE
      if (cellLeft < container.scrollLeft + HEADER_SIZE) container.scrollLeft = cellLeft - HEADER_SIZE

      const needDown  = cellTop  + cellSize > container.scrollTop  + container.clientHeight
      const needRight = cellLeft + cellSize > container.scrollLeft + container.clientWidth
      if (needDown || needRight) {
        container
          .querySelector<HTMLElement>(`td[aria-label="Cell ${activeCell.row + 1},${activeCell.col + 1}"]`)
          ?.scrollIntoView({
            block:  needDown  ? 'end' : 'nearest',
            inline: needRight ? 'end' : 'nearest',
          })
      }
    }, [activeCell, cellSize])

    // Scroll the walker cell into view on every step, using the same logic as activeCell above.
    useLayoutEffect(() => {
      if (!walkState || !containerRef.current) return
      const container = containerRef.current
      if (container.clientWidth === 0) return // JSDOM — no layout

      const { row, col } = walkState.path[walkState.currentIndex]
      const cellTop  = HEADER_SIZE + row * cellSize
      const cellLeft = HEADER_SIZE + col * cellSize

      if (cellTop  < container.scrollTop  + HEADER_SIZE) container.scrollTop  = cellTop  - HEADER_SIZE
      if (cellLeft < container.scrollLeft + HEADER_SIZE) container.scrollLeft = cellLeft - HEADER_SIZE

      const needDown  = cellTop  + cellSize > container.scrollTop  + container.clientHeight
      const needRight = cellLeft + cellSize > container.scrollLeft + container.clientWidth
      if (needDown || needRight) {
        container
          .querySelector<HTMLElement>(`td[aria-label="Cell ${row + 1},${col + 1}"]`)
          ?.scrollIntoView({
            block:  needDown  ? 'end' : 'nearest',
            inline: needRight ? 'end' : 'nearest',
          })
      }
    }, [walkState, cellSize])

    // Scroll a lookahead cell into view after each successful game move.
    useEffect(() => {
      if (!game || !containerRef.current) return
      const container = containerRef.current
      if (container.clientWidth === 0) return // JSDOM — no layout
      const rowCount = grid.length
      const colCount = grid[0]?.length ?? 0
      let tr: number, tc: number
      if      (playerDir === MazeGameDirection.Up)    { tr = playerRow - 1; tc = playerCol + 1 }
      else if (playerDir === MazeGameDirection.Down)  { tr = playerRow + 3; tc = playerCol + 1 }
      else if (playerDir === MazeGameDirection.Left)  { tr = playerRow + 1; tc = playerCol - 1 }
      else if (playerDir === MazeGameDirection.Right) { tr = playerRow + 1; tc = playerCol + 3 }
      else                                            { tr = playerRow + 1; tc = playerCol + 1 }
      tr = Math.max(1, Math.min(rowCount, tr)) - 1
      tc = Math.max(1, Math.min(colCount, tc)) - 1
      container
        .querySelector<HTMLElement>(`td[aria-label="Cell ${tr + 1},${tc + 1}"]`)
        ?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
    }, [version, game, grid]) // eslint-disable-line react-hooks/exhaustive-deps

    function getCellClasses(row: number, col: number): string {
      const classes = ['maze-cell']
      const key = `${row},${col}`
      const isWalker = walkInfo?.walkerKey === key
      const inWalked = !isWalker && (walkInfo?.walkedMap.has(key) ?? false)
      const inSolution = solutionMap.has(key)
      const inSolutionOrWalk = inSolution || isWalker || inWalked

      // Only apply selection highlights to cells not in the solution/walk path —
      // solution styling takes visual precedence over the active/range highlight.
      if (activeCell && !inSolutionOrWalk) {
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

      // Walked cells get the solution highlight colour; walker cell keeps normal background
      if (inWalked) {
        classes.push('maze-cell--solution')
      } else if (inSolution && grid[row][col] !== 'S' && grid[row][col] !== 'F') {
        classes.push('maze-cell--solution')
      }

      return classes.join(' ')
    }

    const effectiveOnCellClick      = game ? undefined : onCellClick
    const effectiveOnCellDoubleClick = game ? undefined : onCellDoubleClick
    const effectiveOnRowHeaderClick = game ? undefined : onRowHeaderClick
    const effectiveOnColHeaderClick = game ? undefined : onColHeaderClick
    const effectiveOnCornerClick    = game ? undefined : onCornerClick
    const effectiveOnKeyDown        = game ? undefined : onKeyDown

    const isAllSelected =
      selectionRect !== null &&
      selectionRect.minRow === 0 && selectionRect.maxRow === rows - 1 &&
      selectionRect.minCol === 0 && selectionRect.maxCol === cols - 1

    return (
      <div
        className="maze-grid-container"
        ref={setContainerRef}
        tabIndex={0}
        onKeyDown={effectiveOnKeyDown}
        aria-label="Maze grid"
      >
        <table className="maze-grid">
          <thead>
            <tr>
              <th
                className={`maze-cell-corner${isAllSelected ? ' maze-cell-corner--selected' : ''}`}
                onClick={() => effectiveOnCornerClick?.()}
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
                  onClick={e => effectiveOnColHeaderClick?.(c, e.shiftKey)}
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
                  onClick={e => effectiveOnRowHeaderClick?.(r, e.shiftKey)}
                  aria-label={`Row ${r + 1}`}
                >
                  {r + 1}
                </th>
                {Array.from({ length: cols }, (_, c) => {
                  const cell = grid[r]?.[c] ?? ' '
                  const key = `${r},${c}`
                  const isWalker = walkInfo?.walkerKey === key
                  const walkedImgSrc = !isWalker ? walkInfo?.walkedMap.get(key) : undefined
                  const solutionImgSrc = solutionMap.get(key)
                  const isGamePlayer = game !== null && game !== undefined && playerRow === r && playerCol === c
                  const img = (isWalker || isGamePlayer) ? null : cellImage(cell)
                  return (
                    <td
                      key={`cell-${r}-${c}`}
                      className={getCellClasses(r, c)}
                      onClick={e => effectiveOnCellClick?.(r, c, e.shiftKey)}
                      onDoubleClick={() => effectiveOnCellDoubleClick?.(r, c)}
                      aria-label={`Cell ${r + 1},${c + 1}`}
                    >
                      {isWalker && (
                        <img src={walkInfo!.walkerImg} alt="Walker" className="maze-cell-solution-img" />
                      )}
                      {!isWalker && img && <img src={img.src} alt={img.alt} />}
                      {walkedImgSrc && cell !== 'S' && cell !== 'F' && (
                        <img src={walkedImgSrc} alt="Solution path" className="maze-cell-solution-img" />
                      )}
                      {!isWalker && solutionImgSrc && cell !== 'S' && cell !== 'F' && (
                        <img src={solutionImgSrc} alt="Solution path" className="maze-cell-solution-img" />
                      )}
                      {game && visitedSet?.has(key) && !isGamePlayer && (
                        cell === 'S'
                          ? <img src="/images/maze/start_flag.png" alt="Start"
                                 style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', pointerEvents: 'none' }} />
                          : <img src={VISITED_DOT_IMG} alt="" aria-hidden
                                 style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', pointerEvents: 'none' }} />
                      )}
                      {isGamePlayer && (
                        <img src={gameWalkerImg(playerDir, isComplete)} alt="Player"
                             style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', pointerEvents: 'none' }} />
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
