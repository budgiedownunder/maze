import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockMazeAlpha, resetMockMazes } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { MazePage } from '../../src/pages/MazePage'

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: true,
      profile: null,
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

beforeEach(() => {
  vi.clearAllMocks()
  resetMockMazes()
})

function renderMazePage(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <ThemeProvider>
        <Routes>
          <Route path="/mazes/new" element={<MazePage />} />
          <Route path="/mazes/:id" element={<MazePage />} />
        </Routes>
      </ThemeProvider>
    </MemoryRouter>,
  )
}

describe('MazePage', () => {
  it('shows loading indicator while fetching an existing maze', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
  })

  it('renders the maze name in the header after loading', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())
  })

  it('renders the maze grid after loading', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Wait for loading to finish
    await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    // Grid container should be present
    expect(screen.getByLabelText('Maze grid')).toBeInTheDocument()
    // Start cell image should be visible
    expect(screen.getByAltText('Start')).toBeInTheDocument()
    // Finish cell image should be visible
    expect(screen.getByAltText('Finish')).toBeInTheDocument()
  })

  it('shows not-found message for a 404 response', async () => {
    server.use(
      http.get('/api/v1/mazes/:id', () => new HttpResponse(null, { status: 404 })),
    )
    renderMazePage('/mazes/nonexistent-id')
    await waitFor(() => expect(screen.getByText(/not found/i)).toBeInTheDocument())
  })

  it('shows error message when API fails', async () => {
    server.use(
      http.get('/api/v1/mazes/:id', () => new HttpResponse('Server exploded', { status: 500 })),
    )
    renderMazePage('/mazes/some-id')
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(screen.getByRole('alert')).toHaveTextContent('Server exploded')
  })

  it('shows (unsaved) in the header for the /mazes/new route', () => {
    renderMazePage('/mazes/new')
    expect(screen.getByText('(unsaved)')).toBeInTheDocument()
  })

  it('renders a blank grid for /mazes/new without making an API call', () => {
    const getMazeSpy = vi.fn()
    server.use(http.get('/api/v1/mazes/:id', () => { getMazeSpy(); return new HttpResponse(null, { status: 404 }) }))
    renderMazePage('/mazes/new')
    // Grid should appear immediately (no loading state)
    expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Maze grid')).toBeInTheDocument()
    // No API call should have been made
    expect(getMazeSpy).not.toHaveBeenCalled()
  })
})

// ──────────────────────────────────────────────────────────────
// Toolbar visibility and button states
// ──────────────────────────────────────────────────────────────

