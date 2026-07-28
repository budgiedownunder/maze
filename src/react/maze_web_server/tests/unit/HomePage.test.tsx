import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { HomePage } from '../../src/pages/HomePage'
import { server } from '../../src/mocks/server'
import { launchDefinition } from '../../src/utils/play3dLaunch'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

vi.mock('../../src/utils/play3dLaunch', () => ({
  launchDefinition: vi.fn(),
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

// A daily-member game inside the curated "Daily Challenges" collection.
const dailyGame = {
  id: 'dg1', ownerId: 'admin', name: 'Daily Maze', visibility: 'curated',
  seed: 1, rotation: 'daily', config: {}, createdAt: 'x', updatedAt: 'x',
}
const dailyCollection = {
  id: 'col-daily', ownerId: 'admin', name: 'Daily Challenges', visibility: 'curated',
  playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x',
}

// Featured catalogue holding the Daily Challenges collection + its detail with the
// daily member — the happy path the Today's Challenge tile resolves.
function dailyChallengeHandlers() {
  return [
    http.get('/api/v1/featured-game-items', () =>
      HttpResponse.json({ items: [{ kind: 'collection', ownerUsername: 'admin', collection: dailyCollection }], limit: 20, offset: 0, hasMore: false }),
    ),
    http.get('/api/v1/game-collections/:id', () =>
      HttpResponse.json({ ...dailyCollection, definitions: [dailyGame] }),
    ),
  ]
}

function renderHomePage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <HomePage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  vi.clearAllMocks()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('HomePage', () => {
  it('renders the tile titles', () => {
    renderHomePage()
    expect(screen.getByRole('heading', { name: /today's challenge/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^3d games$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^3d game workshop$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^mazes$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^leaderboards$/i })).toBeInTheDocument()
  })

  it("Today's Challenge resolves the Daily Challenges collection and launches its daily member", async () => {
    server.use(...dailyChallengeHandlers())
    renderHomePage()

    await userEvent.click(screen.getByRole('button', { name: /today's challenge/i }))

    await waitFor(() => expect(launchDefinition).toHaveBeenCalledWith('dg1'))
    // The tile launches the game, it doesn't route anywhere.
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it("Today's Challenge alerts gracefully when no daily collection is featured", async () => {
    server.use(
      http.get('/api/v1/featured-game-items', () =>
        HttpResponse.json({ items: [], limit: 20, offset: 0, hasMore: false }),
      ),
    )
    renderHomePage()

    await userEvent.click(screen.getByRole('button', { name: /today's challenge/i }))

    await waitFor(() => expect(screen.getByRole('dialog', { name: 'Daily Challenge' })).toBeInTheDocument())
    expect(launchDefinition).not.toHaveBeenCalled()
  })

  it('clicking 3D Game Workshop navigates to /workshop', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /3d game workshop/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop')
  })

  it('clicking 3D Games navigates to the Play-3D hub', async () => {
    renderHomePage()
    // Targeted by the tile's unique description — the Workshop tile's copy also
    // mentions "3D games".
    await userEvent.click(screen.getByRole('button', { name: /browse and play 3d games/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d')
  })

  it('clicking Mazes navigates to /mazes', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /mazes/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/mazes')
  })

  it('clicking Leaderboards navigates to /leaderboards', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /your times and how you rank/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/leaderboards')
  })
})
