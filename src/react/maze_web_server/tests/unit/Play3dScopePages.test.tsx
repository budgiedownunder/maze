import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { type ReactElement } from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dMyGamesPage } from '../../src/pages/Play3dMyGamesPage'
import { Play3dSharedPage } from '../../src/pages/Play3dSharedPage'
import { Play3dCommunityPage } from '../../src/pages/Play3dCommunityPage'
import { server } from '../../src/mocks/server'
import type { GameCollection, GameDefinition } from '../../src/types/api'

const ME = 'me-user'

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: ME, username: 'me', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

const launchDefinition = vi.fn()
vi.mock('../../src/utils/play3dLaunch', () => ({ launchDefinition: (id: string) => launchDefinition(id) }))

function def(overrides: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: ME, visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...overrides }
}
function col(overrides: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: ME, visibility: 'private', playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x', ...overrides }
}

interface Sortable { ownerId: string; visibility: string; name: string; createdAt: string; id: string }

// A pair of list handlers standing in for the server: they honour scope, and (as
// the server does for the public scope) the `q` filter and `sort` ordering — so a
// test can assert the page asks for the right things, and a server-side search
// can't be faked by a client-side filter.
function lists(defs: GameDefinition[], cols: GameCollection[]) {
  const apply = <T extends Sortable>(items: T[], url: URL): T[] => {
    const scope = url.searchParams.get('scope')
    const q = (url.searchParams.get('q') ?? '').toLowerCase()
    const sort = url.searchParams.get('sort')
    let out = items
    if (scope === 'mine') out = out.filter(i => i.ownerId === ME)
    if (scope === 'shared') out = out.filter(i => i.ownerId !== ME && i.visibility === 'shared')
    if (scope === 'public') out = out.filter(i => i.ownerId !== ME && i.visibility === 'public')
    if (q !== '') out = out.filter(i => i.name.toLowerCase().includes(q))
    return [...out].sort((a, b) => (sort === 'newest'
      ? b.createdAt.localeCompare(a.createdAt) || a.id.localeCompare(b.id)
      : a.name.localeCompare(b.name)))
  }
  return [
    http.get('/api/v1/game-definitions', ({ request }) =>
      HttpResponse.json({ definitions: apply(defs, new URL(request.url)), limit: 20, offset: 0, hasMore: false })),
    http.get('/api/v1/game-collections', ({ request }) =>
      HttpResponse.json({ collections: apply(cols, new URL(request.url)), limit: 20, offset: 0, hasMore: false })),
  ]
}

// Rendered card names, in DOM order — so a test can assert the sort, not just
// membership.
function cardNames(): (string | null)[] {
  return [...document.querySelectorAll('.play3d-card-name')].map(e => e.textContent)
}

function renderPage(page: ReactElement) {
  return render(<MemoryRouter><ThemeProvider>{page}</ThemeProvider></MemoryRouter>)
}

beforeEach(() => { vi.clearAllMocks() })
afterEach(() => { vi.unstubAllGlobals() })

