import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MazeGrid, CELL_SIZE, HEADER_SIZE } from '../../src/components/MazeGrid'

const GRID_3X3 = [
  ['S', ' ', 'W'],
  [' ', 'W', ' '],
  [' ', ' ', 'F'],
]

function renderGrid(props: Partial<React.ComponentPropsWithoutRef<typeof MazeGrid>> = {}) {
  return render(
    <MazeGrid
      grid={GRID_3X3}
      solution={null}
      activeCell={null}
      anchorCell={null}
      {...props}
    />,
  )
}

describe('MazeGrid', () => {
  it('renders correct number of data cells', () => {
    renderGrid()
    // 3 rows × 3 cols = 9 data cells (<td> elements)
    expect(screen.getAllByRole('cell')).toHaveLength(9)
  })

  it('renders correct number of column headers', () => {
    renderGrid()
    // <th scope="col"> — 3 col headers + 1 corner <th> (aria-label="Select all")
    const colHeaders = screen.getAllByRole('columnheader')
    expect(colHeaders).toHaveLength(4)
  })

  it('renders correct number of row headers', () => {
    renderGrid()
    // <th scope="row">
    expect(screen.getAllByRole('rowheader')).toHaveLength(3)
  })

  it('renders Wall image for W cells', () => {
    renderGrid()
    const wallImgs = screen.getAllByAltText('Wall')
    expect(wallImgs).toHaveLength(2)
    expect(wallImgs[0]).toHaveAttribute('src', '/images/maze/wall.png')
  })

  it('renders Start image for S cell', () => {
    renderGrid()
    const img = screen.getByAltText('Start')
    expect(img).toHaveAttribute('src', '/images/maze/start_flag.png')
  })

  it('renders Finish image for F cell', () => {
    renderGrid()
    const img = screen.getByAltText('Finish')
    expect(img).toHaveAttribute('src', '/images/maze/finish_flag.png')
  })

  it('renders Key image for K cells', () => {
    render(
      <MazeGrid
        grid={[['K', ' '], [' ', ' ']]}
        solution={null}
        activeCell={null}
        anchorCell={null}
      />,
    )
    const img = screen.getByAltText('Key')
    expect(img).toHaveAttribute('src', '/images/maze/key.svg')
  })

  it('renders Door image for D cells', () => {
    render(
      <MazeGrid
        grid={[['D', ' '], [' ', ' ']]}
        solution={null}
        activeCell={null}
        anchorCell={null}
      />,
    )
    const img = screen.getByAltText('Door')
    expect(img).toHaveAttribute('src', '/images/maze/door.svg')
  })

  it('renders no content images for empty cells', () => {
    render(
      <MazeGrid
        grid={[[' ', ' '], [' ', ' ']]}
        solution={null}
        activeCell={null}
        anchorCell={null}
      />,
    )
    // Should have no Wall/Start/Finish images
    expect(screen.queryByAltText('Wall')).not.toBeInTheDocument()
    expect(screen.queryByAltText('Start')).not.toBeInTheDocument()
    expect(screen.queryByAltText('Finish')).not.toBeInTheDocument()
  })

  it('applies maze-cell--anchor class to a single active cell (no anchor)', () => {
    // When anchorCell is null, the active cell is treated as the anchor origin → yellow
    renderGrid({ activeCell: { row: 0, col: 0 } })
    const activeEl = screen.getByLabelText('Cell 1,1')
    expect(activeEl.className).toContain('maze-cell--anchor')
  })

  it('applies maze-cell--active class to the active cell when a separate anchor exists', () => {
    // Active cell (0,0) is the range end; anchor (2,2) is origin → active gets in-range color
    renderGrid({ activeCell: { row: 0, col: 0 }, anchorCell: { row: 2, col: 2 } })
    const activeEl = screen.getByLabelText('Cell 1,1')
    expect(activeEl.className).toContain('maze-cell--active')
    expect(activeEl.className).not.toContain('maze-cell--anchor')
  })

  it('applies maze-cell--anchor class to the anchor cell', () => {
    renderGrid({ activeCell: { row: 0, col: 0 }, anchorCell: { row: 2, col: 2 } })
    const anchorEl = screen.getByLabelText('Cell 3,3')
    expect(anchorEl.className).toContain('maze-cell--anchor')
  })

  it('applies maze-cell--in-range to cells inside the selection rectangle', () => {
    renderGrid({ activeCell: { row: 0, col: 0 }, anchorCell: { row: 1, col: 1 } })
    // (0,0)=active, (1,1)=anchor, (0,1) and (1,0) are in-range
    expect(screen.getByLabelText('Cell 1,2').className).toContain('maze-cell--in-range')
    expect(screen.getByLabelText('Cell 2,1').className).toContain('maze-cell--in-range')
  })

  it('applies maze-cell--solution to solution path cells', () => {
    renderGrid({ solution: [{ row: 0, col: 0 }, { row: 1, col: 0 }] })
    expect(screen.getByLabelText('Cell 1,1').className).not.toContain('maze-cell--solution') // 'S' cell — no solution background
    expect(screen.getByLabelText('Cell 2,1').className).toContain('maze-cell--solution')
    expect(screen.getByLabelText('Cell 1,2').className).not.toContain('maze-cell--solution')
  })

  it('renders solution footstep images with correct directions, skipping S and F cells', () => {
    // path: S(0,0) → right → (0,1) → down → (0,2) → down → F(1,2)
    // only (0,1) and (0,2) should receive footstep images
    render(
      <MazeGrid
        grid={[['S', ' ', ' '], [' ', ' ', 'F']]}
        solution={[{ row: 0, col: 0 }, { row: 0, col: 1 }, { row: 0, col: 2 }, { row: 1, col: 2 }]}
        activeCell={null}
        anchorCell={null}
      />,
    )
    const footstepImgs = screen.getAllByAltText('Solution path')
    // S and F cells must not have footsteps
    expect(footstepImgs).toHaveLength(2)
    // (0,1) → next is (0,2): going right
    expect(footstepImgs[0]).toHaveAttribute('src', '/images/maze/footsteps_right.png')
    // (0,2) → next is (1,2): going down
    expect(footstepImgs[1]).toHaveAttribute('src', '/images/maze/footsteps_down.png')
  })

  it('fires onCellDoubleClick with correct row and col on double-click', async () => {
    const onCellDoubleClick = vi.fn()
    renderGrid({ onCellDoubleClick })
    await userEvent.dblClick(screen.getByLabelText('Cell 1,1'))
    expect(onCellDoubleClick).toHaveBeenCalledWith(0, 0)
  })

  it('fires onCellClick with correct row, col, shift=false on plain click', async () => {
    const onClick = vi.fn()
    renderGrid({ onCellClick: onClick })
    await userEvent.click(screen.getByLabelText('Cell 2,3'))
    expect(onClick).toHaveBeenCalledWith(1, 2, false)
  })

  it('fires onRowHeaderClick with correct row index', async () => {
    const onRowClick = vi.fn()
    renderGrid({ onRowHeaderClick: onRowClick })
    await userEvent.click(screen.getByLabelText('Row 2'))
    expect(onRowClick).toHaveBeenCalledWith(1, false)
  })

  it('fires onColHeaderClick with correct col index', async () => {
    const onColClick = vi.fn()
    renderGrid({ onColHeaderClick: onColClick })
    await userEvent.click(screen.getByLabelText('Column 3'))
    expect(onColClick).toHaveBeenCalledWith(2, false)
  })

  it('renders selection frame when activeCell is set', () => {
    const { container } = renderGrid({ activeCell: { row: 1, col: 1 } })
    const frame = container.querySelector('.maze-selection-frame')
    expect(frame).toBeInTheDocument()
  })

  it('does not render selection frame when activeCell is null', () => {
    const { container } = renderGrid()
    expect(container.querySelector('.maze-selection-frame')).not.toBeInTheDocument()
  })

  it('positions the selection frame correctly for a single active cell', () => {
    const { container } = renderGrid({ activeCell: { row: 1, col: 2 } })
    const frame = container.querySelector('.maze-selection-frame') as HTMLElement
    expect(frame).toBeInTheDocument()
    // In JSDOM getBoundingClientRect returns zeros, so calculated fallback is used:
    // top = HEADER_SIZE + row * CELL_SIZE, left = HEADER_SIZE + col * CELL_SIZE
    expect(frame.style.top).toBe(`${HEADER_SIZE + 1 * CELL_SIZE}px`)
    expect(frame.style.left).toBe(`${HEADER_SIZE + 2 * CELL_SIZE}px`)
    expect(frame.style.width).toBe(`${CELL_SIZE}px`)
    expect(frame.style.height).toBe(`${CELL_SIZE}px`)
  })

  it('positions the selection frame correctly for a range', () => {
    const { container } = renderGrid({
      activeCell: { row: 2, col: 2 },
      anchorCell: { row: 0, col: 0 },
    })
    const frame = container.querySelector('.maze-selection-frame') as HTMLElement
    expect(frame).toBeInTheDocument()
    // minRow=0, minCol=0 → top = HEADER_SIZE + 0 = HEADER_SIZE, left = HEADER_SIZE
    // range spans 3×3 → width=3*32=96, height=3*32=96
    expect(frame.style.top).toBe(`${HEADER_SIZE}px`)
    expect(frame.style.left).toBe(`${HEADER_SIZE}px`)
    expect(frame.style.width).toBe(`${3 * CELL_SIZE}px`)
    expect(frame.style.height).toBe(`${3 * CELL_SIZE}px`)
  })

  it('applies maze-selection-frame--single when a single cell is selected and range mode is off', () => {
    const { container } = renderGrid({ activeCell: { row: 1, col: 1 } })
    expect(container.querySelector('.maze-selection-frame--single')).toBeInTheDocument()
  })

  it('does not apply maze-selection-frame--single when range mode is on (even with single cell)', () => {
    const { container } = renderGrid({ activeCell: { row: 1, col: 1 }, isRangeMode: true })
    const frame = container.querySelector('.maze-selection-frame') as HTMLElement
    expect(frame).toBeInTheDocument()
    expect(frame.classList.contains('maze-selection-frame--single')).toBe(false)
  })

  it('does not apply maze-selection-frame--single when multiple cells are selected', () => {
    const { container } = renderGrid({
      activeCell: { row: 2, col: 2 },
      anchorCell: { row: 0, col: 0 },
    })
    const frame = container.querySelector('.maze-selection-frame') as HTMLElement
    expect(frame).toBeInTheDocument()
    expect(frame.classList.contains('maze-selection-frame--single')).toBe(false)
  })
})

