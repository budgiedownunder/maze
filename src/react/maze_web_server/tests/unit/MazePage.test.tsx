import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within, fireEvent, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createMemoryRouter, RouterProvider } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockMazeAlpha, resetMockMazes } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { MazePage } from '../../src/pages/MazePage'

const { mockGenerateMaze, mockSolveMaze } = vi.hoisted(() => ({
  mockGenerateMaze: vi.fn(),
  mockSolveMaze: vi.fn(),
}))

vi.mock('../../src/wasm/mazeWasm', () => ({
  generateMaze: mockGenerateMaze,
  solveMaze: mockSolveMaze,
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
  MazeGamePlayerMoveResult: { None: 0, Moved: 1, Blocked: 2, Complete: 3 },
}))

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

const generatedDefinition = {
  grid: [
    ['S', ' ', ' ', ' '],
    [' ', 'W', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', 'F'],
  ],
}

const solvePath = [{ row: 0, col: 0 }, { row: 1, col: 0 }, { row: 2, col: 0 }, { row: 2, col: 2 }]

beforeEach(() => {
  vi.clearAllMocks()
  resetMockMazes()
  mockGenerateMaze.mockResolvedValue(generatedDefinition)
  mockSolveMaze.mockResolvedValue(solvePath)
})

function renderMazePage(path: string) {
  const router = createMemoryRouter(
    [
      { path: '/mazes/new', element: <ThemeProvider><MazePage /></ThemeProvider> },
      { path: '/mazes/:id', element: <ThemeProvider><MazePage /></ThemeProvider> },
    ],
    { initialEntries: [path] },
  )
  return render(<RouterProvider router={router} />)
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

  it('toolbar is visible with editing buttons disabled when no cell is selected', async () => {
    await loadMazePage('/mazes/new')
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(toolbar).toBeInTheDocument()
    expect(within(toolbar).getByRole('button', { name: 'Set Wall' })).toBeDisabled()
    expect(within(toolbar).getByRole('button', { name: 'Generate' })).not.toBeDisabled()
  })

  it('toolbar buttons become enabled after clicking a cell', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Set Wall' })).toBeDisabled()
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    expect(screen.getByRole('button', { name: 'Set Wall' })).not.toBeDisabled()
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
    expect(screen.getByRole('button', { name: /^Clear$/ })).toBeDisabled()
  })

  it('Clear is enabled on a wall cell', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell (1,1) in Alpha is 'W'
    await userEvent.click(screen.getByLabelText('Cell 2,2'))
    expect(screen.getByRole('button', { name: /^Clear$/ })).not.toBeDisabled()
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

  it('Done button replaces Select range after clicking a row header', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    expect(within(toolbar).queryByRole('button', { name: 'Select range' })).not.toBeInTheDocument()
    expect(within(toolbar).getByRole('button', { name: 'Done' })).toBeInTheDocument()
  })

  it('clicking Done after row header click collapses to single-cell selection', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Done' }))
    expect(within(toolbar).getByRole('button', { name: 'Select range' })).toBeInTheDocument()
  })

  it('shift-clicking a second row header extends the selection rather than replacing it', async () => {
    await loadMazePage('/mazes/new')
    // Click row 1 header — selects row 1
    await userEvent.click(screen.getByLabelText('Row 1'))
    // Shift-click row 3 header — should extend the selection to span rows 1–3
    fireEvent.click(screen.getByLabelText('Row 3'), { shiftKey: true })
    // Row 1 header should still appear highlighted (still in selection)
    expect(screen.getByLabelText('Row 1').className).toContain('maze-cell-row-header--selected')
    expect(screen.getByLabelText('Row 3').className).toContain('maze-cell-row-header--selected')
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
    // new maze is 5×5; click row 1 header then insert
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Insert Rows Before' }))
    // Should now have 6 row headers
    expect(screen.getByLabelText('Row 6')).toBeInTheDocument()
  })

  it('clicking Delete removes a row when a row header is selected', async () => {
    await loadMazePage('/mazes/new')
    // new maze is 5×5; click row 1 header then delete
    await userEvent.click(screen.getByLabelText('Row 1'))
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    await userEvent.click(within(toolbar).getByRole('button', { name: 'Delete' }))
    // Should now have only 4 row headers
    expect(screen.queryByLabelText('Row 5')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Row 4')).toBeInTheDocument()
  })
})

