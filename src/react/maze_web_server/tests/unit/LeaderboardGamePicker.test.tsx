import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { LeaderboardGamePicker, type PickedGame } from '../../src/components/LeaderboardGamePicker'
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

function def(overrides: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: ME, visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...overrides }
}
function col(overrides: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: ME, visibility: 'private', playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x', ...overrides }
}

// Scope-aware list handlers + a featured catalogue + a collection detail, so the
// picker's four tabs and the collection expand all resolve.
function handlers(opts: {
  featured?: { kind: 'definition' | 'collection'; definition?: GameDefinition; collection?: GameCollection }[]
  defs?: GameDefinition[]
  cols?: GameCollection[]
  members?: Record<string, GameDefinition[]>
} = {}) {
  const { featured = [], defs = [], cols = [], members = {} } = opts
  return [
    http.get('/api/v1/featured-game-items', () =>
      HttpResponse.json({ items: featured, limit: 20, offset: 0, hasMore: false })),
    http.get('/api/v1/game-definitions', ({ request }) => {
      const url = new URL(request.url)
      const scope = url.searchParams.get('scope')
      const q = (url.searchParams.get('q') ?? '').toLowerCase()
      let items = defs.filter(d => (scope === 'mine' ? d.ownerId === ME : d.ownerId !== ME && d.visibility === (scope === 'public' ? 'public' : 'shared')))
      if (q !== '') items = items.filter(d => d.name.toLowerCase().includes(q))
      return HttpResponse.json({ definitions: items, limit: 20, offset: 0, hasMore: false })
    }),
    http.get('/api/v1/game-collections', ({ request }) => {
      const url = new URL(request.url)
      const scope = url.searchParams.get('scope')
      const items = cols.filter(c => (scope === 'mine' ? c.ownerId === ME : c.ownerId !== ME && c.visibility === (scope === 'public' ? 'public' : 'shared')))
      return HttpResponse.json({ collections: items, limit: 20, offset: 0, hasMore: false })
    }),
    http.get('/api/v1/game-collections/:id', ({ params }) => {
      const c = cols.find(x => x.id === params.id)
      if (!c) return new HttpResponse(null, { status: 404 })
      return HttpResponse.json({ ...c, definitions: members[String(params.id)] ?? [] })
    }),
  ]
}

function renderPicker(value: PickedGame | null = null) {
  const onSelect = vi.fn()
  render(<LeaderboardGamePicker value={value} onSelect={onSelect} />)
  return onSelect
}

beforeEach(() => { vi.clearAllMocks() })

describe('LeaderboardGamePicker', () => {
  it('is collapsed until Choose a game, and shows the selected name', async () => {
    server.use(...handlers())
    render(<LeaderboardGamePicker value={{ id: 'd1', name: 'Tricky', ownerId: ME }} onSelect={vi.fn()} />)
    expect(screen.getByText('Tricky')).toBeInTheDocument()
    // The panel (scope tabs) is hidden until expanded.
    expect(screen.queryByRole('tab', { name: 'Featured' })).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Change' }))
    expect(screen.getByRole('tab', { name: 'Featured' })).toBeInTheDocument()
  })

  it('shows "No game selected" + Choose a game with no value', () => {
    server.use(...handlers())
    renderPicker(null)
    expect(screen.getByText('No game selected')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Choose a game' })).toBeInTheDocument()
  })

  it('selects a featured game and collapses', async () => {
    server.use(...handlers({
      featured: [{ kind: 'definition', definition: def({ id: 'f1', name: 'Easy', ownerId: 'admin', visibility: 'curated' }) }],
    }))
    const onSelect = renderPicker(null)
    await userEvent.click(screen.getByRole('button', { name: 'Choose a game' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Show leaderboard for Easy' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Show leaderboard for Easy' }))
    expect(onSelect).toHaveBeenCalledWith({ id: 'f1', name: 'Easy', ownerId: 'admin' })
    // Collapsed again.
    expect(screen.queryByRole('tab', { name: 'Featured' })).not.toBeInTheDocument()
  })

  it('expands a featured collection to reach a member game not featured itself', async () => {
    const member = def({ id: 'm1', name: 'Tricky', ownerId: 'admin' })
    server.use(...handlers({
      featured: [{ kind: 'collection', collection: col({ id: 'c1', name: 'Difficulty', ownerId: 'admin', visibility: 'curated' }) }],
      cols: [col({ id: 'c1', name: 'Difficulty', ownerId: 'admin', visibility: 'curated' })],
      members: { c1: [member] },
    }))
    const onSelect = renderPicker(null)
    await userEvent.click(screen.getByRole('button', { name: 'Choose a game' }))
    const toggle = await screen.findByRole('button', { name: /Difficulty/ })
    expect(toggle).toHaveAttribute('aria-expanded', 'false')

    await userEvent.click(toggle)
    await waitFor(() => expect(screen.getByRole('button', { name: 'Show leaderboard for Tricky' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Show leaderboard for Tricky' }))
    expect(onSelect).toHaveBeenCalledWith({ id: 'm1', name: 'Tricky', ownerId: 'admin' })
  })

  it('My Games shows the caller’s collections above their games', async () => {
    server.use(...handlers({
      defs: [def({ id: 'd1', name: 'My Game' })],
      cols: [col({ id: 'c1', name: 'My Set' })],
    }))
    renderPicker(null)
    await userEvent.click(screen.getByRole('button', { name: 'Choose a game' }))
    await userEvent.click(screen.getByRole('tab', { name: 'My Games' }))

    await waitFor(() => expect(screen.getByRole('button', { name: 'Show leaderboard for My Game' })).toBeInTheDocument())
    expect(screen.getByRole('heading', { name: 'Collections' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Games' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /My Set/ })).toBeInTheDocument()
  })

  it('Community lists other users’ public games and searches them', async () => {
    server.use(...handlers({
      defs: [
        def({ id: 'p1', name: 'Community Classic', ownerId: 'other', visibility: 'public' }),
        def({ id: 'p2', name: 'Another Public', ownerId: 'other', visibility: 'public' }),
      ],
    }))
    renderPicker(null)
    await userEvent.click(screen.getByRole('button', { name: 'Choose a game' }))
    await userEvent.click(screen.getByRole('tab', { name: 'Community' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Show leaderboard for Community Classic' })).toBeInTheDocument())

    // Server-side `q` for the public scope narrows the list. Both games start in
    // the list and the refetch is debounced, so settle on the state where the
    // non-match is gone AND the match remains (not just one of the two, which
    // also holds during the loading flash / before the refetch).
    await userEvent.type(screen.getByLabelText('Search games'), 'Another')
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'Show leaderboard for Community Classic' })).not.toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'Show leaderboard for Another Public' })).toBeInTheDocument()
    })
  })
})