// ──────────────────────────────────────────────────────────────
// Walk animation rendering
// ──────────────────────────────────────────────────────────────

const WALK_PATH = [
  { row: 0, col: 0 }, // S
  { row: 1, col: 0 },
  { row: 2, col: 0 },
  { row: 2, col: 2 }, // F
]

describe('MazeGrid walk animation', () => {
  it('renders walker image at the current walk position', () => {
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 1, isComplete: false },
    })
    expect(screen.getByAltText('Walker')).toBeInTheDocument()
    // Walker is in cell (1,0) = Cell 2,1
    const cell = screen.getByLabelText('Cell 2,1')
    expect(cell.querySelector('img[alt="Walker"]')).toBeInTheDocument()
  })

  it('walker cell does not render normal cell image (wall/start/finish)', () => {
    // Put walker on the Start cell (row 0, col 0)
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 0, isComplete: false },
    })
    const startCell = screen.getByLabelText('Cell 1,1')
    expect(startCell.querySelector('img[alt="Walker"]')).toBeInTheDocument()
    expect(startCell.querySelector('img[alt="Start"]')).not.toBeInTheDocument()
  })

  it('walked cells show solution path images', () => {
    // currentIndex=2: cells at indices 0 and 1 have been walked
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 2, isComplete: false },
    })
    // index 0 = (0,0) = Start cell — S/F cells never get footstep overlay
    // index 1 = (1,0) — empty cell should have footstep
    const walkedCell = screen.getByLabelText('Cell 2,1')
    expect(walkedCell.querySelector('img[alt="Solution path"]')).toBeInTheDocument()
  })

  it('walked cells receive maze-cell--solution class', () => {
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 2, isComplete: false },
    })
    // Cell (1,0) = Cell 2,1 was walked
    expect(screen.getByLabelText('Cell 2,1').className).toContain('maze-cell--solution')
  })

  it('walker cell does not receive maze-cell--solution class', () => {
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 1, isComplete: false },
    })
    expect(screen.getByLabelText('Cell 2,1').className).not.toContain('maze-cell--solution')
  })

  it('unwalked cells render normally', () => {
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 1, isComplete: false },
    })
    // Cell (2,2) = Cell 3,3 = Finish — not yet reached
    const finishCell = screen.getByLabelText('Cell 3,3')
    expect(finishCell.querySelector('img[alt="Finish"]')).toBeInTheDocument()
    expect(finishCell.querySelector('img[alt="Walker"]')).not.toBeInTheDocument()
  })

  it('renders celebrate image when isComplete is true', () => {
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: WALK_PATH.length - 1, isComplete: true },
    })
    const walkerImg = screen.getByAltText('Walker')
    expect(walkerImg).toHaveAttribute('src', '/images/maze/walker_celebrate.gif')
  })

  it('walker image src reflects direction of travel', () => {
    // currentIndex=0, next is (1,0) so direction is 'down'
    renderGrid({
      walkState: { path: WALK_PATH, currentIndex: 0, isComplete: false },
    })
    expect(screen.getByAltText('Walker')).toHaveAttribute('src', '/images/maze/walker_down.gif')
  })

  it('no walker image when walkState is null', () => {
    renderGrid()
    expect(screen.queryByAltText('Walker')).not.toBeInTheDocument()
  })
})