describe('MazePage toolbar', () => {
  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('toolbar is hidden when no cell is selected', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.queryByLabelText('Maze editor toolbar')).not.toBeInTheDocument()
  })

  it('toolbar appears after clicking a cell', async () => {
    await loadMazePage('/mazes/new')
    const cell = screen.getByLabelText('Cell 1,1')
    await userEvent.click(cell)
    expect(screen.getByLabelText('Maze editor toolbar')).toBeInTheDocument()
  })

  it('Set Wall is enabled on an empty cell', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    expect(screen.getByRole('button', { name: 'Set Wall' })).not.toBeDisabled()
  })

  it('Set Wall is disabled when selection is all walls', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell (1,1) in Alpha is 'W'
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    const btn = screen.getByRole('button', { name: 'Set Wall' })
    expect(btn).toBeDisabled()
  })

  it('Set Start is enabled on a single empty cell', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    expect(screen.getByRole('button', { name: 'Set Start' })).not.toBeDisabled()
  })

  it('Set Start is disabled when selected cell already contains S', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell (0,0) in Alpha is 'S'
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    expect(screen.getByRole('button', { name: 'Set Start' })).toBeDisabled()
  })

  it('Set Start is disabled for multi-cell selection', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    fireEvent.click(screen.getByLabelText('Cell 2,2'), { shiftKey: true })
    expect(screen.getByRole('button', { name: 'Set Start' })).toBeDisabled()
  })

  it('Set Finish is enabled on a single empty cell', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    expect(screen.getByRole('button', { name: 'Set Finish' })).not.toBeDisabled()
  })

  it('Set Finish is disabled when selected cell already contains F', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell (2,2) in Alpha is 'F'
    await userEvent.click(screen.getByLabelText('Cell 3,3'))
    expect(screen.getByRole('button', { name: 'Set Finish' })).toBeDisabled()
  })

  it('Clear is disabled on an empty cell', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    expect(screen.getByRole('button', { name: 'Clear' })).toBeDisabled()
  })

  it('Clear is enabled on a wall cell', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell (1,1) in Alpha is 'W'
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    expect(screen.getByRole('button', { name: 'Clear' })).not.toBeDisabled()
  })

  it('clicking Set Wall changes the cell and keeps toolbar visible', async () => {
    await loadMazePage('/mazes/new')
    const cell = screen.getByLabelText('Cell 1,1')
    await userEvent.click(cell)
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    // Cell should now show wall image
    const cellEl = screen.getByLabelText('Cell 1,1')
    expect(within(cellEl).getByAltText('Wall')).toBeInTheDocument()
    // Toolbar still visible
    expect(screen.getByLabelText('Maze editor toolbar')).toBeInTheDocument()
  })

  it('clicking Set Start places S and clears previous S', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Alpha has S at (0,0) — click cell (1,2) which is empty and set start there
    await userEvent.click(screen.getByLabelText('Cell 2,3'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Start' }))
    // Old start cell (0,0) should no longer show Start image; new cell should
    const oldStartCell = screen.getByLabelText('Cell 1,1')
    expect(within(oldStartCell).queryByAltText('Start')).not.toBeInTheDocument()
    const newStartCell = screen.getByLabelText('Cell 2,3')
    expect(within(newStartCell).getByAltText('Start')).toBeInTheDocument()
  })

  it('shift+click extends selection and disables Set Start / Set Finish', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    fireEvent.click(screen.getByLabelText('Cell 2,2'), { shiftKey: true })
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Set Start' })).toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Set Finish' })).toBeDisabled()
    // Set Wall should still be enabled (not all walls)
    expect(within(toolbar).getByRole('button', { name: 'Set Wall' })).not.toBeDisabled()
  })

  it('Select range button is present in toolbar when cell is selected', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Select range' })).toBeInTheDocument()
  })

  it('Select range button is replaced by Done button after clicking it', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Select range' }))
    expect(within(toolbar).queryByRole('button', { name: 'Select range' })).not.toBeInTheDocument()
    expect(within(toolbar).getByRole('button', { name: 'Done' })).toBeInTheDocument()
  })

  it('Done button restores Select range button and clears anchor', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Select range' }))
    // Click another cell — should extend selection (anchor set)
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    expect(screen.getByLabelText('Cell 2,2').className).toContain('maze-cell--active')
    // Click Done — anchor should be cleared, back to single cell
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Done' }))
    expect(within(toolbar).getByRole('button', { name: 'Select range' })).toBeInTheDocument()
    expect(screen.getByLabelText('Cell 2,2').className).toContain('maze-cell--anchor')
  })

  // ── Structural editing button enable/disable ─────────────────

  it('Insert Rows Before and Delete are enabled after clicking a row header', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Rows Before' })).not.toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Delete' })).not.toBeDisabled()
  })

  it('Insert Columns Before is disabled and Delete is enabled after clicking a row header', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Columns Before' })).toBeDisabled()
  })

  it('Insert Columns Before and Delete are enabled after clicking a column header', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Column 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Columns Before' })).not.toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Delete' })).not.toBeDisabled()
  })

  it('Insert Rows Before is disabled after clicking a column header', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Column 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Rows Before' })).toBeDisabled()
  })

  it('Insert Rows Before and Insert Columns Before are enabled after clicking the corner (select all)', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Select all'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Rows Before' })).not.toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Insert Columns Before' })).not.toBeDisabled()
  })

  it('Delete is disabled after clicking the corner (select all — entire grid)', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Select all'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Delete' })).toBeDisabled()
  })

  it('Insert Rows Before and Delete are enabled when full row selected via keyboard', async () => {
    await loadMazePage('/mazes/new')
    // Select first cell then Shift+End to span the full row
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    fireEvent.keyDown(screen.getByLabelText('Maze grid'), { key: 'End', shiftKey: true })
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).getByRole('button', { name: 'Insert Rows Before' })).not.toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Delete' })).not.toBeDisabled()
  })

  it('clicking Insert Rows Before adds a row to the grid', async () => {
    await loadMazePage('/mazes/new')
    // new maze is 7×7; click row 1 header then insert
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Insert Rows Before' }))
    // Should now have 8 row headers
    expect(screen.getByLabelText('Row 8')).toBeInTheDocument()
  })

  it('clicking Delete removes a row when a row header is selected', async () => {
    await loadMazePage('/mazes/new')
    // new maze is 7×7; click row 1 header then delete
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Delete' }))
    // Should now have only 6 row headers
    expect(screen.queryByLabelText('Row 7')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Row 6')).toBeInTheDocument()
  })
})
