import { describe, it, expect, vi } from 'vitest'
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
    expect(screen.getByLabelText('Cell 1,1').className).toContain('maze-cell--solution')
    expect(screen.getByLabelText('Cell 2,1').className).toContain('maze-cell--solution')
    expect(screen.getByLabelText('Cell 1,2').className).not.toContain('maze-cell--solution')
  })

  it('renders solution footstep images with correct directions', () => {
    // path goes right then down
    render(
      <MazeGrid
        grid={[['S', ' '], [' ', 'F']]}
        solution={[{ row: 0, col: 0 }, { row: 0, col: 1 }, { row: 1, col: 1 }]}
        activeCell={null}
        anchorCell={null}
      />,
    )
    const footstepImgs = screen.getAllByAltText('Solution path')
    expect(footstepImgs).toHaveLength(3)
    // (0,0) → next is (0,1): going right
    expect(footstepImgs[0]).toHaveAttribute('src', '/images/maze/footsteps_right.png')
    // (0,1) → next is (1,1): going down
    expect(footstepImgs[1]).toHaveAttribute('src', '/images/maze/footsteps_down.png')
    // (1,1) → last: incoming direction was down
    expect(footstepImgs[2]).toHaveAttribute('src', '/images/maze/footsteps_down.png')
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