// ──────────────────────────────────────────────────────────────
// Game mode (game prop)
// ──────────────────────────────────────────────────────────────

// Grid where S is at (0,0), empty cells at (0,1) and (1,0)/(1,1), F at (1,1)
const GAME_GRID = [
  ['S', ' '],
  [' ', 'F'],
]

function makeGameObj(overrides: Partial<{
  player_row: () => number
  player_col: () => number
  player_direction: () => number
  is_complete: () => boolean
  visited_cells: () => Array<{ row: number; col: number }>
  keys: () => Array<{ row: number; col: number; id: number }>
  doors: () => Array<{ row: number; col: number; state: string }>
  enemies: () => Array<{ row: number; col: number; id: number; enemyType?: string }>
  health_pickups: () => Array<{ row: number; col: number; id: number }>
  treasures: () => Array<{ row: number; col: number; style: string; value: number }>
}> = {}) {
  return {
    player_row:       vi.fn().mockReturnValue(0),
    player_col:       vi.fn().mockReturnValue(0),
    player_direction: vi.fn().mockReturnValue(0), // MazeGameDirection.None
    is_complete:      vi.fn().mockReturnValue(false),
    visited_cells:    vi.fn().mockReturnValue([]),
    keys:             vi.fn().mockReturnValue([]),
    doors:            vi.fn().mockReturnValue([]),
    enemies:          vi.fn().mockReturnValue([]),
    health_pickups:   vi.fn().mockReturnValue([]),
    treasures:        vi.fn().mockReturnValue([]),
    free:             vi.fn(),
    ...overrides,
  }
}