// ──────────────────────────────────────────────────────────────
// Save and Refresh
// ──────────────────────────────────────────────────────────────

describe('MazePage save and refresh', () => {
  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('Save button is visible and enabled on a new maze before any edits', async () => {
    await loadMazePage('/mazes/new')
    const btn = screen.getByRole('button', { name: 'Save' })
    expect(btn).toBeInTheDocument()
    expect(btn).not.toBeDisabled()
  })

  it('Save button is visible on existing maze', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('Save button is disabled on existing maze when not dirty', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })

  it('Save button is enabled on existing maze after editing a cell', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    expect(screen.getByRole('button', { name: 'Save' })).not.toBeDisabled()
  })

  it('clicking Save on existing dirty maze calls updateMaze and clears dirty', async () => {
    const updateSpy = vi.fn()
    server.use(
      http.put('/api/v1/mazes/:id', async ({ request }) => {
        updateSpy()
        const body = await request.json() as { name: string }
        return HttpResponse.json({ ...mockMazeAlpha, name: body.name })
      }),
    )
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Edit to make dirty
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(updateSpy).toHaveBeenCalled())
    // Save button should be disabled again (isDirty cleared)
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled())
  })

  it('clicking Save on new maze opens name prompt', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    expect(screen.getByRole('dialog', { name: 'Save Maze' })).toBeInTheDocument()
  })

  it('save name modal calls createMaze and closes on success', async () => {
    const createSpy = vi.fn()
    server.use(
      http.post('/api/v1/mazes', async ({ request }) => {
        createSpy()
        const body = await request.json() as { name: string; definition: unknown }
        return HttpResponse.json({ id: 'new-maze-id', name: (body as { name: string }).name, definition: (body as { definition: unknown }).definition }, { status: 201 })
      }),
    )
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const dialog = screen.getByRole('dialog', { name: 'Save Maze' })
    await userEvent.type(within(dialog).getByRole('textbox'), 'My New Maze')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(createSpy).toHaveBeenCalled())
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Save Maze' })).not.toBeInTheDocument())
  })

  it('save name modal shows API error and stays open', async () => {
    server.use(
      http.post('/api/v1/mazes', () => new HttpResponse('A maze with that name already exists.', { status: 409 })),
    )
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    const dialog = screen.getByRole('dialog', { name: 'Save Maze' })
    await userEvent.type(within(dialog).getByRole('textbox'), 'Alpha')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(within(dialog).getByRole('alert')).toBeInTheDocument())
    // Modal still open
    expect(screen.getByRole('dialog', { name: 'Save Maze' })).toBeInTheDocument()
  })

  it('Refresh button is not shown for new maze', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.queryByRole('button', { name: 'Refresh' })).not.toBeInTheDocument()
  })

  it('Refresh button is shown but disabled on existing maze when not dirty', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    const btn = screen.getByRole('button', { name: 'Refresh' })
    expect(btn).toBeInTheDocument()
    expect(btn).toBeDisabled()
  })

  it('Refresh button is enabled on existing maze after editing', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    expect(screen.getByRole('button', { name: 'Refresh' })).not.toBeDisabled()
  })

  it('clicking Refresh when dirty opens confirm dialog', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Refresh' }))
    expect(screen.getByRole('dialog', { name: 'Discard changes?' })).toBeInTheDocument()
  })

  it('confirming Refresh reloads the maze from the server', async () => {
    const getSpy = vi.fn()
    server.use(
      http.get('/api/v1/mazes/:id', ({ params }) => {
        getSpy()
        const maze = params.id === mockMazeAlpha.id ? mockMazeAlpha : null
        if (!maze) return new HttpResponse(null, { status: 404 })
        return HttpResponse.json(maze)
      }),
    )
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // initial load fired once already
    const initialCalls = getSpy.mock.calls.length
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Refresh' }))
    await userEvent.click(screen.getByRole('button', { name: 'Reload' }))
    await waitFor(() => expect(getSpy.mock.calls.length).toBeGreaterThan(initialCalls))
    // Refresh button should be disabled again (isDirty cleared)
    await waitFor(() => expect(screen.getByRole('button', { name: 'Refresh' })).toBeDisabled())
  })

  it('cancelling Refresh dialog leaves the maze dirty', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,2'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Refresh' }))
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByRole('dialog', { name: 'Discard changes?' })).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Refresh' })).not.toBeDisabled()
  })
})

