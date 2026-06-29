import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest'
import { createMemoryRouter, RouterProvider } from 'react-router-dom'
import { ThemeProvider } from '../context/ThemeProvider'
import { AuthProvider } from '../context/AuthProvider'
import { AppFeaturesContext } from '../context/AppFeaturesContext'
import { http, HttpResponse } from 'msw'
import { MazePage } from './MazePage'
import { solveMaze, splitDefinition, buildDefinitionWithOverrides } from '../wasm/mazeWasm'
import { server } from '../mocks/server'

const BASE = '/api/v1'

vi.mock('../wasm/mazeWasm', () => ({
  solveMaze: vi.fn(),
  generateMaze: vi.fn(),
  // Lightweight JS stand-ins for the WASM codec: split parses the full maze JSON and
  // reports no overrides (mock mazes are pure-char); build echoes back the grid.
  splitDefinition: vi.fn(async (json: string) => {
    const parsed = JSON.parse(json) as { definition: { grid: string[][] } }
    return { grid: parsed.definition.grid, overrides: [] }
  }),
  buildDefinitionWithOverrides: vi.fn(async (grid: string[][]) => ({ grid })),
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
  MazeGamePlayerMoveResult: { None: 0, Moved: 1, Blocked: 2, Complete: 3 },
}))

// Path through mockMazeAlpha (S at 0,0 → F at 2,2)
const MOCK_SOLVE_PATH = [
  { row: 0, col: 0 },
  { row: 0, col: 1 },
  { row: 0, col: 2 },
  { row: 1, col: 2 },
  { row: 2, col: 2 },
]

function setupAuth() {
  sessionStorage.setItem(
    'auth',
    JSON.stringify({
      token: 'test-token',
      issuedAt: new Date().toISOString(),
      expiry: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }),
  )
}

function renderMazePage(id: string) {
  const router = createMemoryRouter(
    [{ path: '/mazes/:id', element: <MazePage /> }],
    { initialEntries: [`/mazes/${id}`] },
  )
  return render(
    <AppFeaturesContext.Provider value={{ allow_signup: true, oauth_providers: [], email_enabled: false, max_maze_cells: null }}>
      <ThemeProvider>
        <AuthProvider>
          <RouterProvider router={router} />
        </AuthProvider>
      </ThemeProvider>
    </AppFeaturesContext.Provider>,
  )
}

describe('MazePage busy state', () => {
  beforeEach(() => {
    setupAuth()
  })

  afterEach(() => {
    sessionStorage.clear()
    document.body.classList.remove('is-busy')
    vi.clearAllMocks()
  })

  it('adds is-busy to document.body while solving and removes it when done', async () => {
    // Set up a deferred solveMaze so we can assert the busy state mid-operation
    let resolveSolve!: (path: typeof MOCK_SOLVE_PATH) => void
    const solveDeferred = new Promise<typeof MOCK_SOLVE_PATH>(resolve => {
      resolveSolve = resolve
    })
    vi.mocked(solveMaze).mockReturnValueOnce(solveDeferred)

    renderMazePage('maze-0001')

    // Wait for maze to load and Solve button to be enabled
    const solveBtn = await screen.findByRole('button', { name: 'Solve' })
    await waitFor(() => expect(solveBtn).not.toBeDisabled())

    // Click Solve — isSolving becomes true, is-busy should appear on body
    await userEvent.click(solveBtn)
    await waitFor(() =>
      expect(document.body.classList.contains('is-busy')).toBe(true),
    )

    // Resolve the WASM call
    await act(async () => {
      resolveSolve(MOCK_SOLVE_PATH)
      await solveDeferred
    })

    // is-busy should be removed once the operation completes
    await waitFor(() =>
      expect(document.body.classList.contains('is-busy')).toBe(false),
    )
  })

  it('removes is-busy from document.body if the component unmounts during an operation', async () => {
    let resolveSolve!: (path: typeof MOCK_SOLVE_PATH) => void
    const solveDeferred = new Promise<typeof MOCK_SOLVE_PATH>(resolve => {
      resolveSolve = resolve
    })
    vi.mocked(solveMaze).mockReturnValueOnce(solveDeferred)

    const { unmount } = renderMazePage('maze-0001')

    const solveBtn = await screen.findByRole('button', { name: 'Solve' })
    await waitFor(() => expect(solveBtn).not.toBeDisabled())

    await userEvent.click(solveBtn)
    await waitFor(() =>
      expect(document.body.classList.contains('is-busy')).toBe(true),
    )

    // Unmount while the operation is still in flight
    unmount()
    expect(document.body.classList.contains('is-busy')).toBe(false)

    // Clean up the deferred to avoid unhandled promise warnings
    resolveSolve(MOCK_SOLVE_PATH)
  })
})

