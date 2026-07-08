import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { GamesStubPage } from '../../src/pages/GamesStubPage'
import { resetMockGameDefinitions } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'
import type { GameDefinitionRequest } from '../../src/types/api'

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
})