describe('Play3dMyGamesPage', () => {
  it('shows the caller’s own games on the Games tab and collections on the Collections tab', async () => {
    server.use(...lists(
      [def({ id: 'd1', name: 'My Game' }), def({ id: 'd2', name: 'Not Mine', ownerId: 'other' })],
      [col({ id: 'c1', name: 'My Set' })],
    ))
    renderPage(<Play3dMyGamesPage />)

    expect(screen.getByText('My Games')).toBeInTheDocument()
    // Games tab is active by default → only my game, not the other user's.
    await waitFor(() => expect(screen.getByText('My Game')).toBeInTheDocument())
    expect(screen.queryByText('Not Mine')).not.toBeInTheDocument()

    // Switch to the Collections tab.
    await userEvent.click(screen.getByRole('tab', { name: 'Collections' }))
    await waitFor(() => expect(screen.getByText('My Set')).toBeInTheDocument())
    expect(screen.queryByText('My Game')).not.toBeInTheDocument()
  })

  it('Play launches the selected game', async () => {
    server.use(...lists([def({ id: 'd1', name: 'My Game' })], []))
    renderPage(<Play3dMyGamesPage />)
    await waitFor(() => expect(screen.getByText('My Game')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Play My Game' }))
    expect(launchDefinition).toHaveBeenCalledWith('d1')
  })

  it('shows an empty state when the caller has no games', async () => {
    server.use(...lists([], []))
    renderPage(<Play3dMyGamesPage />)
    await waitFor(() => expect(screen.getByText(/haven't created any 3D games/i)).toBeInTheDocument())
  })
})

describe('Play3dCommunityPage', () => {
  // Names run opposite to creation order so A–Z and Newest can't both be
  // satisfied by one ordering.
  const publicDefs = () => [
    def({ id: 'p1', name: 'Community Classic', ownerId: 'other', visibility: 'public', createdAt: '2026-01-01' }),
    def({ id: 'p2', name: 'Zephyr Heights', ownerId: 'other', visibility: 'public', createdAt: '2026-02-01' }),
    def({ id: 'd1', name: 'My Public', visibility: 'public' }),
  ]

  it('lists other users’ public games, not the caller’s own', async () => {
    server.use(...lists(publicDefs(), []))
    renderPage(<Play3dCommunityPage />)

    expect(screen.getByText('Community')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('Community Classic')).toBeInTheDocument())
    expect(screen.getByText('Zephyr Heights')).toBeInTheDocument()
    // The caller's own public game belongs to My Games, not the Community pool.
    expect(screen.queryByText('My Public')).not.toBeInTheDocument()
  })

  it('searches server-side (the loader only returns matches)', async () => {
    server.use(...lists(publicDefs(), []))
    renderPage(<Play3dCommunityPage />)
    await waitFor(() => expect(screen.getByText('Community Classic')).toBeInTheDocument())

    await userEvent.type(screen.getByLabelText('Search games…'), 'Zephyr')
    // Debounced refetch — settle on the narrowed page, not the loading flash.
    await waitFor(() => {
      expect(screen.queryByText('Community Classic')).not.toBeInTheDocument()
      expect(screen.getByText('Zephyr Heights')).toBeInTheDocument()
    })
  })

  it('the sort control reorders the catalogue by newest', async () => {
    server.use(...lists(publicDefs(), []))
    renderPage(<Play3dCommunityPage />)
    await waitFor(() => expect(cardNames()).toEqual(['Community Classic', 'Zephyr Heights']))

    await userEvent.selectOptions(screen.getByLabelText('Sort'), 'newest')
    await waitFor(() => expect(cardNames()).toEqual(['Zephyr Heights', 'Community Classic']))
  })

  it('offers no sort control on a bounded scope', async () => {
    server.use(...lists([def({ id: 'd1', name: 'My Game' })], []))
    renderPage(<Play3dMyGamesPage />)
    await waitFor(() => expect(screen.getByText('My Game')).toBeInTheDocument())
    expect(screen.queryByLabelText('Sort')).not.toBeInTheDocument()
  })
})

describe('Play3dSharedPage', () => {
  it('shows games + collections shared with the caller, not their own', async () => {
    server.use(...lists(
      [def({ id: 's1', name: 'Shared Game', ownerId: 'friend', visibility: 'shared' }), def({ id: 'd1', name: 'Mine' })],
      [col({ id: 'sc1', name: 'Shared Set', ownerId: 'friend', visibility: 'shared' })],
    ))
    renderPage(<Play3dSharedPage />)

    expect(screen.getByText('Shared with me')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('Shared Game')).toBeInTheDocument())
    expect(screen.queryByText('Mine')).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('tab', { name: 'Collections' }))
    await waitFor(() => expect(screen.getByText('Shared Set')).toBeInTheDocument())
  })
})
