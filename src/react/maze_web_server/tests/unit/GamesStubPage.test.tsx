import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { GamesStubPage } from '../../src/pages/GamesStubPage'
import { resetMockGameDefinitions } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'
import type { GameDefinition, GameDefinitionRequest } from '../../src/types/api'

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: null, login: vi.fn(), logout: vi.fn() }),
  }
})

function renderGamesPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <GamesStubPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  resetMockGameDefinitions()
})

describe('GamesStubPage', () => {
  it('shows the empty state when the caller has no games', async () => {
    renderGamesPage()
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())
  })

  it('lists the definitions the caller may see', async () => {
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({
          definitions: [
            { id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' },
            { id: 'd2', ownerId: 'o1', name: 'Caverns', visibility: 'public', seed: 2, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' },
          ],
          limit: 20,
          offset: 0,
          hasMore: false,
        }),
      ),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    expect(screen.getByText('Caverns')).toBeInTheDocument()
  })

  it('surfaces a list failure', async () => {
    server.use(http.get('/api/v1/game-definitions', () => new HttpResponse('boom', { status: 500 })))
    renderGamesPage()
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('boom'))
  })

  it('New game → wizard → Finish creates the definition and refreshes the list', async () => {
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))

    // The default create handler stores the definition, so the post-create
    // refresh lists it and the wizard is gone.
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    expect(screen.queryByRole('button', { name: 'Finish' })).toBeNull()
  })

  it('Finish posts the request the editor built, with title / mode seeded from the name', async () => {
    let posted: GameDefinitionRequest | undefined
    server.use(
      http.post('/api/v1/game-definitions', async ({ request }) => {
        posted = await request.json() as GameDefinitionRequest
        return HttpResponse.json({ id: 'd9', ownerId: 'o1', name: posted.name, visibility: 'private', seed: 5, rotation: 'static', config: posted.config, createdAt: 'x', updatedAt: 'x' }, { status: 201 })
      }),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))

    await waitFor(() => expect(posted).toBeDefined())
    expect(posted?.name).toBe('Tower')
    expect(posted?.config).toMatchObject({ title: 'Tower', mode: 'Tower', rows: 8, cols: 8 })
  })

  it('Edit opens the tabs editor hydrated from the definition, and Save refreshes the list', async () => {
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))

    // Tabs mode: Save, no wizard navigation — and the name is hydrated.
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument())
    expect(screen.queryByRole('button', { name: 'Next' })).toBeNull()
    expect(screen.queryByRole('button', { name: 'Back' })).toBeNull()
    expect(screen.getByLabelText('Name')).toHaveValue('Tower')

    await userEvent.clear(screen.getByLabelText('Name'))
    await userEvent.type(screen.getByLabelText('Name'), 'Spire')
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByText('Spire')).toBeInTheDocument())
    expect(screen.queryByText('Tower')).toBeNull()
    expect(screen.queryByRole('button', { name: 'Save' })).toBeNull()
  })

  it('Edit → Layout → Reshuffle layout confirms and calls the reshuffle endpoint', async () => {
    let reshuffled = false
    server.use(
      http.post('/api/v1/game-definitions/:id/reshuffle', ({ params }) => {
        reshuffled = true
        return HttpResponse.json({
          id: params.id, ownerId: 'o1', name: 'Tower', visibility: 'private',
          seed: 5150, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x',
        })
      }),
    )

    renderGamesPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('tab', { name: 'Layout' }))
    await userEvent.click(screen.getByRole('button', { name: 'Reshuffle Layout' }))
    // A private draft has no scores → the mild wording.
    const dialog = screen.getByRole('dialog', { name: 'Reshuffle Layout' })
    expect(dialog).not.toHaveTextContent(/leaderboard/i)
    await userEvent.click(screen.getByRole('button', { name: 'Reshuffle' }))

    await waitFor(() => expect(reshuffled).toBe(true))
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Reshuffle Layout' })).toBeNull())
  })

  it('Save echoes visibility / rotation unchanged and sends the stored seed, not the effective one', async () => {
    const stored = { id: 'd1', ownerId: 'o1', name: 'Daily Tower', visibility: 'public', seed: 99, rotation: 'daily', config: { rows: 11, cols: 9, title: 'Ascend!', mode: 'Endless' }, createdAt: 'x', updatedAt: 'x' }
    let put: GameDefinitionRequest | undefined
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({ definitions: [stored], limit: 20, offset: 0, hasMore: false }),
      ),
      // The play-fetch splices a date-mixed seed into `config` for a Daily game,
      // while the record's own `seed` stays the stored one.
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...stored, config: { ...stored.config, seed: 777777 }, challengeKey: 'def:d1:2026-07-08', leaderboardTracked: true }),
      ),
      http.put('/api/v1/game-definitions/d1', async ({ request }) => {
        put = await request.json() as GameDefinitionRequest
        return HttpResponse.json({ ...stored, ...put })
      }),
    )

    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Daily Tower')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Daily Tower' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(put).toBeDefined())
    expect(put?.visibility).toBe('public')
    expect(put?.rotation).toBe('daily')
    // The record's seed, not the 777777 the play-fetch spliced in.
    expect(put?.config).toMatchObject({ seed: 99, rows: 11, cols: 9, title: 'Ascend!', mode: 'Endless' })
  })

  it('surfaces a failure to load the definition being edited', async () => {
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({
          definitions: [{ id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' }],
          limit: 20, offset: 0, hasMore: false,
        }),
      ),
      http.get('/api/v1/game-definitions/d1', () => new HttpResponse('gone', { status: 404 })),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('gone'))
    expect(screen.queryByRole('button', { name: 'Save' })).toBeNull()
  })

  it('keeps the editor open and reports the error when save fails', async () => {
    const stored = { id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' }
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({ definitions: [stored], limit: 20, offset: 0, hasMore: false }),
      ),
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...stored, challengeKey: 'def:d1', leaderboardTracked: false }),
      ),
      http.put('/api/v1/game-definitions/d1', () => new HttpResponse('A game with that name already exists.', { status: 409 })),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Edit Tower' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('A game with that name already exists.'))
    expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
  })

  it('keeps the wizard open and reports the error when create fails', async () => {
    server.use(http.post('/api/v1/game-definitions', () => new HttpResponse('A game with that name already exists.', { status: 409 })))
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('No games yet.')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'New game' }))
    await userEvent.type(screen.getByLabelText('Name'), 'Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Finish' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('A game with that name already exists.'))
    expect(screen.getByRole('button', { name: 'Finish' })).toBeInTheDocument()
  })

  it('Duplicate clones a definition into a fresh private draft named "Copy of X"', async () => {
    const stored: GameDefinition = { id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'public', seed: 7, rotation: 'static', config: { rows: 9, cols: 9, title: 'Tower', mode: 'Tower' }, createdAt: 'x', updatedAt: 'x' }
    const list: GameDefinition[] = [stored]
    let posted: GameDefinitionRequest | undefined
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({ definitions: list, limit: 20, offset: 0, hasMore: false }),
      ),
      http.post('/api/v1/game-definitions', async ({ request }) => {
        posted = await request.json() as GameDefinitionRequest
        const created: GameDefinition = { id: 'd2', ownerId: 'o1', name: posted.name, visibility: 'private', seed: 999, rotation: posted.rotation ?? 'static', config: posted.config, createdAt: 'x', updatedAt: 'x' }
        list.push(created)
        return HttpResponse.json(created, { status: 201 })
      }),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Duplicate Tower' }))
    // The prompt is seeded with "Copy of <source name>".
    expect(screen.getByLabelText('Name')).toHaveValue('Copy of Tower')
    await userEvent.click(screen.getByRole('button', { name: 'Duplicate' }))

    await waitFor(() => expect(posted).toBeDefined())
    expect(posted?.name).toBe('Copy of Tower')
    // A public source becomes a fresh private draft (no leaderboard).
    expect(posted?.visibility).toBe('private')
    // The source's stored config carries over verbatim.
    expect(posted?.config).toMatchObject({ rows: 9, cols: 9, title: 'Tower', mode: 'Tower' })
    // The list refreshes to include the copy.
    await waitFor(() => expect(screen.getByText('Copy of Tower')).toBeInTheDocument())
  })

  it('blocks a duplicate name that collides with an existing game', async () => {
    server.use(
      http.get('/api/v1/game-definitions', () =>
        HttpResponse.json({
          definitions: [
            { id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' },
            { id: 'd2', ownerId: 'o1', name: 'Copy of Tower', visibility: 'private', seed: 2, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x' },
          ],
          limit: 20, offset: 0, hasMore: false,
        }),
      ),
    )
    renderGamesPage()
    await waitFor(() => expect(screen.getByText('Tower')).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Duplicate Tower' }))
    // The default "Copy of Tower" already exists → the confirm is blocked client-side.
    await userEvent.click(screen.getByRole('button', { name: 'Duplicate' }))
    expect(screen.getByRole('alert')).toHaveTextContent('A game with that name already exists.')
  })
})