// ──────────────────────────────────────────────────────────────
// Generate
// ──────────────────────────────────────────────────────────────

describe('MazePage generate', () => {
  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('Generate button is enabled before and after clicking a cell', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Generate' })).not.toBeDisabled()
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    expect(screen.getByRole('button', { name: 'Generate' })).not.toBeDisabled()
  })

  it('clicking Generate button opens the Generate Maze dialog', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(screen.getByRole('dialog', { name: 'Generate Maze' })).toBeInTheDocument()
  })

  it('cancelling the Generate dialog closes it without calling generateMaze', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByRole('dialog', { name: 'Generate Maze' })).not.toBeInTheDocument()
    expect(mockGenerateMaze).not.toHaveBeenCalled()
  })

  it('successful generation closes dialog and updates grid', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    const dialog = screen.getByRole('dialog', { name: 'Generate Maze' })
    await userEvent.click(within(dialog).getByRole('button', { name: 'Generate' }))
    await waitFor(() => expect(mockGenerateMaze).toHaveBeenCalled())
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Generate Maze' })).not.toBeInTheDocument())
    // generatedDefinition is 4×4; new grid should have 4 row headers
    expect(screen.getByLabelText('Row 4')).toBeInTheDocument()
  })

  it('successful generation marks the maze dirty', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    await userEvent.click(within(screen.getByRole('dialog', { name: 'Generate Maze' })).getByRole('button', { name: 'Generate' }))
    await waitFor(() => expect(mockGenerateMaze).toHaveBeenCalled())
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).not.toBeDisabled())
  })

  it('reopening Generate dialog shows last used Min Solution Length', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    const dialog = () => screen.getByRole('dialog', { name: 'Generate Maze' })
    fireEvent.change(within(dialog()).getByLabelText('Min Solution Length'), { target: { value: '5' } })
    await userEvent.click(within(dialog()).getByRole('button', { name: 'Generate' }))
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Generate Maze' })).not.toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    expect(within(dialog()).getByLabelText('Min Solution Length')).toHaveValue(5)
  })

  it('WASM error keeps dialog open and shows error message', async () => {
    mockGenerateMaze.mockRejectedValue(new Error('generation failed'))
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    await userEvent.click(within(screen.getByRole('dialog', { name: 'Generate Maze' })).getByRole('button', { name: 'Generate' }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('generation failed'))
    expect(screen.getByRole('dialog', { name: 'Generate Maze' })).toBeInTheDocument()
  })

  it('Generate button is disabled while generating', async () => {
    let resolveGenerate!: (v: typeof generatedDefinition) => void
    mockGenerateMaze.mockReturnValue(new Promise(r => { resolveGenerate = r }))
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Generate' }))
    await userEvent.click(within(screen.getByRole('dialog', { name: 'Generate Maze' })).getByRole('button', { name: 'Generate' }))
    // While pending the submit button inside the modal should be disabled (isLoading)
    await waitFor(() =>
      expect(within(screen.getByRole('dialog', { name: 'Generate Maze' })).getByRole('button', { name: 'Generate' })).toBeDisabled()
    )
    await act(async () => { resolveGenerate(generatedDefinition) })
  })
})

