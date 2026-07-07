import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { LeaderboardsPage } from '../../src/pages/LeaderboardsPage'
import { server } from '../../src/mocks/server'
import { launchPlay3dWithSettings, launchPlay3dCurated } from '../../src/utils/play3dLaunch'
import { solveMaze } from '../../src/wasm/mazeWasm'
import type { ScoreEntry } from '../../src/types/api'

vi.mock('../../src/utils/play3dLaunch', () => ({
  launchPlay3dWithSettings: vi.fn(),
  launchPlay3dCurated: vi.fn(),
}))

// The Play button gates a personal maze through `solveMaze` (the shared
// solvability check) before launching, so stub the WASM solver.
vi.mock('../../src/wasm/mazeWasm', () => ({
  solveMaze: vi.fn(),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => vi.fn() }
})

// Mutable auth profile so a test can flip the caller to an admin (the Reset
// button on a global Play-3D board is admin-gated). Reset to a non-admin default
// in beforeEach.
const { authState } = vi.hoisted(() => ({
  authState: { profile: { id: 'me', username: 'bob' } as { id: string; username: string; is_admin?: boolean } },
}))

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: true,
      profile: authState.profile,
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

function row(over: Partial<ScoreEntry>): ScoreEntry {
  return {
    id: 'id', user_id: 'u', maze_id: null, challenge: null,
    score: 1, elapsed_ms: 1000, recorded_at: '2025-04-01T12:00:00Z', ...over,
  }
}

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <LeaderboardsPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  authState.profile = { id: 'me', username: 'bob' }
  // Default: mazes solve (are playable); a test overrides this to reject.
  ;(solveMaze as Mock).mockResolvedValue([{ row: 0, col: 0 }])
})

