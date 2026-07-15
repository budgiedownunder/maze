import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dFeaturedPage } from '../../src/pages/Play3dFeaturedPage'
import { server } from '../../src/mocks/server'
import type { FeaturedGameItem, GameCollection, GameDefinition } from '../../src/types/api'
import { launchDefinition } from '../../src/utils/play3dLaunch'

vi.mock('../../src/utils/play3dLaunch', () => ({ launchDefinition: vi.fn() }))

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: 'me', username: 'me', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

function def(over: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: 'admin', visibility: 'curated', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...over }
}
function col(over: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: 'admin', visibility: 'curated', playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x', ...over }
}
function defItem(d: GameDefinition): FeaturedGameItem { return { kind: 'definition', ownerUsername: 'admin', definition: d } }
function colItem(c: GameCollection): FeaturedGameItem { return { kind: 'collection', ownerUsername: 'admin', collection: c } }

function featuredOf(...items: FeaturedGameItem[]) {
  return http.get('/api/v1/featured-game-items', ({ request }) => {
    const url = new URL(request.url)
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    return HttpResponse.json({ items: items.slice(offset, offset + limit), limit, offset, hasMore: offset + limit < items.length })
  })
}

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <Play3dFeaturedPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => vi.clearAllMocks())

describe('Play3dFeaturedPage', () => {
  it('renders a card per featured item with Play (and Leaderboard for games)', async () => {
    server.use(featuredOf(
      defItem(def({ id: 'd1', name: 'Night Climb', description: 'A 3-level ascent' })),
      colItem(col({ id: 'c1', name: 'Difficulty', items: [
        { definitionId: 'e', sortOrder: 0 }, { definitionId: 't', sortOrder: 1 },
      ] })),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Night Climb')).toBeInTheDocument())
    expect(screen.getByText('A 3-level ascent')).toBeInTheDocument()
    expect(screen.getByText('Difficulty')).toBeInTheDocument()

    // A game card has Play + Leaderboard, each with its icon.
    const play = screen.getByRole('button', { name: 'Play Night Climb' })
    expect(play.querySelector('img')).toHaveAttribute('src', '/images/icons/icon_play_3d.png')
    const board = screen.getByRole('button', { name: 'Leaderboard for Night Climb' })
    expect(board.querySelector('img')).toHaveAttribute('src', '/images/icons/icon_leaderboard.svg')
    // A collection card has no Leaderboard.
    expect(screen.queryByRole('button', { name: 'Leaderboard for Difficulty' })).toBeNull()
  })

  it('Play launches a game definition', async () => {
    server.use(featuredOf(defItem(def({ id: 'd1', name: 'Night Climb' }))))
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Night Climb' }))
    expect(launchDefinition).toHaveBeenCalledWith('d1')
  })

  it('Play on a single-game collection launches its sole member; a multi-game collection is disabled', async () => {
    server.use(featuredOf(
      colItem(col({ id: 'c1', name: 'Solo Set', items: [{ definitionId: 'only', sortOrder: 0 }] })),
      colItem(col({ id: 'c2', name: 'Difficulty', items: [
        { definitionId: 'e', sortOrder: 0 }, { definitionId: 't', sortOrder: 1 },
      ] })),
    ))
    renderPage()
    // Single-game collection → launches its member.
    await userEvent.click(await screen.findByRole('button', { name: 'Play Solo Set' }))
    expect(launchDefinition).toHaveBeenCalledWith('only')
    // Multi-game collection → Play disabled with the coming-soon hint.
    const multi = screen.getByRole('button', { name: 'Play Difficulty' })
    expect(multi).toBeDisabled()
    expect(multi).toHaveAttribute('title', expect.stringMatching(/coming soon/i))
  })

  it('Leaderboard opens the game leaderboard modal', async () => {
    server.use(
      featuredOf(defItem(def({ id: 'd1', name: 'Night Climb' }))),
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...def({ id: 'd1', name: 'Night Climb' }), challengeKey: 'def:d1', leaderboardTracked: true })),
      http.get('/api/v1/scores', () => HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Leaderboard for Night Climb' }))
    expect(await screen.findByRole('dialog', { name: 'Leaderboard: Night Climb' })).toBeInTheDocument()
  })

  it('filters featured items client-side', async () => {
    server.use(featuredOf(
      defItem(def({ id: 'd1', name: 'Alpha' })),
      defItem(def({ id: 'd2', name: 'Beta' })),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Alpha')).toBeInTheDocument())
    await userEvent.type(screen.getByLabelText('Filter featured…'), 'bet')
    expect(screen.getByText('Beta')).toBeInTheDocument()
    expect(screen.queryByText('Alpha')).toBeNull()
  })

  it('shows the empty state', async () => {
    server.use(featuredOf())
    renderPage()
    await waitFor(() => expect(screen.getByText('No featured games or collections yet.')).toBeInTheDocument())
  })
})