describe('MazePage walk speed control', () => {
  beforeEach(() => {
    setupAuth()
    localStorage.clear()
    vi.mocked(solveMaze).mockResolvedValue(MOCK_SOLVE_PATH)
  })

  afterEach(() => {
    sessionStorage.clear()
    localStorage.clear()
    vi.clearAllMocks()
  })

  async function startWalk() {
    renderMazePage('maze-0001')
    const walkBtn = await screen.findByRole('button', { name: 'Walk Solution' })
    await waitFor(() => expect(walkBtn).not.toBeDisabled())
    await userEvent.click(walkBtn)
    return screen.findByRole('combobox', { name: 'Walk speed' })
  }

  it('speed select is not shown before a walk starts', async () => {
    renderMazePage('maze-0001')
    await screen.findByRole('button', { name: 'Walk Solution' })
    expect(screen.queryByRole('combobox', { name: 'Walk speed' })).not.toBeInTheDocument()
  })

  it('speed select appears once a walk is in progress', async () => {
    const select = await startWalk()
    expect(select).toBeInTheDocument()
  })

  it('speed select defaults to Normal', async () => {
    const select = await startWalk()
    expect(select).toHaveValue('1') // index 1 = Normal
  })

  it('selecting a different speed persists to localStorage', async () => {
    const select = await startWalk()
    await userEvent.selectOptions(select, '3') // Fast
    expect(localStorage.getItem('walkSpeed')).toBe('3')
  })

  it('speed select disappears after Clear Solution cancels the walk', async () => {
    await startWalk()
    await userEvent.click(screen.getByRole('button', { name: 'Clear Solution' }))
    await waitFor(() =>
      expect(screen.queryByRole('combobox', { name: 'Walk speed' })).not.toBeInTheDocument(),
    )
  })
})

describe('MazePage override persistence + oversized-definition error', () => {
  beforeEach(() => {
    setupAuth()
  })

  afterEach(() => {
    sessionStorage.clear()
    vi.clearAllMocks()
  })

  // Loads maze-0001, selects an empty cell and stamps a wall so the maze is dirty
  // and the header Save button is enabled.
  async function loadAndDirty() {
    renderMazePage('maze-0001')
    const cell = await screen.findByLabelText('Cell 1,2')
    await userEvent.click(cell)
    const wallBtn = await screen.findByRole('button', { name: 'Set Wall' })
    await waitFor(() => expect(wallBtn).toBeEnabled())
    await userEvent.click(wallBtn)
  }

  it('splits the loaded definition through the WASM codec on load', async () => {
    renderMazePage('maze-0001')
    await screen.findByRole('button', { name: 'Solve' })
    expect(splitDefinition).toHaveBeenCalled()
  })

  it('saves the canonical definition built from grid + overrides', async () => {
    let sentBody: { definition?: unknown } | undefined
    server.use(
      http.put(`${BASE}/mazes/:id`, async ({ request }) => {
        sentBody = (await request.json()) as { definition?: unknown }
        return HttpResponse.json({ id: 'maze-0001', name: 'Alpha', definition: { grid: [] } })
      }),
    )
    await loadAndDirty()
    const saveBtn = screen.getByRole('button', { name: 'Save' })
    await waitFor(() => expect(saveBtn).toBeEnabled())
    await userEvent.click(saveBtn)

    await waitFor(() => expect(buildDefinitionWithOverrides).toHaveBeenCalled())
    // The PUT body carries the definition produced by the codec (mock: { grid }).
    await waitFor(() => expect(sentBody?.definition).toBeDefined())
  })

  it('shows the server message when a save is rejected with HTTP 422', async () => {
    server.use(
      http.put(
        `${BASE}/mazes/:id`,
        () => new HttpResponse('Maze definition is too large.', { status: 422 }),
      ),
    )
    await loadAndDirty()
    const saveBtn = screen.getByRole('button', { name: 'Save' })
    await waitFor(() => expect(saveBtn).toBeEnabled())
    await userEvent.click(saveBtn)

    expect(await screen.findByText('Maze definition is too large.')).toBeInTheDocument()
  })
})
