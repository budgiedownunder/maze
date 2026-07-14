import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { WorkshopFeaturesPage } from '../../src/pages/WorkshopFeaturesPage'
import { server } from '../../src/mocks/server'
import { resetMockShares } from '../../src/mocks/handlers'
import type { FeaturedGameItem, GameCollection, GameDefinition } from '../../src/types/api'

const OWNER = 'admin-me'

let isAdmin = true
vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: OWNER, username: 'admin', is_admin: isAdmin }, login: vi.fn(), logout: vi.fn() }),
  }
})

function def(overrides: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: OWNER, visibility: 'curated', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...overrides }
}

function col(overrides: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: OWNER, visibility: 'curated', items: [], createdAt: 'x', updatedAt: 'x', ...overrides }
}

function defItem(d: GameDefinition): FeaturedGameItem { return { kind: 'definition', definition: d } }
function colItem(c: GameCollection): FeaturedGameItem { return { kind: 'collection', collection: c } }

// A stateful mock of the featured endpoints: GET pages the current list; PUT
// /order reorders it (matching entries by kind+id) and returns the new order.
function seedFeatured(initial: FeaturedGameItem[]) {
  let items = [...initial]
  const idOf = (it: FeaturedGameItem) => (it.kind === 'definition' ? it.definition!.id : it.collection!.id)
  const captured: { entries?: { kind: string; id: string }[] } = {}
  server.use(
    http.get('/api/v1/featured-game-items', ({ request }) => {
      const url = new URL(request.url)
      const limit = Number(url.searchParams.get('limit') ?? '20')
      const offset = Number(url.searchParams.get('offset') ?? '0')
      return HttpResponse.json({ items: items.slice(offset, offset + limit), limit, offset, hasMore: offset + limit < items.length })
    }),
    http.put('/api/v1/featured-game-items/order', async ({ request }) => {
      captured.entries = (await request.json() as { entries: { kind: string; id: string }[] }).entries
      const byKey = new Map(items.map(it => [`${it.kind}:${idOf(it)}`, it]))
      items = captured.entries.map(e => byKey.get(`${e.kind}:${e.id}`)).filter((x): x is FeaturedGameItem => !!x)
      return HttpResponse.json({ items, limit: items.length, offset: 0, hasMore: false })
    }),
  )
  return captured
}

function renderPage() {
  return render(
    <MemoryRouter initialEntries={['/workshop/features']}>
      <ThemeProvider>
        <Routes>
          <Route path="/workshop/features" element={<WorkshopFeaturesPage />} />
          <Route path="/workshop" element={<div>Workshop hub</div>} />
        </Routes>
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  resetMockShares()
  isAdmin = true
})