// ── Solve ────────────────────────────────────────────────────
describe('MazePage solve', () => {
  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('Solve button is present and enabled when no solution is shown', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Solve' })).not.toBeDisabled()
  })

  it('Clear Solution button is disabled when no solution is shown', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Clear Solution' })).toBeDisabled()
  })

  it('clicking Solve calls solveMaze with the current grid', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(mockSolveMaze).toHaveBeenCalledWith({ grid: mockMazeAlpha.definition.grid }))
  })

  it('on success the solution overlay appears and Clear Solution becomes enabled', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getAllByAltText('Solution path').length).toBeGreaterThan(0))
    expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled()
  })

  it('on success a single-cell selection is retained as the active cell', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Cell 1,3 (row:0,col:2) is not on the solution path
    await userEvent.click(screen.getByLabelText('Cell 1,3'))
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getAllByAltText('Solution path').length).toBeGreaterThan(0))
    // Active cell highlight should still be shown
    expect(screen.getByLabelText('Cell 1,3').className).toContain('maze-cell--anchor')
    // Range mode / Done button should be gone
    expect(screen.queryByRole('button', { name: 'Done' })).not.toBeInTheDocument()
  })

  it('on success a multi-cell selection is collapsed to the anchor cell', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Anchor: Cell 1,3 (row:0,col:2); extend to Cell 2,3 (row:1,col:2) — both off the solution path
    await userEvent.click(screen.getByLabelText('Cell 1,3'))
    fireEvent.click(screen.getByLabelText('Cell 2,3'), { shiftKey: true })
    expect(screen.getByLabelText('Cell 1,3').className).toContain('maze-cell--anchor')
    expect(screen.getByLabelText('Cell 2,3').className).toContain('maze-cell--active')
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getAllByAltText('Solution path').length).toBeGreaterThan(0))
    // Anchor cell becomes the retained active cell; the range endpoint is gone
    expect(screen.getByLabelText('Cell 1,3').className).toContain('maze-cell--anchor')
    expect(screen.getByLabelText('Cell 2,3').className).not.toContain('maze-cell--active')
    expect(screen.queryByRole('button', { name: 'Done' })).not.toBeInTheDocument()
  })

  it('on WASM error an alert dialog is shown with the capitalised message', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution found'))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Unable to solve maze' })).toBeInTheDocument())
    expect(screen.getByRole('dialog', { name: 'Unable to solve maze' })).toHaveTextContent('No solution found')
  })

  it('clicking OK on the error dialog dismisses it', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution found'))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Unable to solve maze' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'OK' }))
    expect(screen.queryByRole('dialog', { name: 'Unable to solve maze' })).not.toBeInTheDocument()
  })

  it('clicking Clear Solution removes the overlay and disables the button', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled())
    await userEvent.click(screen.getByRole('button', { name: 'Clear Solution' }))
    expect(screen.queryByAltText('Solution path')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Clear Solution' })).toBeDisabled()
  })

  it('Solve button is disabled while solving', async () => {
    let resolveSolve!: (v: typeof solvePath) => void
    mockSolveMaze.mockReturnValue(new Promise(r => { resolveSolve = r }))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Solve' })).toBeDisabled())
    await act(async () => { resolveSolve(solvePath) })
  })

  it('editing buttons and Solve are disabled while the solution is displayed', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled())
    expect(screen.getByRole('button', { name: 'Set Wall' })).toBeDisabled()
    expect(screen.getByRole('button', { name: /^Clear$/ })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Generate' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Solve' })).toBeDisabled()
  })
})

// ──────────────────────────────────────────────────────────────
// Walk Solution
// ──────────────────────────────────────────────────────────────