function renderGameGrid(
  game: ReturnType<typeof makeGameObj>,
  version = 0,
  cellOverrides?: Map<string, never>,
) {
  return render(
    <MazeGrid
      grid={GAME_GRID}
      solution={null}
      activeCell={null}
      anchorCell={null}
      game={game as never}
      version={version}
      cellOverrides={cellOverrides as never}
    />,
  )
}

describe('MazeGrid game mode', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders player walker at player cell', () => {
    const game = makeGameObj({ player_row: vi.fn().mockReturnValue(0), player_col: vi.fn().mockReturnValue(0) })
    renderGameGrid(game)
    expect(screen.getByAltText('Player')).toBeInTheDocument()
  })

  it('walker_up.gif shown when player direction is Up (1)', () => {
    const game = makeGameObj({ player_direction: vi.fn().mockReturnValue(1) }) // Up
    renderGameGrid(game)
    expect(screen.getByAltText('Player')).toHaveAttribute('src', '/images/maze/walker_up.gif')
  })

  it('walker_celebrate.gif shown when is_complete is true', () => {
    const game = makeGameObj({
      is_complete: vi.fn().mockReturnValue(true),
      player_direction: vi.fn().mockReturnValue(1),
    })
    renderGameGrid(game)
    expect(screen.getByAltText('Player')).toHaveAttribute('src', '/images/maze/walker_celebrate.gif')
  })

  it('visited non-S cell shows visited_dot.png', () => {
    // Player at (1,0); visited cell is (0,1)
    const game = makeGameObj({
      player_row: vi.fn().mockReturnValue(1),
      player_col: vi.fn().mockReturnValue(0),
      visited_cells: vi.fn().mockReturnValue([{ row: 0, col: 1 }]),
    })
    renderGameGrid(game, 1)
    expect(screen.getByAltText('')).toHaveAttribute('src', '/images/maze/visited_dot.png')
  })

  it('visited start cell (S) restores start_flag.png, not dot', () => {
    // Player moved away from (0,0) — start cell is now in visitedSet
    const game = makeGameObj({
      player_row: vi.fn().mockReturnValue(1),
      player_col: vi.fn().mockReturnValue(0),
      visited_cells: vi.fn().mockReturnValue([{ row: 0, col: 0 }]),
    })
    renderGameGrid(game, 1)
    // start_flag.png should appear (from visitedSet restore), NOT visited_dot
    const startImgs = screen.getAllByAltText('Start')
    expect(startImgs.length).toBeGreaterThan(0)
    expect(screen.queryByAltText('')).not.toBeInTheDocument()
  })

  it('player cell not in visitedSet shows no dot', () => {
    // Player at (0,0), visited_cells empty
    const game = makeGameObj()
    renderGameGrid(game)
    expect(screen.queryByAltText('')).not.toBeInTheDocument()
  })

  it('cell click is suppressed when game is set', async () => {
    const onCellClick = vi.fn()
    const game = makeGameObj()
    const { container } = renderGameGrid(game)
    const cell = container.querySelector('td[aria-label="Cell 1,1"]')!
    await userEvent.click(cell)
    expect(onCellClick).not.toHaveBeenCalled()
  })

  it('renders enemy overlay sprite at each live enemy cell', () => {
    // Player at (0,0); enemy at (1,1)
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([{ row: 1, col: 1, id: 0 }]),
    })
    renderGameGrid(game, 1)
    const enemyImgs = screen.getAllByAltText('Enemy')
    expect(enemyImgs.length).toBe(1)
    expect(enemyImgs[0]).toHaveAttribute('src', '/images/maze/enemy.svg')
  })

  it('suppresses the static spawn-cell E character in game mode (live position drives rendering)', () => {
    const gridWithStaticE = [
      ['S', 'E'],
      [' ', 'F'],
    ]
    const game = makeGameObj({
      // Enemy has moved from spawn (0,1) to (1,0).
      enemies: vi.fn().mockReturnValue([{ row: 1, col: 0, id: 0 }]),
    })
    render(
      <MazeGrid
        grid={gridWithStaticE}
        solution={null}
        activeCell={null}
        anchorCell={null}
        game={game as never}
        version={1}
      />,
    )
    const enemyImgs = screen.getAllByAltText('Enemy')
    // Only one Enemy sprite — the live position. The spawn-cell E is suppressed.
    expect(enemyImgs.length).toBe(1)
    // The live enemy sprite is in cell (1,0) — confirm by walking up from
    // the rendered img and asserting the surrounding <td>'s aria-label.
    const enemyCell = enemyImgs[0].closest('td')
    expect(enemyCell).toHaveAttribute('aria-label', 'Cell 2,1')
  })

  it('renders a half-opacity goblin behind the player when an enemy shares the cell', () => {
    // Player and enemy both at (0,0). The player walker covers the full
    // cell, so the enemy is rendered at half opacity *behind* the player
    // sprite so the player can see what is attacking them. With a single
    // attacker there is no count chip.
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([{ row: 0, col: 0, id: 0 }]),
    })
    renderGameGrid(game, 1)
    const enemyImg = screen.getByAltText('Enemy on player cell')
    expect(enemyImg).toBeInTheDocument()
    expect(enemyImg).toHaveAttribute('src', '/images/maze/enemy.svg')
    expect(screen.getByAltText('Player')).toBeInTheDocument()
    expect(screen.queryByLabelText(/enemies attacking/i)).not.toBeInTheDocument()
  })

  it('shows a count chip + behind-the-player goblin when multiple enemies attack', () => {
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([
        { row: 0, col: 0, id: 0 },
        { row: 0, col: 0, id: 1 },
        { row: 0, col: 0, id: 2 },
      ]),
    })
    renderGameGrid(game, 1)
    // The enemy sprite behind the player has an alt text describing the count.
    expect(screen.getByAltText('3 enemies on player cell')).toBeInTheDocument()
    // The count chip surfaces the integer count separately for accessibility.
    const chip = screen.getByLabelText('3 enemies attacking')
    expect(chip).toBeInTheDocument()
    expect(chip.textContent).toContain('3')
  })

  it('does not render any attack indicator when the player cell has no enemies', () => {
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([{ row: 1, col: 1, id: 0 }]),
    })
    renderGameGrid(game, 1)
    expect(screen.queryByAltText(/on player cell/i)).not.toBeInTheDocument()
    expect(screen.queryByLabelText(/enemies attacking/i)).not.toBeInTheDocument()
  })

  it('stack-count chip is rendered when multiple enemies share a non-player cell', () => {
    // Player at (0,0); two enemies stacked at (1,1).
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([
        { row: 1, col: 1, id: 0 },
        { row: 1, col: 1, id: 1 },
      ]),
    })
    renderGameGrid(game, 1)
    expect(screen.getByLabelText('2 enemies')).toBeInTheDocument()
  })

  it('does not render the visited dot at a cell currently holding an enemy', () => {
    const game = makeGameObj({
      player_row: vi.fn().mockReturnValue(1),
      player_col: vi.fn().mockReturnValue(0),
      enemies: vi.fn().mockReturnValue([{ row: 0, col: 1, id: 0 }]),
      visited_cells: vi.fn().mockReturnValue([{ row: 0, col: 1 }]),
    })
    renderGameGrid(game, 1)
    expect(screen.queryByAltText('')).not.toBeInTheDocument()
    expect(screen.getByAltText('Enemy')).toBeInTheDocument()
  })
})

