import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { WorkshopGamesPage } from '../../src/pages/WorkshopGamesPage'
import { mockProfile, resetMockGameDefinitions, resetMockShares } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'
import type { GameDefinition, GameDefinitionRequest } from '../../src/types/api'

// The page filters to the caller's own games (ownerId === profile.id), so the
// mock profile id is the owner every test definition is stamped with.
const OWNER = mockProfile.id

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: OWNER, username: 'testuser', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

function def(overrides: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: OWNER, visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...overrides }
}

function listOf(...defs: GameDefinition[]) {
  return http.get('/api/v1/game-definitions', () =>
    HttpResponse.json({ definitions: defs, limit: 20, offset: 0, hasMore: false }))
}

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <WorkshopGamesPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  resetMockGameDefinitions()
  resetMockShares()
})
afterEach(() => {
  vi.unstubAllGlobals()
})

describe('WorkshopGamesPage', () => {
  it('shows the empty state when the caller has no games', async () => {
    renderPage()
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())
  })

  it('lists only the caller’s own games (not shared/public ones)', async () => {
    server.use(listOf(
      def({ id: 'd1', name: 'Mine' }),
      def({ id: 'd2', name: 'Someone Else', ownerId: 'other-user' }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Mine')).toBeInTheDocument())
    expect(screen.queryByText('Someone Else')).toBeNull()
  })

  it('excludes the caller’s own curated (featured) games — those belong to the Features area', async () => {
    server.use(listOf(
      def({ id: 'd1', name: 'Mine' }),
      def({ id: 'd2', name: 'Featured One', visibility: 'curated' }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Mine')).toBeInTheDocument())
    expect(screen.queryByText('Featured One')).toBeNull()
  })

  it('shows the visibility marker and tier in each game summary', async () => {
    server.use(listOf(
      def({ id: 'd1', name: 'Draft', visibility: 'private' }),
      def({ id: 'd2', name: 'Open', visibility: 'public' }),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Draft')).toBeInTheDocument())
    // Tier folded into the summary line.
    expect(screen.getByText(/Just me/)).toBeInTheDocument()
    expect(screen.getByText(/Everyone/)).toBeInTheDocument()
    // Marker src reflects each game's visibility.
    const draftRow = screen.getByText('Draft').closest('.game-list-item')!
    expect(draftRow.querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-private.svg')
    const openRow = screen.getByText('Open').closest('.game-list-item')!
    expect(openRow.querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-public.svg')
  })

  it('surfaces a list failure', async () => {
    server.use(http.get('/api/v1/game-definitions', () => new HttpResponse('boom', { status: 500 })))
    renderPage()
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('boom'))
  })

  it('New game → wizard → Finish creates the definition and refreshes the list', async () => {
    renderPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '+ New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))

    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    expect(screen.queryByRole('button', { name: 'Finish' })).toBeNull()
  })

  it('Edit opens the tabs editor hydrated from the definition, and Save refreshes the list', async () => {
    renderPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: '+ New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument())
    expect(screen.getByLabelText('Name')).toHaveValue('Tower')

    await userEvent.clear(screen.getByLabelText('Name'))
    await userEvent.type(screen.getByLabelText('Name'), 'Spire')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByText('Spire')).toBeInTheDocument())
    expect(screen.queryByText('Tower')).toBeNull()
  })

  it('Play launches the game host with the definition id', async () => {
    server.use(listOf(def({ id: 'd1', name: 'Tower' })))
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    // Stub location only after the list has loaded — replacing it earlier breaks
    // the relative-URL resolution the fetch needs.
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)
    await userEvent.click(screen.getByRole('button', { name: 'Play Tower' }))
    expect(locationStub.href).toBe('/game/?def=d1')
  })

  it('Leaderboard opens the board modal showing the game’s board', async () => {
    server.use(
      listOf(def({ id: 'd1', name: 'Tower', visibility: 'private' })),
      // The play-fetch reads the store; provide the single-definition response.
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...def({ id: 'd1', name: 'Tower', visibility: 'private' }), challengeKey: 'def:d1', leaderboardTracked: true })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Leaderboard for Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Leaderboard: Tower' })
    // Every game has a board (a private game's is owner-only); the modal shows it.
    await waitFor(() => expect(within(dialog).getByText('alice')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Close' }))
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Leaderboard: Tower' })).toBeNull())
  })

  it('Reshuffle confirms (mild wording for an unscored draft) and calls the endpoint', async () => {
    let reshuffled = false
    server.use(
      listOf(def({ id: 'd1', name: 'Tower', visibility: 'private' })),
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...def({ id: 'd1', name: 'Tower', visibility: 'private' }), challengeKey: 'def:d1', leaderboardTracked: false })),
      http.post('/api/v1/game-definitions/d1/reshuffle', () => {
        reshuffled = true
        return HttpResponse.json(def({ id: 'd1', name: 'Tower', seed: 5150 }))
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Reshuffle Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Reshuffle Layout' })
    // A private draft has no scores → the mild wording (no leaderboard warning).
    expect(dialog).not.toHaveTextContent(/leaderboard/i)
    await userEvent.click(screen.getByRole('button', { name: 'Reshuffle' }))

    await waitFor(() => expect(reshuffled).toBe(true))
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Reshuffle Layout' })).toBeNull())
  })

  it('Reshuffle on a scored, published game warns that the leaderboard is wiped', async () => {
    server.use(
      listOf(def({ id: 'd1', name: 'Tower', visibility: 'public' })),
      // Published ⇒ the play-fetch reports a tracked board; the board has a row.
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...def({ id: 'd1', name: 'Tower', visibility: 'public' }), challengeKey: 'def:d1', leaderboardTracked: true })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Reshuffle Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Reshuffle Layout' })
    expect(dialog).toHaveTextContent(/leaderboard/i)
  })

  it('Delete confirms and removes the game from the list', async () => {
    server.use(listOf(def({ id: 'd1', name: 'Tower' })))
    // The default DELETE handler needs the def in the store to 204; seed it.
    server.use(http.delete('/api/v1/game-definitions/d1', () => new HttpResponse(null, { status: 204 })))
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Delete Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Delete Game' })
    expect(dialog).toHaveTextContent(/permanently removes/i)

    // After deletion the list refetch returns the original (still-populated) mock
    // list, so override it to reflect the removal.
    server.use(listOf())
    await userEvent.click(screen.getByRole('button', { name: 'Delete' }))
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())
  })

  it('Duplicate clones a game into a fresh private draft named "Copy of X"', async () => {
    const list: GameDefinition[] = [def({ id: 'd1', name: 'Tower', visibility: 'public', config: { rows: 9, cols: 9 } })]
    let posted: GameDefinitionRequest | undefined
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({ definitions: list, limit: 20, offset: 0, hasMore: false })),
      http.post('/api/v1/game-definitions', async ({ request }) => {
        posted = await request.json() as GameDefinitionRequest
        const created = def({ id: 'd2', name: posted.name, config: posted.config })
        list.push(created)
        return HttpResponse.json(created, { status: 201 })
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Duplicate Tower' }))
    expect(screen.getByLabelText('Name')).toHaveValue('Copy of Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Duplicate' }))

    await waitFor(() => expect(posted).toBeDefined())
    expect(posted?.name).toBe('Copy of Tower')
    expect(posted?.visibility).toBe('private')
    expect(posted?.config).toMatchObject({ rows: 9, cols: 9 })
    await waitFor(() => expect(screen.getByText('Copy of Tower')).toBeInTheDocument())
  })

  it('shows the global busy cursor while loading a game to edit', async () => {
    let release!: () => void
    const gate = new Promise<void>(r => { release = r })
    server.use(
      listOf(def({ id: 'd1', name: 'Tower' })),
      http.get('/api/v1/game-definitions/d1', async () => {
        await gate
        return HttpResponse.json({ ...def({ id: 'd1', name: 'Tower' }), challengeKey: 'def:d1', leaderboardTracked: false })
      }),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))
    // The load is in flight (gate held) → the busy cursor is on.
    await waitFor(() => expect(document.body).toHaveClass('is-busy'))
    release()
    // Once the editor opens the busy cursor clears.
    await waitFor(() => expect(document.body).not.toHaveClass('is-busy'))
  })

  it('refreshes the access badge when a share is added (private → shared)', async () => {
    // Use the default (mock-store-backed) create/list/share handlers so the grant
    // handler mirrors B14 by mutating the same definition the list reads back.
    renderPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: '+ New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    // A fresh game is private.
    expect(screen.getByText(/Just me/)).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Share Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Share: Tower' })
    await userEvent.type(within(dialog).getByLabelText('Add user'), 'bob')
    await userEvent.click(await within(dialog).findByRole('button', { name: 'Add bob' }))
    await waitFor(() => expect(within(dialog).getByRole('button', { name: 'Remove bob' })).toBeInTheDocument())

    await userEvent.click(within(dialog).getByRole('button', { name: 'Close' }))
    // The row's visibility reloaded on the grant, so the summary now reads the
    // shared tier and the marker updates.
    await waitFor(() => expect(screen.getByText(/Specific people/)).toBeInTheDocument())
    expect(screen.queryByText(/Just me/)).toBeNull()
    const row = screen.getByText('Tower').closest('.game-list-item')!
    expect(row.querySelector('.game-thumb-marker')).toHaveAttribute('src', '/images/workshop/marker-shared.svg')
  })

  it('Share opens the manage-sharing modal for the row', async () => {
    server.use(listOf(def({ id: 'd1', name: 'Tower', visibility: 'shared' })))
    renderPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Share Tower' }))
    const dialog = await screen.findByRole('dialog', { name: 'Share: Tower' })
    expect(dialog).toHaveTextContent('Tower')
    expect(screen.getByLabelText('Add user')).toBeInTheDocument()
  })
})
