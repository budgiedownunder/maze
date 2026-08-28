import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { LeaderboardsPage } from '../../src/pages/LeaderboardsPage'
import { server } from '../../src/mocks/server'
import { launchPlay3dWithSettings, launchDefinition } from '../../src/utils/play3dLaunch'
import { solveMaze } from '../../src/wasm/mazeWasm'
import { todayUtc } from '../../src/utils/gameDefinitions'
import type { ScoreEntry } from '../../src/types/api'

vi.mock('../../src/utils/play3dLaunch', () => ({
  launchPlay3dWithSettings: vi.fn(),
  launchDefinition: vi.fn(),
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

// Mutable auth profile so a test can flip the caller to an admin (a 3D game's
// board is resettable by its owner or an admin). Reset to a non-admin default
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

function gameDef(id: string, name: string, ownerId: string) {
  return { id, ownerId, name, visibility: 'curated', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' }
}

// The play-fetch shape: the definition plus its computed challenge key.
function playResponse(id: string, name: string, ownerId: string) {
  return { ...gameDef(id, name, ownerId), challengeKey: `def:${id}`, leaderboardTracked: true }
}

// History whose most-recent run is on the stored game `abc-123`, that game's
// play-fetch, and its board — the fixture behind the def-default tests.
function defDefaultHandlers(ownerId = 'owner-x', boardRows: ScoreEntry[] = []) {
  return [
    http.get('/api/v1/scores/me', () =>
      HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'def:abc-123', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
    ),
    http.get('/api/v1/mazes', () => HttpResponse.json([])),
    http.get('/api/v1/game-definitions/:id', ({ params }) =>
      HttpResponse.json(playResponse(String(params.id), 'Tricky', ownerId)),
    ),
    http.get('/api/v1/scores', ({ request }) => {
      const challenge = new URL(request.url).searchParams.get('challenge')
      return HttpResponse.json({
        scores: challenge === 'def:abc-123' ? boardRows : [],
        limit: 20, offset: 0, has_more: false,
      })
    }),
  ]
}

// History whose most-recent run is on a *daily* stored game, its play-fetch
// (rotation 'daily'), its dated boards, and a board handler that records every
// challenge key the page requests — the fixture behind the daily date-picker
// tests.
function dailyDefaultHandlers(boardDates: string[] = ['2026-07-10', '2026-07-05']) {
  const requested: string[] = []
  return {
    requested,
    handlers: [
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'def:daily-1', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () => HttpResponse.json([])),
      http.get('/api/v1/game-definitions/:id', ({ params }) =>
        HttpResponse.json({ ...gameDef(String(params.id), 'Daily Maze', 'owner-x'), rotation: 'daily', challengeKey: `def:${String(params.id)}:${todayUtc()}`, leaderboardTracked: true }),
      ),
      http.get('/api/v1/scores/board-dates', () => HttpResponse.json({ dates: boardDates })),
      http.get('/api/v1/scores', ({ request }) => {
        const challenge = new URL(request.url).searchParams.get('challenge')
        if (challenge) requested.push(challenge)
        return HttpResponse.json({
          scores: [row({ id: 's1', challenge, user_id: 'me', username: 'bob', elapsed_ms: 42137 })],
          limit: 20, offset: 0, has_more: false,
        })
      }),
    ],
  }
}

// `url` carries the optional `?id=` / `?def=` preselect the page reads on mount.
function renderPage(url = '/leaderboards') {
  return render(
    <MemoryRouter initialEntries={[url]}>
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

  it('defaults to the stored game behind a def: most-recent run', async () => {
    server.use(...defDefaultHandlers('owner-x', [
      row({ id: 's1', challenge: 'def:abc-123', user_id: 'me', username: 'bob', elapsed_ms: 42137 }),
    ]))
    renderPage()

    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('play3d')
    // The picker summary names the resolved game.
    expect(screen.getByText('Tricky')).toBeInTheDocument()
  })

  it('falls back to a maze when the most-recent def: game is gone', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', challenge: 'def:abc-123', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      // The game was deleted (or is no longer visible) — the play-fetch 404s.
      http.get('/api/v1/game-definitions/:id', () => new HttpResponse(null, { status: 404 })),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage()

    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('my-mazes')
  })

  it('opens the maze named by ?id, in preference to the most-recent run', async () => {
    const requested: Array<string | null> = []
    server.use(
      // The most-recent run is on m1 — the preselect must win over it.
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([
          { id: 'm1.json', name: 'My Maze', definition: null },
          { id: 'm2.json', name: 'Other Maze', definition: null },
        ]),
      ),
      http.get('/api/v1/scores', ({ request }) => {
        requested.push(new URL(request.url).searchParams.get('maze_id'))
        return HttpResponse.json({
          scores: [row({ id: 's1', maze_id: 'm2.json', user_id: 'me', elapsed_ms: 42137 })],
          limit: 20, offset: 0, has_more: false,
        })
      }),
    )
    renderPage('/leaderboards?id=m2.json')

    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('my-mazes')
    expect((screen.getByLabelText('Game') as HTMLSelectElement).value).toBe('m2.json')
    expect(requested).toEqual(['m2.json'])
  })

  it('opens the stored game named by ?def, in preference to the most-recent run', async () => {
    server.use(
      // The most-recent run is on a maze — the preselect must win over it.
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/game-definitions/:id', ({ params }) =>
        HttpResponse.json(playResponse(String(params.id), 'Tricky', 'owner-x')),
      ),
      http.get('/api/v1/scores', ({ request }) =>
        HttpResponse.json({
          scores: new URL(request.url).searchParams.get('challenge') === 'def:abc-123'
            ? [row({ id: 's1', challenge: 'def:abc-123', user_id: 'me', username: 'bob', elapsed_ms: 42137 })]
            : [],
          limit: 20, offset: 0, has_more: false,
        }),
      ),
    )
    renderPage('/leaderboards?def=abc-123')

    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('play3d')
    expect(screen.getByText('Tricky')).toBeInTheDocument()
  })

  it('falls back to the usual default when the ?def game is gone', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      // Deleted, or no longer visible to the caller — the play-fetch 404s.
      http.get('/api/v1/game-definitions/:id', () => new HttpResponse(null, { status: 404 })),
      http.get('/api/v1/scores', () =>
        HttpResponse.json({ scores: [row({ id: 's1', maze_id: 'm1.json', user_id: 'me', elapsed_ms: 42137 })], limit: 20, offset: 0, has_more: false }),
      ),
    )
    renderPage('/leaderboards?def=abc-123')

    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    expect((screen.getByLabelText('Game Type') as HTMLSelectElement).value).toBe('my-mazes')
  })

  it('sets the busy cursor while loading and clears it when done', async () => {
    // Uses the default MSW score handlers.
    renderPage()
    expect(document.body.classList.contains('is-busy')).toBe(true)
    await waitFor(() => expect(screen.getByText('0:42.137')).toBeInTheDocument())
    await waitFor(() => expect(document.body.classList.contains('is-busy')).toBe(false))
  })

  it('switching to 3D Games prompts for a game until one is picked', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
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

    await userEvent.selectOptions(screen.getByLabelText('Game Type'), 'play3d')

    expect(screen.getByText('Choose a game to see its leaderboard.')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '▶ Play' })).toBeDisabled()
  })

  it('picking a featured game shows its board with usernames', async () => {
    server.use(
      http.get('/api/v1/scores/me', () =>
        HttpResponse.json({ scores: [row({ id: 'h1', maze_id: 'm1.json', user_id: 'me' })], limit: 100, offset: 0, has_more: false }),
      ),
      http.get('/api/v1/mazes', () =>
        HttpResponse.json([{ id: 'm1.json', name: 'My Maze', definition: null }]),
      ),
      http.get('/api/v1/featured-game-items', () =>
        HttpResponse.json({ items: [{ kind: 'definition', definition: gameDef('g1', 'Tricky', 'admin') }], limit: 20, offset: 0, hasMore: false }),
      ),
      http.get('/api/v1/scores', ({ request }) => {
        const challenge = new URL(request.url).searchParams.get('challenge')
        if (challenge === 'def:g1') {
          return HttpResponse.json({
            scores: [
              row({ id: 'a', challenge: 'def:g1', user_id: 'other', username: 'alice', elapsed_ms: 31204, score: 9 }),
              row({ id: 'b', challenge: 'def:g1', user_id: 'me', username: 'bob', elapsed_ms: 42137, score: 7 }),
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
    await userEvent.click(screen.getByRole('button', { name: 'Choose a game' }))
    await userEvent.click(await screen.findByRole('button', { name: 'Show leaderboard for Tricky' }))

    // Scope username assertions to the board table — the caller's username also
    // appears in the page header (the account link), so a page-wide query is
    // ambiguous.
    const board = await screen.findByRole('table')
    await waitFor(() => expect(within(board).getByText('alice')).toBeInTheDocument())
    // A 3D game's board shows every player's username (not "You").
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
    expect(launchDefinition).not.toHaveBeenCalled()
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

  it('shows a date dropdown defaulting to the most-recent day with runs for a daily game', async () => {
    const { handlers, requested } = dailyDefaultHandlers()
    server.use(...handlers)
    renderPage()

    // Defaults to the daily game behind the most-recent run.
    await waitFor(() => expect(screen.getByText('Daily Maze')).toBeInTheDocument())
    const select = (await screen.findByLabelText('Day')) as HTMLSelectElement
    // Defaults to the newest day that has runs (2026-07-10), not today.
    expect(select.value).toBe('2026-07-10')
    await waitFor(() => expect(requested).toContain('def:daily-1:2026-07-10'))
    // Never loaded today's (empty) board first.
    expect(requested).not.toContain(`def:daily-1:${todayUtc()}`)
    // Today is still offered (pinned first), then the days with runs.
    expect(Array.from(select.options).map(o => o.value)).toEqual([todayUtc(), '2026-07-10', '2026-07-05'])
  })

  it('re-keys the board to a past day when another day is selected', async () => {
    const { handlers, requested } = dailyDefaultHandlers()
    server.use(...handlers)
    renderPage()

    const select = (await screen.findByLabelText('Day')) as HTMLSelectElement
    await userEvent.selectOptions(select, '2026-07-05')

    // The board now asks for that day's dated challenge, and the select reflects it.
    await waitFor(() => expect(requested).toContain('def:daily-1:2026-07-05'))
    expect(select.value).toBe('2026-07-05')
  })

  it('shows no date picker for a static game', async () => {
    server.use(...defDefaultHandlers())
    renderPage()
    await waitFor(() => expect(screen.getByText('Tricky')).toBeInTheDocument())
    expect(screen.queryByLabelText('Day')).not.toBeInTheDocument()
  })

  it('Play launches the selected 3D game by id', async () => {
    server.use(...defDefaultHandlers())
    renderPage()
    await waitFor(() => expect(screen.getByRole('button', { name: '▶ Play' })).toBeEnabled())

    await userEvent.click(screen.getByRole('button', { name: '▶ Play' }))

    expect(launchDefinition).toHaveBeenCalledWith('abc-123')
    expect(launchPlay3dWithSettings).not.toHaveBeenCalled()
  })

  it('disables Play when the Mazes game type has no mazes', async () => {
    server.use(...defDefaultHandlers())
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

  it('hides Reset on another user’s game board for a non-admin', async () => {
    server.use(...defDefaultHandlers('owner-x', [
      row({ id: 'a', challenge: 'def:abc-123', user_id: 'other', username: 'alice', elapsed_ms: 31204 }),
    ]))
    renderPage()
    await waitFor(() => expect(screen.getByText('0:31.204')).toBeInTheDocument())
    // A board with rows, but the game is someone else's and the caller isn't an
    // admin → no Reset.
    expect(screen.queryByRole('button', { name: 'Reset leaderboard' })).not.toBeInTheDocument()
  })

  it('shows Reset on the caller’s own game board without admin', async () => {
    // The caller owns the game → they may reset its board.
    server.use(...defDefaultHandlers('me', [
      row({ id: 'a', challenge: 'def:abc-123', user_id: 'other', username: 'alice', elapsed_ms: 31204 }),
    ]))
    renderPage()
    await waitFor(() => expect(screen.getByText('0:31.204')).toBeInTheDocument())
    expect(await screen.findByRole('button', { name: 'Reset leaderboard' })).toBeInTheDocument()
  })

  it('shows Reset on another user’s game board for an admin', async () => {
    authState.profile = { id: 'me', username: 'bob', is_admin: true }
    server.use(...defDefaultHandlers('owner-x', [
      row({ id: 'a', challenge: 'def:abc-123', user_id: 'other', username: 'alice', elapsed_ms: 31204 }),
    ]))
    renderPage()
    await waitFor(() => expect(screen.getByText('0:31.204')).toBeInTheDocument())
    expect(await screen.findByRole('button', { name: 'Reset leaderboard' })).toBeInTheDocument()
  })
})
