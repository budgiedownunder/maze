import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, fireEvent, act, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createMemoryRouter, RouterProvider } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockMazeAlpha, mockMazeOverrideStatic } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { MazeGamePage } from '../../src/pages/MazeGamePage'

// ── Mocks ────────────────────────────────────────────────────

const { mockMove, mockRestart, mockTogglePause, mockUseMazeGame, mockGameInstance } = vi.hoisted(() => {
  const mockGameInstance = {
    is_complete:      vi.fn().mockReturnValue(false),
    is_lost:          vi.fn().mockReturnValue(false),
    lose_reason:      vi.fn().mockReturnValue(null),
    player_row:       vi.fn().mockReturnValue(0),
    player_col:       vi.fn().mockReturnValue(0),
    player_direction: vi.fn().mockReturnValue(0),
    visited_cells:    vi.fn().mockReturnValue([]),
    keys:             vi.fn().mockReturnValue([]),
    doors:            vi.fn().mockReturnValue([]),
    bag:              vi.fn().mockReturnValue([]),
    hp:               vi.fn().mockReturnValue(3),
    max_hp:           vi.fn().mockReturnValue(3),
    enemies:          vi.fn().mockReturnValue([]),
    health_pickups:   vi.fn().mockReturnValue([]),
    free:             vi.fn(),
  }
  const mockMove = vi.fn()
  const mockRestart = vi.fn()
  const mockTogglePause = vi.fn()
  const mockUseMazeGame = vi.fn().mockReturnValue([
    { game: mockGameInstance, version: 0, loading: false, error: null, damageFlashKey: 0, paused: false },
    mockMove,
    mockRestart,
    mockTogglePause,
  ])
  return { mockMove, mockRestart, mockTogglePause, mockUseMazeGame, mockGameInstance }
})

vi.mock('../../src/hooks/useMazeGame', () => ({
  useMazeGame: mockUseMazeGame,
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
}))

// Mock only splitDefinition (the override codec) — keep the rest of the WASM bridge
// real so getBag/getHp/getMaxHp still operate on the mock game object.
const { mockSplitDefinition } = vi.hoisted(() => ({
  mockSplitDefinition: vi.fn().mockResolvedValue({ grid: [], overrides: [] }),
}))
vi.mock('../../src/wasm/mazeWasm', async () => {
  const actual = await vi.importActual<typeof import('../../src/wasm/mazeWasm')>('../../src/wasm/mazeWasm')
  return { ...actual, splitDefinition: mockSplitDefinition }
})

vi.mock('../../src/components/MazeGrid', () => ({
  MazeGrid: (props: Record<string, unknown>) => (
    <div
      data-testid="maze-grid"
      data-version={props.version as number}
      data-grid={JSON.stringify(props.grid)}
    />
  ),
}))

vi.mock('../../src/components/GameResultPopup', () => ({
  GameResultPopup: ({ message, tone = 'success', onClose }: { message: string; tone?: 'success' | 'fail'; onClose: () => void }) => (
    <div data-testid="game-result-popup" data-tone={tone}>
      <span>{message}</span>
      <button type="button" onClick={onClose}>Close</button>
    </div>
  ),
}))