describe('WorkshopFeaturesPage', () => {
  it('bounces a non-admin back to the hub', async () => {
    isAdmin = false
    renderPage()
    await waitFor(() => expect(screen.getByText('Workshop hub')).toBeInTheDocument())
    expect(screen.queryByText('Manage Features')).not.toBeInTheDocument()
  })

  it('shows the empty state for an admin with no featured items', async () => {
    seedFeatured([])
    renderPage()
    await waitFor(() => expect(screen.getByText(/No featured items yet/)).toBeInTheDocument())
  })

  it('lists games and collections in order, with the Featured marker and per-kind actions', async () => {
    seedFeatured([
      defItem(def({ id: 'd1', name: 'Alpha Game', config: { levels: { count: 3 } } })),
      colItem(col({ id: 'c1', name: 'Beta Set', items: [{ definitionId: 'd1', sortOrder: 0 }] })),
    ])
    renderPage()
    await waitFor(() => expect(screen.getByText('Alpha Game')).toBeInTheDocument())

    // Order: the game first, then the collection.
    const items = screen.getAllByRole('listitem')
    expect(within(items[0]).getByText('Alpha Game')).toBeInTheDocument()
    expect(within(items[1]).getByText('Beta Set')).toBeInTheDocument()

    // Summaries reflect kind.
    expect(within(items[0]).getByText('Game · 3 levels · Static')).toBeInTheDocument()
    expect(within(items[1]).getByText('Collection · 1 game')).toBeInTheDocument()

    // The relocated curated-marker assertion: a featured row shows the star.
    expect(items[0].querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-curated.svg')

    // A game row has Play + Leaderboard; a collection row does not.
    expect(within(items[0]).getByRole('button', { name: 'Play Alpha Game' })).toBeInTheDocument()
    expect(within(items[0]).getByRole('button', { name: 'Leaderboard for Alpha Game' })).toBeInTheDocument()
    expect(within(items[1]).queryByRole('button', { name: /^Play / })).toBeNull()
    // Both have Edit + Access.
    expect(within(items[1]).getByRole('button', { name: 'Edit Beta Set' })).toBeInTheDocument()
    expect(within(items[1]).getByRole('button', { name: 'Access for Beta Set' })).toBeInTheDocument()
  })

  it('disables Up on the first row and Down on the last', async () => {
    seedFeatured([
      defItem(def({ id: 'd1', name: 'First' })),
      defItem(def({ id: 'd2', name: 'Last' })),
    ])
    renderPage()
    await waitFor(() => expect(screen.getByText('First')).toBeInTheDocument())
    const items = screen.getAllByRole('listitem')
    expect(within(items[0]).getByRole('button', { name: 'Move First up' })).toBeDisabled()
    expect(within(items[0]).getByRole('button', { name: 'Move First down' })).toBeEnabled()
    expect(within(items[1]).getByRole('button', { name: 'Move Last up' })).toBeEnabled()
    expect(within(items[1]).getByRole('button', { name: 'Move Last down' })).toBeDisabled()
  })

  it('reorders via the Down button, submitting the whole order', async () => {
    const captured = seedFeatured([
      defItem(def({ id: 'd1', name: 'First' })),
      defItem(def({ id: 'd2', name: 'Second' })),
      defItem(def({ id: 'd3', name: 'Third' })),
    ])
    renderPage()
    await waitFor(() => expect(screen.getByText('First')).toBeInTheDocument())

    // Move the first item down.
    await userEvent.click(screen.getByRole('button', { name: 'Move First down' }))

    // The reorder submits the complete order with the swap applied.
    await waitFor(() => expect(captured.entries).toEqual([
      { kind: 'definition', id: 'd2' },
      { kind: 'definition', id: 'd1' },
      { kind: 'definition', id: 'd3' },
    ]))
    // And the list re-renders in the new order.
    await waitFor(() => {
      const names = screen.getAllByRole('listitem').map(li => within(li).getByText(/First|Second|Third/).textContent)
      expect(names).toEqual(['Second', 'First', 'Third'])
    })
  })

  it('opens the Access modal for a featured item', async () => {
    seedFeatured([defItem(def({ id: 'd1', name: 'Alpha Game' }))])
    renderPage()
    await waitFor(() => expect(screen.getByText('Alpha Game')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Access for Alpha Game' }))
    expect(await screen.findByRole('dialog', { name: 'Access: Alpha Game' })).toBeInTheDocument()
  })

  it('opens the game editor for a featured game (admin-override edit)', async () => {
    seedFeatured([defItem(def({ id: 'd1', name: 'Alpha Game' }))])
    // Edit loads the definition (play-fetch) and probes the board for score count.
    server.use(
      http.get('/api/v1/game-definitions/d1', () => HttpResponse.json({
        ...def({ id: 'd1', name: 'Alpha Game' }), config: {}, challengeKey: 'def:d1', leaderboardTracked: true,
      })),
      http.get('/api/v1/scores', () => HttpResponse.json({ scores: [], limit: 1, offset: 0, hasMore: false })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Alpha Game')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Alpha Game' }))
    expect(await screen.findByRole('dialog', { name: 'Edit Game' })).toBeInTheDocument()
  })
})