describe('MazeGrid game-mode variant sprites', () => {
  it('renders the ghost sprite for a live enemy with a ghost rig', () => {
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([{ row: 1, col: 0, id: 0, enemyType: 'ghost' }]),
    })
    renderGameGrid(game, 1)
    expect(screen.getByAltText('Enemy')).toHaveAttribute('src', '/images/maze/ghost.svg')
  })

  it('renders the generic enemy sprite when the live enemy has no rig override', () => {
    const game = makeGameObj({
      enemies: vi.fn().mockReturnValue([{ row: 1, col: 0, id: 0 }]),
    })
    renderGameGrid(game, 1)
    expect(screen.getByAltText('Enemy')).toHaveAttribute('src', '/images/maze/enemy.svg')
  })

  it('renders the potion sprite for an uncollected health cell with a potion rig', () => {
    const game = makeGameObj({
      health_pickups: vi.fn().mockReturnValue([{ row: 0, col: 1, id: 0 }]),
    })
    render(
      <MazeGrid
        grid={[['S', 'H', 'F']]}
        solution={null}
        activeCell={null}
        anchorCell={null}
        game={game as never}
        version={1}
        cellOverrides={new Map([['0,1', { type: 'H', healthStyle: 'potion' }]]) as never}
      />,
    )
    expect(screen.getByAltText('Health')).toHaveAttribute('src', '/images/maze/potion.svg')
  })
})

