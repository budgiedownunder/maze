import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { WorkshopCollectionsPage } from '../../src/pages/WorkshopCollectionsPage'
import { mockProfile, resetMockGameCollections, resetMockShares } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'
import type { GameCollection, GameDefinition } from '../../src/types/api'

// The page filters to the caller's own collections (ownerId === profile.id).
const OWNER = mockProfile.id

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: OWNER, username: 'testuser', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

function col(overrides: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: OWNER, visibility: 'private', items: [], createdAt: 'x', updatedAt: 'x', ...overrides }
}

function listOf(...cols: GameCollection[]) {
  return http.get('/api/v1/game-collections', () =>
    HttpResponse.json({ collections: cols, limit: 20, offset: 0, hasMore: false }))
}

function def(overrides: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: OWNER, visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...overrides }
}

// Seeds a collection `c1` whose members are `members`, an Add-picker drawn from
// `ownGames`, and captures the reconcile PUT body. Returns a getter for it.
function seedForEdit(members: GameDefinition[], ownGames: GameDefinition[]) {
  const collection = col({ id: 'c1', name: 'Campaign', items: members.map((d, i) => ({ definitionId: d.id, sortOrder: i })) })
  const captured: { body?: { definitionIds: string[] } } = {}
  server.use(
    listOf(collection),
    http.get('/api/v1/game-collections/c1', () => HttpResponse.json({ ...collection, definitions: members })),
    http.get('/api/v1/game-definitions', () => HttpResponse.json({ definitions: ownGames, limit: 20, offset: 0, hasMore: false })),
    http.put('/api/v1/game-collections/c1/items', async ({ request }) => {
      captured.body = await request.json() as { definitionIds: string[] }
      const items = captured.body.definitionIds.map((definitionId, sortOrder) => ({ definitionId, sortOrder }))
      return HttpResponse.json({ ...collection, items })
    }),
  )
  return captured
}

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <WorkshopCollectionsPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  resetMockGameCollections()
  resetMockShares()
})