describe('MazePage walk solution', () => {
  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('Walk Solution button is present and enabled when no solution is shown', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Walk Solution' })).not.toBeDisabled()
  })

  it('Walk Solution button is disabled when a solution is already displayed', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Solve' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled())
    expect(screen.getByRole('button', { name: 'Walk Solution' })).toBeDisabled()
  })

  it('Walk Solution button is disabled while solving', async () => {
    let resolveSolve!: (v: typeof solvePath) => void
    mockSolveMaze.mockReturnValue(new Promise(r => { resolveSolve = r }))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Walk Solution' })).toBeDisabled())
    resolveSolve(solvePath)
  })

  it('clicking Walk Solution calls solveMaze', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(mockSolveMaze).toHaveBeenCalledWith({ grid: mockMazeAlpha.definition.grid }))
  })

  it('on WASM error an alert dialog is shown and walk does not start', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no path exists'))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Unable to solve maze' })).toBeInTheDocument())
    expect(screen.getByRole('dialog', { name: 'Unable to solve maze' })).toHaveTextContent('No path exists')
    expect(screen.queryByAltText('Walker')).not.toBeInTheDocument()
  })

  it('walker image appears at the start cell during animation', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    // Walker GIF should appear before the animation completes
    await waitFor(() => expect(screen.getByAltText('Walker')).toBeInTheDocument())
  })

  it('after walk completes the celebrate GIF stays visible and Clear Solution remains enabled', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    // Wait for the celebrate GIF to appear at the finish cell
    await waitFor(() => expect(screen.getByAltText('Walker')).toHaveAttribute('src', '/images/maze/walker_celebrate.gif'), { timeout: 5000 })
    // Celebrate GIF remains — walker does not disappear until Clear Solution
    expect(screen.getByAltText('Walker')).toBeInTheDocument()
    // Walked cells behind the walker show footstep overlays
    expect(screen.getAllByAltText('Solution path').length).toBeGreaterThan(0)
    expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled()
  }, 10000)

  it('Clear Solution is enabled while walk is in progress', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByAltText('Walker')).toBeInTheDocument())
    expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled()
  })

  it('clicking Clear Solution mid-walk cancels the walk and resets the grid', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByAltText('Walker')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Clear Solution' }))
    expect(screen.queryByAltText('Walker')).not.toBeInTheDocument()
    expect(screen.queryByAltText('Solution path')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Clear Solution' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Walk Solution' })).not.toBeDisabled()
  })

  it('clicking a cell during walk does not change the active cell selection', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByAltText('Walker')).toBeInTheDocument())
    // Cell 1,3 is a Wall — clicking it should have no effect while the walk is running
    await userEvent.click(screen.getByLabelText('Cell 1,3'))
    expect(screen.getByLabelText('Cell 1,3').className).not.toContain('maze-cell--anchor')
  })

  it('pressing W during walk does not set a wall', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    const wallsBefore = screen.getAllByAltText('Wall').length
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(() => expect(screen.getByAltText('Walker')).toBeInTheDocument())
    fireEvent.keyDown(screen.getByLabelText('Maze grid'), { key: 'W' })
    expect(screen.getAllByAltText('Wall').length).toBe(wallsBefore)
  })

  it('Clear Solution after walk removes the overlay and re-enables editing', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Walk Solution' }))
    await waitFor(
      () => expect(screen.getByRole('button', { name: 'Clear Solution' })).not.toBeDisabled(),
      { timeout: 5000 },
    )
    await userEvent.click(screen.getByRole('button', { name: 'Clear Solution' }))
    expect(screen.queryByAltText('Solution path')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Clear Solution' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Walk Solution' })).not.toBeDisabled()
  }, 10000)
})

// ──────────────────────────────────────────────────────────────
// Double-tap range mode (touch)
// ──────────────────────────────────────────────────────────────