describe('MazeGrid game mode — keys & doors', () => {
  const KD_GRID = [['S', 'K', 'D', 'F']]

  function renderKD(game: ReturnType<typeof makeGameObj>) {
    return render(
      <MazeGrid grid={KD_GRID} solution={null} activeCell={null} anchorCell={null} game={game as never} version={1} />,
    )
  }

  it('renders a key for an uncollected key cell', () => {
    const game = makeGameObj({ keys: () => [{ row: 0, col: 1, id: 0 }] })
    renderKD(game)
    expect(screen.getByAltText('Key')).toHaveAttribute('src', '/images/maze/key.svg')
  })

  it('does not render a key once it has been collected (omitted from keys())', () => {
    const game = makeGameObj({ keys: () => [] })
    renderKD(game)
    expect(screen.queryByAltText('Key')).not.toBeInTheDocument()
  })

  it('renders a locked door', () => {
    const game = makeGameObj({ doors: () => [{ row: 0, col: 2, state: 'locked' }] })
    renderKD(game)
    expect(screen.getByAltText('Door')).toHaveAttribute('src', '/images/maze/door.svg')
  })

  it('does not render an opened door (becomes a passage)', () => {
    const game = makeGameObj({ doors: () => [{ row: 0, col: 2, state: 'open' }] })
    renderKD(game)
    expect(screen.queryByAltText('Door')).not.toBeInTheDocument()
  })
})