describe('WorkshopCollectionsPage', () => {
  it('shows the empty state when the caller has no collections', async () => {
    renderPage()
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())
  })

  it('lists only the caller’s own collections (not shared/public ones)', async () => {
    server.use(listOf(
      col({ id: 'c1', name: 'Mine' }),
      col({ id: 'c2', name: 'Someone Else', ownerId: 'other-user' }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Mine')).toBeInTheDocument())
    expect(screen.queryByText('Someone Else')).toBeNull()
  })

  it('summarises each collection with its game count and access tier', async () => {
    server.use(listOf(
      col({ id: 'c1', name: 'Solo', items: [{ definitionId: 'd1', sortOrder: 0 }] }),
      col({ id: 'c2', name: 'Trio', visibility: 'public', items: [
        { definitionId: 'd1', sortOrder: 0 },
        { definitionId: 'd2', sortOrder: 1 },
        { definitionId: 'd3', sortOrder: 2 },
      ] }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('1 game · Just me')).toBeInTheDocument())
    expect(screen.getByText('3 games · Everyone')).toBeInTheDocument()
  })

  it('shows the base thumbnail and the visibility marker per row', async () => {
    server.use(listOf(
      col({ id: 'c1', name: 'Private one' }),
      col({ id: 'c2', name: 'Public one', visibility: 'public' }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Private one')).toBeInTheDocument())

    const privateRow = screen.getByText('Private one').closest('.game-list-item')!
    expect(privateRow.querySelector('.game-thumb-base')).toHaveAttribute('src', '/images/workshop/workshop-game-collection.svg')
    expect(privateRow.querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-private.svg')

    const publicRow = screen.getByText('Public one').closest('.game-list-item')!
    expect(publicRow.querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-public.svg')
  })

  it('creates a collection and shows it in the list', async () => {
    // The default (mock-store-backed) create + list handlers persist the new
    // collection so the post-create refresh reads it back.
    renderPage()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '+ New collection' }))
    const dialog = await screen.findByRole('dialog', { name: 'New Collection' })
    await userEvent.type(screen.getByLabelText('Name'), 'Campaign')
    await userEvent.type(screen.getByLabelText('Description (optional)'), 'My best levels')
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'New Collection' })).toBeNull())
    expect(screen.getByText('Campaign')).toBeInTheDocument()
    expect(dialog).not.toBeInTheDocument()
  })

  it('Edit renames a collection and refreshes the list', async () => {
    renderPage()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: '+ New collection' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Campaign')
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    expect(within(dialog).getByLabelText('Name')).toHaveValue('Campaign')
    await userEvent.clear(within(dialog).getByLabelText('Name'))
    await userEvent.type(within(dialog).getByLabelText('Name'), 'Season 1')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByText('Season 1')).toBeInTheDocument())
    expect(screen.queryByText('Campaign')).toBeNull()
  })

  it('Delete confirms and removes the collection from the list', async () => {
    renderPage()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: '+ New collection' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Doomed')
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))
    await waitFor(() => expect(screen.getByText('Doomed')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Delete Doomed' }))
    const dialog = await screen.findByRole('dialog', { name: 'Delete Collection' })
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }))

    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())
    expect(screen.queryByText('Doomed')).toBeNull()
  })

  it('Edit lists the member games and adding one reconciles on Save', async () => {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    const g3 = def({ id: 'g3', name: 'Gamma' })
    const captured = seedForEdit([g1, g2], [g1, g2, g3])
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    await waitFor(() => expect(within(dialog).getByText('Alpha')).toBeInTheDocument())
    expect(within(dialog).getByText('Beta')).toBeInTheDocument()

    // Save is idle until something changes.
    expect(within(dialog).getByRole('button', { name: 'Save' })).toBeDisabled()

    // The picker offers the owner's other game; add it.
    await userEvent.type(within(dialog).getByLabelText('Add game'), 'Gam')
    await userEvent.click(await within(dialog).findByRole('button', { name: 'Add Gamma' }))
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Edit Collection' })).toBeNull())
    expect(captured.body).toEqual({ definitionIds: ['g1', 'g2', 'g3'] })
  })

  it('Edit reorders and removes members, sending the new order on Save', async () => {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    const g3 = def({ id: 'g3', name: 'Gamma' })
    const captured = seedForEdit([g1, g2, g3], [g1, g2, g3])
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    await waitFor(() => expect(within(dialog).getByText('Alpha')).toBeInTheDocument())

    // Alpha down (→ Beta, Alpha, Gamma), then remove Gamma (→ Beta, Alpha).
    await userEvent.click(within(dialog).getByRole('button', { name: 'Move Alpha down' }))
    await userEvent.click(within(dialog).getByRole('button', { name: 'Remove Gamma' }))
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Edit Collection' })).toBeNull())
    expect(captured.body).toEqual({ definitionIds: ['g2', 'g1'] })
  })

  it('adds a game after the selected row and selects the new one', async () => {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    const g3 = def({ id: 'g3', name: 'Gamma' })
    const captured = seedForEdit([g1, g2], [g1, g2, g3])
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    await waitFor(() => expect(within(dialog).getByText('Alpha')).toBeInTheDocument())

    // Select the first row (Alpha), then add Gamma — it lands right after Alpha.
    await userEvent.click(within(dialog).getByText('Alpha'))
    await userEvent.type(within(dialog).getByLabelText('Add game'), 'Gam')
    await userEvent.click(await within(dialog).findByRole('button', { name: 'Add Gamma' }))

    // Gamma becomes the selected row.
    expect(within(dialog).getByText('Gamma').closest('.collection-member')).toHaveClass('selected')

    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Edit Collection' })).toBeNull())
    expect(captured.body).toEqual({ definitionIds: ['g1', 'g3', 'g2'] })
  })

  it('highlights a clicked source (available games) row under its heading', async () => {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    seedForEdit([g1], [g1, g2])
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    await waitFor(() => expect(within(dialog).getByText('Alpha')).toBeInTheDocument())

    expect(within(dialog).getByText('Available Games')).toBeInTheDocument()
    // Beta is the only available game; click its row to highlight it (no add).
    await userEvent.click(within(dialog).getByText('Beta'))
    expect(within(dialog).getByText('Beta').closest('.collection-picker-item')).toHaveClass('selected')
    // Clicking the row did not add it — Beta is still an available game, not a member.
    expect(within(dialog).getByRole('button', { name: 'Add Beta' })).toBeInTheDocument()
  })

  it('selecting a member row keeps the highlight on it after a move', async () => {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    seedForEdit([g1, g2], [g1, g2])
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Edit Collection' })
    await waitFor(() => expect(within(dialog).getByText('Alpha')).toBeInTheDocument())

    // Click Alpha's row to select it.
    await userEvent.click(within(dialog).getByText('Alpha'))
    expect(within(dialog).getByText('Alpha').closest('.collection-member')).toHaveClass('selected')

    // Move it down; the highlight stays on Alpha, not on the row it swapped with.
    await userEvent.click(within(dialog).getByRole('button', { name: 'Move Alpha down' }))
    expect(within(dialog).getByText('Alpha').closest('.collection-member')).toHaveClass('selected')
    expect(within(dialog).getByText('Beta').closest('.collection-member')).not.toHaveClass('selected')
  })

  it('Access opens the access modal for a collection', async () => {
    server.use(listOf(col({ id: 'c1', name: 'Campaign', visibility: 'shared' })))
    renderPage()
    await waitFor(() => expect(screen.getByText('Campaign')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Access for Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Access: Campaign' })
    expect(dialog).toHaveTextContent('Campaign')
    // A shared collection opens with the people-picker shown.
    expect(within(dialog).getByLabelText('Add user')).toBeInTheDocument()
  })

  it('setting a collection to Everyone via the access modal updates its summary', async () => {
    // The default (store-backed) handlers persist the share list + the
    // visibility PUT so the reloaded row reflects the new tier.
    renderPage()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: '+ New collection' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Campaign')
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))
    await waitFor(() => expect(screen.getByText('0 games · Just me')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Access for Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Access: Campaign' })
    await userEvent.click(within(dialog).getByRole('radio', { name: /Everyone/ }))
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByText('0 games · Everyone')).toBeInTheDocument())
    expect(screen.queryByText('0 games · Just me')).toBeNull()
  })

  it('blocks an empty name and surfaces a create failure', async () => {
    server.use(
      http.post('/api/v1/game-collections', () => new HttpResponse('nope', { status: 500 })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('No collections yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '+ New collection' }))
    // Empty name is rejected client-side before any request.
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))
    expect(await screen.findByText('Name cannot be empty.')).toBeInTheDocument()

    await userEvent.type(screen.getByLabelText('Name'), 'Campaign')
    await userEvent.click(screen.getByRole('button', { name: 'Create' }))
    // The server error keeps the modal open with the message shown.
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('nope'))
    expect(screen.getByRole('dialog', { name: 'New Collection' })).toBeInTheDocument()
  })
})
