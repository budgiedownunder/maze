import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockMazeAlpha, resetMockMazes } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { MazePage } from '../../src/pages/MazePage'

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

beforeEach(() => {
  vi.clearAllMocks()
  resetMockMazes()
})

function renderMazePage(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <ThemeProvider>
        <Routes>
          <Route path="/mazes/new" element={<MazePage />} />
          <Route path="/mazes/:id" element={<MazePage />} />
        </Routes>
      </ThemeProvider>
    </MemoryRouter>,
  )
}

describe('MazePage', () => {
  it('shows loading indicator while fetching an existing maze', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
  })

  it('renders the maze name in the header after loading', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())
  })

  it('renders the maze grid after loading', async () => {
    renderMazePage(`/mazes/${mockMazeAlpha.id}`)
    // Wait for loading to finish
    await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
    // Grid container should be present
    expect(screen.getByLabelText('Maze grid')).toBeInTheDocument()
    // Start cell image should be visible
    expect(screen.getByAltText('Start')).toBeInTheDocument()
    // Finish cell image should be visible
    expect(screen.getByAltText('Finish')).toBeInTheDocument()
  })

  it('shows not-found message for a 404 response', async () => {
    server.use(
      http.get('/api/v1/mazes/:id', () => new HttpResponse(null, { status: 404 })),
    )
    renderMazePage('/mazes/nonexistent-id')
    await waitFor(() => expect(screen.getByText(/not found/i)).toBeInTheDocument())
  })

  it('shows error message when API fails', async () => {
    server.use(
      http.get('/api/v1/mazes/:id', () => new HttpResponse('Server exploded', { status: 500 })),
    )
    renderMazePage('/mazes/some-id')
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(screen.getByRole('alert')).toHaveTextContent('Server exploded')
  })

  it('shows (unsaved) in the header for the /mazes/new route', () => {
    renderMazePage('/mazes/new')
    expect(screen.getByText('(unsaved)')).toBeInTheDocument()
  })

  it('renders a blank grid for /mazes/new without making an API call', () => {
    const getMazeSpy = vi.fn()
    server.use(http.get('/api/v1/mazes/:id', () => { getMazeSpy(); return new HttpResponse(null, { status: 404 }) }))
    renderMazePage('/mazes/new')
    // Grid should appear immediately (no loading state)
    expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Maze grid')).toBeInTheDocument()
    // No API call should have been made
    expect(getMazeSpy).not.toHaveBeenCalled()
  })
})