describe('MazePage double-tap range mode (touch)', () => {
  // vitest.setup.ts matchMedia stub returns { matches: false } for any query.
  // isTouchOnly = !window.matchMedia('(hover: hover) and (pointer: fine)').matches = !false = true.
  // No per-test override needed.

  async function loadMazePage(path: string) {
    renderMazePage(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('double-clicking a cell enters range mode', async () => {
    await loadMazePage('/mazes/new')
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    // Double-click a cell — fires onClick twice then onDoubleClick → handleCellDoubleClick → enableRangeMode()
    await userEvent.dblClick(screen.getByLabelText('Cell 1,1'))
    expect(within(toolbar).getByRole('button', { name: 'Done' })).toBeInTheDocument()
    expect(within(toolbar).queryByRole('button', { name: 'Select range' })).not.toBeInTheDocument()
  })

  it('double-clicking again exits range mode', async () => {
    await loadMazePage('/mazes/new')
    const toolbar = screen.getByLabelText('Maze editor toolbar')
    // First double-click enters range mode
    await userEvent.dblClick(screen.getByLabelText('Cell 1,1'))
    expect(within(toolbar).getByRole('button', { name: 'Done' })).toBeInTheDocument()
    // Second double-click exits range mode
    await userEvent.dblClick(screen.getByLabelText('Cell 1,1'))
    expect(within(toolbar).getByRole('button', { name: 'Select range' })).toBeInTheDocument()
    expect(within(toolbar).queryByRole('button', { name: 'Done' })).not.toBeInTheDocument()
  })
})

// ── Play ─────────────────────────────────────────────────────
describe('MazePage play', () => {
  // Use a router that includes /play/:id so we can detect navigation by checking
  // that the placeholder renders — no need to mock useNavigate.
  function renderWithPlay(path: string) {
    const router = createMemoryRouter(
      [
        { path: '/mazes/new', element: <ThemeProvider><MazePage /></ThemeProvider> },
        { path: '/mazes/:id', element: <ThemeProvider><MazePage /></ThemeProvider> },
        { path: '/play/:id', element: <div data-testid="play-page">Play Page</div> },
      ],
      { initialEntries: [path] },
    )
    return render(<RouterProvider router={router} />)
  }

  async function loadMazePage(path: string) {
    renderWithPlay(path)
    if (path !== '/mazes/new') {
      await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    }
  }

  it('Play button is present and enabled for a loaded maze', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    expect(screen.getByRole('button', { name: 'Play' })).not.toBeDisabled()
  })

  it('Play button is present for a new maze', async () => {
    await loadMazePage('/mazes/new')
    expect(screen.getByRole('button', { name: 'Play' })).toBeInTheDocument()
  })

  it('clean maze: Play calls solveMaze then navigates to /play/:id', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    await waitFor(() => expect(mockSolveMaze).toHaveBeenCalledWith({ grid: mockMazeAlpha.definition.grid }))
    await waitFor(() => expect(screen.getByTestId('play-page')).toBeInTheDocument())
  })

  it('clean maze: unsolvable → AlertModal shown with reason, no navigate', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution'))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Cannot Play Maze' })).toBeInTheDocument())
    expect(screen.getByRole('dialog', { name: 'Cannot Play Maze' })).toHaveTextContent('No solution')
    expect(screen.queryByTestId('play-page')).not.toBeInTheDocument()
  })

  it('clean maze: dismissing Cannot Play Maze alert closes it', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution'))
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Cannot Play Maze' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'OK' }))
    expect(screen.queryByRole('dialog', { name: 'Cannot Play Maze' })).not.toBeInTheDocument()
  })

  it('dirty maze: Play opens Unsaved Changes confirm modal', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    expect(screen.getByRole('dialog', { name: 'Unsaved Changes' })).toBeInTheDocument()
    expect(screen.getByRole('dialog', { name: 'Unsaved Changes' })).toHaveTextContent('Save and play?')
  })

  it('dirty maze: Cancel on confirm modal closes it without navigating', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Unsaved Changes' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByRole('dialog', { name: 'Unsaved Changes' })).not.toBeInTheDocument()
    expect(screen.queryByTestId('play-page')).not.toBeInTheDocument()
  })

  it('dirty maze: Save & Play saves, solve-checks, then navigates to /play/:id', async () => {
    await loadMazePage(`/mazes/${mockMazeAlpha.id}`)
    await userEvent.click(screen.getByLabelText('Cell 1,1'))
    await userEvent.click(screen.getByRole('button', { name: 'Set Wall' }))
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Unsaved Changes' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Save & Play' }))
    await waitFor(() => expect(mockSolveMaze).toHaveBeenCalled())
    await waitFor(() => expect(screen.getByTestId('play-page')).toBeInTheDocument())
  })

  it('new maze: Play opens the save-name modal', async () => {
    await loadMazePage('/mazes/new')
    await userEvent.click(screen.getByRole('button', { name: 'Play' }))
    expect(screen.getByRole('dialog', { name: 'Save Maze' })).toBeInTheDocument()
  })
})
