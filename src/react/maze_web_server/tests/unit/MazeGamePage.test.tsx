import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createMemoryRouter, RouterProvider } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockMazeAlpha } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { MazeGamePage } from '../../src/pages/MazeGamePage'

// ── Mocks ────────────────────────────────────────────────────

const { mockMove, mockUseMazeGame, mockGameInstance } = vi.hoisted(() => {
  const mockGameInstance = {
    is_complete:      vi.fn().mockReturnValue(false),
    player_row:       vi.fn().mockReturnValue(0),
    player_col:       vi.fn().mockReturnValue(0),
    player_direction: vi.fn().mockReturnValue(0),
    visited_cells:    vi.fn().mockReturnValue([]),
    free:             vi.fn(),
  }
  const mockMove = vi.fn()
  const mockUseMazeGame = vi.fn().mockReturnValue([
    { game: mockGameInstance, version: 0, loading: false, error: null },
    mockMove,
  ])
  return { mockMove, mockUseMazeGame, mockGameInstance }
})

vi.mock('../../src/hooks/useMazeGame', () => ({
  useMazeGame: mockUseMazeGame,
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
}))

vi.mock('../../src/components/MazeGrid', () => ({
  MazeGrid: (props: Record<string, unknown>) => (
    <div data-testid="maze-grid" data-version={props.version as number} />
  ),
}))

vi.mock('../../src/components/GameResultPopup', () => ({
  GameResultPopup: ({ message, onClose }: { message: string; onClose: () => void }) => (
    <div data-testid="game-result-popup">
      <span>{message}</span>
      <button type="button" onClick={onClose}>Close</button>
    </div>
  ),
}))

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: null, login: vi.fn(), logout: vi.fn() }),
  }
})

// ── Helpers ──────────────────────────────────────────────────

function renderPage(id = mockMazeAlpha.id) {
  const router = createMemoryRouter(
    [{ path: '/play/:id', element: <ThemeProvider><MazeGamePage /></ThemeProvider> }],
    { initialEntries: [`/play/${id}`] },
  )
  return render(<RouterProvider router={router} />)
}

async function waitForLoad() {
  await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
}

beforeEach(() => {
  vi.clearAllMocks()
  mockGameInstance.is_complete.mockReturnValue(false)
  mockUseMazeGame.mockReturnValue([
    { game: mockGameInstance, version: 0, loading: false, error: null },
    mockMove,
  ])
})

// ── Tests ────────────────────────────────────────────────────

describe('MazeGamePage', () => {
  it('shows loading while maze is fetching', () => {
    server.use(http.get('/api/v1/mazes/:id', () => new Promise(() => {})))
    renderPage()
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
  })

  it('shows error when getMaze fails', async () => {
    server.use(http.get('/api/v1/mazes/:id', () => new HttpResponse('Not found', { status: 404 })))
    renderPage()
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
  })

  it('renders MazeGrid after maze loads', async () => {
    renderPage()
    await waitForLoad()
    expect(screen.getByTestId('maze-grid')).toBeInTheDocument()
  })

  it('shows maze name in header', async () => {
    renderPage()
    await waitForLoad()
    expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument()
  })

  it('ArrowUp calls move with Up (1)', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowUp' })
    expect(mockMove).toHaveBeenCalledWith(1)
  })

  it('w calls move with Up (1)', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'w' })
    expect(mockMove).toHaveBeenCalledWith(1)
  })

  it('ArrowDown calls move with Down (2)', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowDown' })
    expect(mockMove).toHaveBeenCalledWith(2)
  })

  it('ArrowLeft calls move with Left (3)', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowLeft' })
    expect(mockMove).toHaveBeenCalledWith(3)
  })

  it('ArrowRight calls move with Right (4)', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowRight' })
    expect(mockMove).toHaveBeenCalledWith(4)
  })

  it('keyboard ignored when game is complete', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowUp' })
    expect(mockMove).not.toHaveBeenCalled()
  })

  it('D-pad "Move up" button calls move with Up (1)', async () => {
    renderPage()
    await waitForLoad()
    await userEvent.click(screen.getByRole('button', { name: /move up/i }))
    expect(mockMove).toHaveBeenCalledWith(1)
  })

  it('D-pad "Move down" button calls move with Down (2)', async () => {
    renderPage()
    await waitForLoad()
    await userEvent.click(screen.getByRole('button', { name: /move down/i }))
    expect(mockMove).toHaveBeenCalledWith(2)
  })

  it('D-pad "Move left" button calls move with Left (3)', async () => {
    renderPage()
    await waitForLoad()
    await userEvent.click(screen.getByRole('button', { name: /move left/i }))
    expect(mockMove).toHaveBeenCalledWith(3)
  })

  it('D-pad "Move right" button calls move with Right (4)', async () => {
    renderPage()
    await waitForLoad()
    await userEvent.click(screen.getByRole('button', { name: /move right/i }))
    expect(mockMove).toHaveBeenCalledWith(4)
  })

  it('D-pad buttons are disabled when game is complete', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitForLoad()
    expect(screen.getByRole('button', { name: /move up/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /move down/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /move left/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /move right/i })).toBeDisabled()
  })

  it('GameResultPopup appears when game becomes complete', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitFor(() => expect(screen.getByTestId('game-result-popup')).toBeInTheDocument())
    expect(screen.getByText('Congratulations! You completed the maze!')).toBeInTheDocument()
  })

  it('Close button on GameResultPopup dismisses it', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitFor(() => expect(screen.getByTestId('game-result-popup')).toBeInTheDocument())
    await act(async () => {
      await userEvent.click(screen.getByRole('button', { name: /close/i }))
    })
    expect(screen.queryByTestId('game-result-popup')).not.toBeInTheDocument()
  })

  it('keyboard legend has maze-shortcuts-hint class', async () => {
    renderPage()
    await waitForLoad()
    const legend = document.querySelector('.maze-shortcuts-hint')
    expect(legend).toBeInTheDocument()
  })

  it('keyboard legend shows all four directions', async () => {
    renderPage()
    await waitForLoad()
    const legend = document.querySelector('.maze-shortcuts-hint')!
    expect(legend.textContent).toMatch(/Up/)
    expect(legend.textContent).toMatch(/Down/)
    expect(legend.textContent).toMatch(/Left/)
    expect(legend.textContent).toMatch(/Right/)
  })

  it('keyboard legend shows arrow and letter key hints', async () => {
    renderPage()
    await waitForLoad()
    const legend = document.querySelector('.maze-shortcuts-hint')!
    expect(legend.textContent).toMatch(/W/)
    expect(legend.textContent).toMatch(/S/)
    expect(legend.textContent).toMatch(/A/)
    expect(legend.textContent).toMatch(/D/)
  })
})