vi.mock('../../src/components/PausePopup', () => ({
  PausePopup: ({ onResume, onRestart }: { onResume: () => void; onRestart: () => void }) => (
    <div data-testid="pause-popup">
      <button type="button" onClick={onResume}>Resume</button>
      <button type="button" onClick={onRestart}>Restart</button>
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
  mockGameInstance.is_lost.mockReturnValue(false)
  mockGameInstance.lose_reason.mockReturnValue(null)
  mockGameInstance.hp.mockReturnValue(3)
  mockGameInstance.max_hp.mockReturnValue(3)
  mockUseMazeGame.mockReturnValue([
    { game: mockGameInstance, version: 0, loading: false, error: null, damageFlashKey: 0, paused: false },
    mockMove,
    mockRestart,
    mockTogglePause,
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

  it('D-pad buttons are aria-disabled when game is complete', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitForLoad()
    expect(screen.getByRole('button', { name: /move up/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move down/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move left/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move right/i })).toHaveAttribute('aria-disabled', 'true')
  })

  it('GameResultPopup appears when game becomes complete', async () => {
    mockGameInstance.is_complete.mockReturnValue(true)
    renderPage()
    await waitFor(() => expect(screen.getByTestId('game-result-popup')).toBeInTheDocument())
    expect(screen.getByText('You win!')).toBeInTheDocument()
    expect(screen.getByTestId('game-result-popup')).toHaveAttribute('data-tone', 'success')
  })

  it('GameResultPopup shows stranded message + fail tone when game is lost', async () => {
    mockGameInstance.is_lost.mockReturnValue(true)
    mockGameInstance.lose_reason.mockReturnValue('stranded')
    renderPage()
    await waitFor(() => expect(screen.getByTestId('game-result-popup')).toBeInTheDocument())
    expect(screen.getByText("You're stranded!!")).toBeInTheDocument()
    expect(screen.getByTestId('game-result-popup')).toHaveAttribute('data-tone', 'fail')
  })

  it('GameResultPopup shows "You died!" message + fail tone when player is killed', async () => {
    mockGameInstance.is_lost.mockReturnValue(true)
    mockGameInstance.lose_reason.mockReturnValue('killed')
    renderPage()
    await waitFor(() => expect(screen.getByTestId('game-result-popup')).toBeInTheDocument())
    expect(screen.getByText('You died!')).toBeInTheDocument()
    expect(screen.getByTestId('game-result-popup')).toHaveAttribute('data-tone', 'fail')
  })

  it('HP HUD renders maxHp hearts with hp filled and the remainder dimmed', async () => {
    mockGameInstance.hp.mockReturnValue(2)
    mockGameInstance.max_hp.mockReturnValue(5)
    renderPage()
    await waitForLoad()
    const hpHud = screen.getByLabelText('Health')
    expect(hpHud).toBeInTheDocument()
    const hearts = hpHud.querySelectorAll('img')
    expect(hearts.length).toBe(5)
    expect(hearts[0]).toHaveAttribute('alt', 'Health')
    expect(hearts[1]).toHaveAttribute('alt', 'Health')
    expect(hearts[2]).toHaveAttribute('alt', 'Lost health')
    expect(hearts[3]).toHaveAttribute('alt', 'Lost health')
    expect(hearts[4]).toHaveAttribute('alt', 'Lost health')
    expect(hearts[0]).not.toHaveClass('maze-hp-hud-heart--empty')
    expect(hearts[2]).toHaveClass('maze-hp-hud-heart--empty')
  })

  it('HP HUD is not rendered when maxHp is 0', async () => {
    mockGameInstance.max_hp.mockReturnValue(0)
    renderPage()
    await waitForLoad()
    expect(screen.queryByLabelText('Health')).not.toBeInTheDocument()
  })

  it('damage flash overlay is not rendered when damageFlashKey is 0', async () => {
    mockUseMazeGame.mockReturnValue([
      { game: mockGameInstance, version: 0, loading: false, error: null, damageFlashKey: 0, paused: false },
      mockMove,
      mockRestart,
      mockTogglePause,
    ])
    const { container } = renderPage()
    await waitForLoad()
    expect(container.querySelector('.maze-damage-flash')).toBeNull()
  })

  it('damage flash overlay is rendered when damageFlashKey > 0', async () => {
    mockUseMazeGame.mockReturnValue([
      { game: mockGameInstance, version: 1, loading: false, error: null, damageFlashKey: 1, paused: false },
      mockMove,
      mockRestart,
      mockTogglePause,
    ])
    const { container } = renderPage()
    await waitForLoad()
    expect(container.querySelector('.maze-damage-flash')).toBeInTheDocument()
  })

  it('keyboard ignored when game is lost', async () => {
    mockGameInstance.is_lost.mockReturnValue(true)
    mockGameInstance.lose_reason.mockReturnValue('stranded')
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'ArrowUp' })
    expect(mockMove).not.toHaveBeenCalled()
  })

  it('D-pad buttons are aria-disabled when game is lost', async () => {
    mockGameInstance.is_lost.mockReturnValue(true)
    mockGameInstance.lose_reason.mockReturnValue('stranded')
    renderPage()
    await waitForLoad()
    expect(screen.getByRole('button', { name: /move up/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move down/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move left/i })).toHaveAttribute('aria-disabled', 'true')
    expect(screen.getByRole('button', { name: /move right/i })).toHaveAttribute('aria-disabled', 'true')
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

  it('bag shows "empty" when nothing is collected', async () => {
    renderPage()
    await waitForLoad()
    const bag = document.querySelector('.maze-bag')!
    expect(bag.textContent).toMatch(/empty/)
  })

  it('bag shows a key icon for each collected key', async () => {
    mockGameInstance.bag.mockReturnValue([{ type: 'key', id: 0 }, { type: 'key', id: 1 }])
    renderPage()
    await waitForLoad()
    const bag = document.querySelector('.maze-bag')!
    expect(bag.querySelectorAll('img')).toHaveLength(2)
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

  it('keyboard legend includes the pause shortcut', async () => {
    renderPage()
    await waitForLoad()
    const legend = document.querySelector('.maze-shortcuts-hint')!
    expect(legend.textContent).toMatch(/Pause/)
  })

  it('"Space" key toggles pause', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: ' ' })
    expect(mockTogglePause).toHaveBeenCalled()
  })

  it('"Escape" key toggles pause', async () => {
    renderPage()
    await waitForLoad()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(mockTogglePause).toHaveBeenCalled()
  })

  it('D-pad pause button toggles pause', async () => {
    renderPage()
    await waitForLoad()
    await userEvent.click(screen.getByRole('button', { name: /pause/i }))
    expect(mockTogglePause).toHaveBeenCalled()
  })

  it('pause popup is shown only when paused, with Resume and Restart wired', async () => {
    mockUseMazeGame.mockReturnValue([
      { game: mockGameInstance, version: 0, loading: false, error: null, damageFlashKey: 0, paused: true },
      mockMove,
      mockRestart,
      mockTogglePause,
    ])
    renderPage()
    await waitForLoad()
    const popup = screen.getByTestId('pause-popup')
    await userEvent.click(within(popup).getByRole('button', { name: 'Resume' }))
    expect(mockTogglePause).toHaveBeenCalled()
    await userEvent.click(within(popup).getByRole('button', { name: 'Restart' }))
    expect(mockRestart).toHaveBeenCalled()
  })

  it('pause popup is absent when not paused', async () => {
    renderPage()
    await waitForLoad()
    expect(screen.queryByTestId('pause-popup')).not.toBeInTheDocument()
  })
})

describe('MazeGamePage per-cell overrides', () => {
  beforeEach(() => {
    mockSplitDefinition.mockResolvedValue({ grid: [], overrides: [] })
  })

  it('hands MazeGrid the pure-char grid (not the array form) for an override maze', async () => {
    // The stored definition has array-form cells; the codec splits it into a pure-char
    // grid. MazeGrid must receive the split grid, not the raw array form — the
    // regression where it received the array form rendered the static cells empty.
    mockSplitDefinition.mockResolvedValue({
      grid: [['S', 'H', 'K', 'D', 'F']],
      overrides: [{ row: 0, col: 1, entity: { type: 'H', healthStyle: 'potion' } }],
    })
    renderPage(mockMazeOverrideStatic.id)
    await waitForLoad()

    const grid = JSON.parse(screen.getByTestId('maze-grid').getAttribute('data-grid')!) as unknown[][]
    expect(grid.flat().every(c => typeof c === 'string')).toBe(true)
    expect(grid).toEqual([['S', 'H', 'K', 'D', 'F']])
    expect(mockSplitDefinition).toHaveBeenCalled()
  })

  it('does not run the codec for a no-override maze (grid passed through as-is)', async () => {
    renderPage(mockMazeAlpha.id)
    await waitForLoad()
    expect(mockSplitDefinition).not.toHaveBeenCalled()
    const grid = JSON.parse(screen.getByTestId('maze-grid').getAttribute('data-grid')!) as unknown[][]
    expect(grid.flat().every(c => typeof c === 'string')).toBe(true)
  })
})