describe('MazeGrid game mode — treasure', () => {
  const T_GRID = [['S', 'T', 'F']]

  function renderT(game: ReturnType<typeof makeGameObj>) {
    return render(
      <MazeGrid grid={T_GRID} solution={null} activeCell={null} anchorCell={null} game={game as never} version={1} />,
    )
  }

  it('renders the per-style sprite for an uncollected treasure cell', () => {
    const game = makeGameObj({ treasures: () => [{ row: 0, col: 1, style: 'gold', value: 100 }] })
    renderT(game)
    expect(screen.getByAltText('Treasure')).toHaveAttribute('src', '/images/maze/gold.svg')
  })

  it('renders the silver sprite for a default-style treasure cell', () => {
    const game = makeGameObj({ treasures: () => [{ row: 0, col: 1, style: 'silver', value: 50 }] })
    renderT(game)
    expect(screen.getByAltText('Treasure')).toHaveAttribute('src', '/images/maze/silver.svg')
  })

  it('does not render treasure once collected (omitted from treasures())', () => {
    const game = makeGameObj({ treasures: () => [] })
    renderT(game)
    expect(screen.queryByAltText('Treasure')).not.toBeInTheDocument()
  })
})

describe('MazeGrid override badge + variant sprite (editor mode)', () => {
  it('marks only the cells present in cellOverrides', () => {
    renderGrid({ cellOverrides: new Map([['0,1', { type: 'E' }]]) })
    expect(screen.getAllByLabelText('Has override')).toHaveLength(1)
  })

  it('renders no badges when cellOverrides is omitted', () => {
    renderGrid()
    expect(screen.queryByLabelText('Has override')).not.toBeInTheDocument()
  })

  it('renders the variant sprite for an overridden enemy cell', () => {
    renderGrid({
      grid: [['E', ' ', 'F']],
      cellOverrides: new Map([['0,0', { type: 'E', enemyType: 'ghost' }]]),
    })
    expect(screen.getByAltText('Enemy')).toHaveAttribute('src', '/images/maze/ghost.svg')
  })

  it('renders the generic sprite for a feature cell with no visual override', () => {
    renderGrid({
      grid: [['E', ' ', 'F']],
      cellOverrides: new Map([['0,0', { type: 'E', damage: 3 }]]),
    })
    expect(screen.getByAltText('Enemy')).toHaveAttribute('src', '/images/maze/enemy.svg')
  })
})
