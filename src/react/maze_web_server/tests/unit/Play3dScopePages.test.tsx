import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { type ReactElement } from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dMyGamesPage } from '../../src/pages/Play3dMyGamesPage'
import { Play3dSharedPage } from '../../src/pages/Play3dSharedPage'
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

// A pair of list handlers that honour the requested scope, so a test can assert
// the page requests (and renders) the right scope per tab.
function lists(defs: GameDefinition[], cols: GameCollection[]) {
  const inScope = <T extends { ownerId: string; visibility: string }>(items: T[], scope: string | null) => {
    if (scope === 'mine') return items.filter(i => i.ownerId === ME)
    if (scope === 'shared') return items.filter(i => i.ownerId !== ME && i.visibility === 'shared')
    return items
  }
  return [
    http.get('/api/v1/game-definitions', ({ request }) => {
      const items = inScope(defs, new URL(request.url).searchParams.get('scope')).sort((a, b) => a.name.localeCompare(b.name))
      return HttpResponse.json({ definitions: items, limit: 20, offset: 0, hasMore: false })
    }),
    http.get('/api/v1/game-collections', ({ request }) => {
      const items = inScope(cols, new URL(request.url).searchParams.get('scope')).sort((a, b) => a.name.localeCompare(b.name))
      return HttpResponse.json({ collections: items, limit: 20, offset: 0, hasMore: false })
    }),
  ]
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