describe('LeaderboardsPage', () => {
  it('defaults to the most-recent subject and renders its board', async () => {
    server.use(
      // History: most recent run is on maze m1.json.
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({
          scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137, score: 7 })],
          limit: 20, offset: 0, has_more: false,
        }),
      ),
    )
    renderPage()

    // The maze name shows in the selector and the board shows its time.
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('my-mazes')
    expect(screen.getByRole('option', { name: 'My Maze' })).toBeInTheDocument()
    // My-Mazes board hides the player column.
    expect(screen.queryByText('Player')).not.toBeInTheDocument()
  })

  it('falls back from a game-definition (def:) most-recent run instead of resolving it as a difficulty', async () => {
    const play3dDifficulties: string[] = []
    server.use(
      // Most recent run is a stored game definition — challenge "def:<id>".
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'def:abc-123', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/game/play3d-config', ({ request }) => {
        play3dDifficulties.push(new URL(request.url).searchParams.get('difficulty') ?? '')
        return HttpResponse.json({ difficulty: 'easy', seed: 42 })
      }),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()

    // Defaults to the player's maze board, not a bogus "def" play-3D difficulty
    // (which the play3d-config endpoint would reject).
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('my-mazes')
    expect(play3dDifficulties).not.toContain('def')
  })

  it('sets the busy cursor while loading and clears it when done', async () => {
    // Uses the default MSW score handlers.
    renderPage()
    expect(document.body.classList.contains('is-busy')).toBe(true)
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    await waitFor(() => expect(document.body.classList.contains('is-busy')).toBe(false))
  })

  it('switching to Play 3D resolves the seed and shows a board with usernames', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/game/play3d-config', () =>
        HttpResponse.json({ difficulty: 'easy', seed: 42 }),
      ),
      // The leaderboard for the easy:42 board, with another player + the caller.
      http.get('/api/v1/scores', ({ request }) => {
        const challenge = new URL(request.url).searchParams.get('challenge')
        if (challenge === 'easy:42') {
          return HttpResponse.json({
            scores: [
              row({ id: 'a', challenge: 'easy:42', user_id: 'other', username: 'alice', elapsed_ms: 31204, score: 9 }),
              row({ id: 'b', challenge: 'easy:42', user_id: 'me', username: 'bob', elapsed_ms: 42137, score: 7 }),
            ],
            limit: 20, offset: 0, has_more: false,
          })
        }
        return HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false })
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByLabelText('Game Type')).toBeInTheDocument())

    await userEvent.selectOptions(screen.getByLabelText('Game Type'), 'play3d')

    // Scope username assertions to the board table — the caller's username also
    // appears in the page header (the account link), so a page-wide query is
    // ambiguous.
    const board = await screen.findByRole('table')
    await waitFor(() => expect(within(board).getByText('alice')).toBeInTheDocument())
    // The caller's own row shows their username (not "You"), on the Player column.
    expect(within(board).getByText('bob')).toBeInTheDocument()
    expect(within(board).queryByText('You')).not.toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Player' })).toBeInTheDocument()
  })

  it('Play launches the selected maze in 3D with its settings', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()
    // The caller has a run on this maze → the button offers "Play Again".
    await waitFor(() => expect(screen.getByRole('button', { name: '↻ Play Again' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '↻ Play Again' }))

    // The launch is gated behind the async solvability check, so wait for it.
    await waitFor(() => expect(launchPlay3dWithSettings).toHaveBeenCalledWith('m1.json', expect.any(Object)))
    expect(launchPlay3dCurated).not.toHaveBeenCalled()
  })

  it('rejects an unplayable (empty/cleared) maze with a Cannot Play Maze alert', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    // A maze with no start/finish fails the solvability check.
    ;(solveMaze as Mock).mockRejectedValue(new Error('No solution found'))
    renderPage()
    await waitFor(() => expect(screen.getByRole('button', { name: '↻ Play Again' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '↻ Play Again' }))

    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Cannot Play Maze' })).toBeInTheDocument())
    expect(launchPlay3dWithSettings).not.toHaveBeenCalled()
  })

  it('Play launches the selected difficulty for a Play 3D subject', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'easy:42', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () => HttpResponse.json([])),
      http.get('/api/v1/game/play3d-config', () => HttpResponse.json({ difficulty: 'easy', seed: 42 })),
      http.get('/api/v1/scores', () => HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByRole('button', { name: '▶ Play' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '▶ Play' }))

    expect(launchPlay3dCurated).toHaveBeenCalledWith('easy')
    expect(launchPlay3dWithSettings).not.toHaveBeenCalled()
  })

  it('disables Play when the Mazes game type has no mazes', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'easy:42', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () => HttpResponse.json([])),
      http.get('/api/v1/game/play3d-config', () => HttpResponse.json({ difficulty: 'easy', seed: 42 })),
      http.get('/api/v1/scores', () => HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByLabelText('Game Type')).toBeInTheDocument())

    // Switch to Mazes — the player has none → nothing to launch → Play disabled.
    await userEvent.selectOptions(screen.getByLabelText('Game Type'), 'my-mazes')

    await waitFor(() => expect(screen.getByRole('button', { name: '▶ Play' })).toBeDisabled())
  })

  it('Refresh reloads the current board', async () => {
    let scoresHits = 0
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () => {
        scoresHits++
        return HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false })
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    const before = scoresHits

    await userEvent.click(screen.getByRole('button', { name: 'Refresh' }))

    await waitFor(() => expect(scoresHits).toBeGreaterThan(before))
  })

  it('shows Reset for a non-empty maze board and clears it after confirming', async () => {
    let cleared = false
    let deleteHits = 0
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () =>
        HttpResponse.json(
          cleared
            ? { scores: [], limit: 20, offset: 0, has_more: false }
            : { scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false },
        ),
      ),
      http.delete('/api/v1/scores', () => {
        deleteHits++
        cleared = true
        return HttpResponse.json({ deleted: 1 })
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())

    // A non-empty maze board the caller owns → Reset is offered (no admin needed).
    // `findByRole` waits for the row count to lift from the board into the page.
    await userEvent.click(await screen.findByRole('button', { name: 'Reset leaderboard' }))

    // Confirm in the modal — the destructive clear only fires on confirm.
    const dialog = await screen.findByRole('dialog', { name: 'Reset leaderboard' })
    await userEvent.click(within(dialog).getByRole('button', { name: 'Reset' }))

    await waitFor(() => expect(deleteHits).toBe(1))
    // The board re-fetches empty → the score row and the Reset button both vanish.
    await waitFor(() => expect(screen.queryByText('0:42.137')).not.toBeInTheDocument())
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Reset leaderboard' })).not.toBeInTheDocument())
  })

  it('hides Reset when the maze board is empty', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()
    await waitFor(() => expect(screen.getByLabelText('Game Type')).toBeInTheDocument())
    await waitFor(() => expect(document.body.classList.contains('is-busy')).toBe(false))
    expect(screen.queryByRole('button', { name: 'Reset leaderboard' })).not.toBeInTheDocument()
  })

  it('hides Reset on a Play 3D board for a non-admin', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'easy:42', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () => HttpResponse.json([])),
      http.get('/api/v1/game/play3d-config', () => HttpResponse.json({ difficulty: 'easy', seed: 42 })),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 'a', challenge: 'easy:42', user_id: 'other', username: 'alice', elapsed_ms: 31204 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('0:31.204')).toBeInTheDocument())
    // A global board with rows, but the caller isn't an admin → no Reset.
    expect(screen.queryByRole('button', { name: 'Reset leaderboard' })).not.toBeInTheDocument()
  })

  it('shows Reset on a Play 3D board for an admin', async () => {
    authState.profile = { id: 'me', username: 'bob', is_admin: true }
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'easy:42', user_id: 'me' })], limit: 1, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () => HttpResponse.json([])),
      http.get('/api/v1/game/play3d-config', () => HttpResponse.json({ difficulty: 'easy', seed: 42 })),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 'a', challenge: 'easy:42', user_id: 'other', username: 'alice', elapsed_ms: 31204 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('0:31.204')).toBeInTheDocument())
    expect(await screen.findByRole('button', { name: 'Reset leaderboard' })).toBeInTheDocument()
  })
})
