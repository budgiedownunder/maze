import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { LeaderboardsPage } from '../../src/pages/LeaderboardsPage'
import { server } from '../../src/mocks/server'
import { launchPlay3dWithSettings, launchPlay3dCurated } from '../../src/utils/play3dLaunch'
import type { ScoreEntry } from '../../src/types/api'

vi.mock('../../src/utils/play3dLaunch', () => ({
  launchPlay3dWithSettings: vi.fn(),
  launchPlay3dCurated: vi.fn(),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => vi.fn() }
})

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: true,
      profile: { id: 'me', username: 'bob' },
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

    await waitFor(() => expect(screen.getByText('alice')).toBeInTheDocument())
    // The caller's own row shows their username (not "You"), on the Player column.
    expect(screen.getByText('bob')).toBeInTheDocument()
    expect(screen.queryByText('You')).not.toBeInTheDocument()
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

    expect(launchPlay3dWithSettings).toHaveBeenCalledWith('m1.json', expect.any(Object))
    expect(launchPlay3dCurated).not.toHaveBeenCalled()
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
    await waitFor(() => expect(screen.getByRole('button', { name: 'Play' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Play' }))

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

    await waitFor(() => expect(screen.getByRole('button', { name: 'Play' })).toBeDisabled())
  })
})
