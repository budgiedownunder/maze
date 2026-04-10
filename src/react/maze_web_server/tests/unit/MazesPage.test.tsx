import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { MazesPage } from '../../src/pages/MazesPage'
import { mockMazeAlpha, mockMazeBeta, resetMockMazes } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

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

function renderMazesPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <MazesPage />
      </ThemeProvider>
    </MemoryRouter>
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  resetMockMazes()
})

describe('MazesPage', () => {
  it('shows loading indicator while fetching', async () => {
    renderMazesPage()
    expect(screen.getByLabelText('Loading')).toBeInTheDocument()
    await waitFor(() => expect(screen.queryByLabelText('Loading')).not.toBeInTheDocument())
  })

  it('renders maze names after loading', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())
    expect(screen.getByText(mockMazeBeta.name)).toBeInTheDocument()
  })

  it('shows dimensions subtitle for each maze', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText('3 rows × 3 columns')).toBeInTheDocument())
    expect(screen.getByText('5 rows × 5 columns')).toBeInTheDocument()
  })

  it('uses singular row/column when dimension is 1', async () => {
    const singularMaze = { id: 'maze-0003', name: 'Tiny', definition: { grid: [['S']] } }
    server.use(
      http.get('/api/v1/mazes', () => HttpResponse.json(
        [singularMaze].map(m => ({ id: m.id, name: m.name, definition: JSON.stringify(m) }))
      ))
    )
    renderMazesPage()
    await waitFor(() => expect(screen.getByText('1 row × 1 column')).toBeInTheDocument())
  })

  it('shows empty state when no mazes returned', async () => {
    server.use(
      http.get('/api/v1/mazes', () => HttpResponse.json([]))
    )
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(/no mazes yet/i)).toBeInTheDocument())
  })

  it('shows error message when API fails', async () => {
    server.use(
      http.get('/api/v1/mazes', () => new HttpResponse('Server error', { status: 500 }))
    )
    renderMazesPage()
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
  })

  it('refresh button reloads the list', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    const extraMaze = { id: 'maze-0003', name: 'Gamma', definition: { grid: [['S', 'F']] } }
    server.use(
      http.get('/api/v1/mazes', () => HttpResponse.json(
        [mockMazeAlpha, mockMazeBeta, extraMaze].map(m => ({
          id: m.id, name: m.name, definition: JSON.stringify(m),
        }))
      ))
    )

    await userEvent.click(screen.getByRole('button', { name: /refresh/i }))
    await waitFor(() => expect(screen.getByText('Gamma')).toBeInTheDocument())
  })

  it('clicking a maze item navigates to /mazes/:id', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByText(mockMazeAlpha.name))
    expect(mockNavigate).toHaveBeenCalledWith(`/mazes/${mockMazeAlpha.id}`)
  })

  it('each maze item has a Delete button', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    // aria-labels are "Delete Alpha" / "Delete Beta" — start with "Delete "
    // li[role=button] accessible names start with the maze name, so /^Delete / excludes them
    const deleteButtons = screen.getAllByRole('button', { name: /^Delete /i })
    expect(deleteButtons).toHaveLength(2)
  })

  it('clicking Delete opens a confirmation modal with the maze name', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Delete ${mockMazeAlpha.name}` }))
    const dialog = screen.getByRole('dialog', { name: 'Delete Maze' })
    expect(dialog).toBeInTheDocument()
    expect(within(dialog).getByText(new RegExp(mockMazeAlpha.name))).toBeInTheDocument()
  })

  it('Cancel closes the modal without deleting', async () => {
    const deleteSpy = vi.fn()
    server.use(http.delete('/api/v1/mazes/:id', () => { deleteSpy(); return new HttpResponse(null, { status: 200 }) }))

    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Delete ${mockMazeAlpha.name}` }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))

    expect(screen.queryByRole('dialog', { name: 'Delete Maze' })).not.toBeInTheDocument()
    expect(deleteSpy).not.toHaveBeenCalled()
    expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument()
  })

  it('confirming Delete removes the maze from the list', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Delete ${mockMazeAlpha.name}` }))
    await userEvent.click(screen.getByRole('button', { name: /^delete$/i }))

    await waitFor(() => expect(screen.queryByText(mockMazeAlpha.name)).not.toBeInTheDocument())
    expect(screen.queryByRole('dialog', { name: 'Delete Maze' })).not.toBeInTheDocument()
    expect(screen.getByText(mockMazeBeta.name)).toBeInTheDocument()
  })

  it('each maze item has a Rename button', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    const renameButtons = screen.getAllByRole('button', { name: /^Rename /i })
    expect(renameButtons).toHaveLength(2)
  })

  it('clicking Rename opens the prompt modal pre-filled with the maze name', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    const dialog = screen.getByRole('dialog', { name: 'Rename Maze' })
    expect(dialog).toBeInTheDocument()
    expect(within(dialog).getByRole('textbox')).toHaveValue(mockMazeAlpha.name)
  })

  it('Rename Cancel closes the modal without saving', async () => {
    const putSpy = vi.fn()
    server.use(http.put('/api/v1/mazes/:id', () => { putSpy(); return new HttpResponse(null, { status: 200 }) }))

    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))

    expect(screen.queryByRole('dialog', { name: 'Rename Maze' })).not.toBeInTheDocument()
    expect(putSpy).not.toHaveBeenCalled()
  })

  it('Rename shows validation error for empty name', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    const input = screen.getByRole('textbox')
    await userEvent.clear(input)
    await userEvent.click(screen.getByRole('button', { name: /^rename$/i }))

    expect(screen.getByRole('alert')).toHaveTextContent(/empty/i)
    expect(screen.getByRole('dialog', { name: 'Rename Maze' })).toBeInTheDocument()
  })

  it('Rename shows validation error for duplicate name', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    const input = screen.getByRole('textbox')
    await userEvent.clear(input)
    await userEvent.type(input, mockMazeBeta.name)
    await userEvent.click(screen.getByRole('button', { name: /^rename$/i }))

    expect(screen.getByRole('alert')).toHaveTextContent(/already exists/i)
    expect(screen.getByRole('dialog', { name: 'Rename Maze' })).toBeInTheDocument()
  })

  it('confirming Rename updates the list', async () => {
    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    const input = screen.getByRole('textbox')
    await userEvent.clear(input)
    await userEvent.type(input, 'AlphaRenamed')
    await userEvent.click(screen.getByRole('button', { name: /^rename$/i }))

    await waitFor(() => expect(screen.getByText('AlphaRenamed')).toBeInTheDocument())
    expect(screen.queryByRole('dialog', { name: 'Rename Maze' })).not.toBeInTheDocument()
  })

  it('shows server error inside the Rename modal when update fails', async () => {
    server.use(
      http.put('/api/v1/mazes/:id', () => new HttpResponse('Name already taken', { status: 409 }))
    )

    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Rename ${mockMazeAlpha.name}` }))
    const input = screen.getByRole('textbox')
    await userEvent.clear(input)
    await userEvent.type(input, 'NewName')
    await userEvent.click(screen.getByRole('button', { name: /^rename$/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(screen.getByRole('alert')).toHaveTextContent('Name already taken')
    expect(screen.getByRole('dialog', { name: 'Rename Maze' })).toBeInTheDocument()
  })

  it('shows server error message inside the modal when delete fails', async () => {
    server.use(
      http.delete('/api/v1/mazes/:id', () => new HttpResponse('Maze is locked', { status: 409 }))
    )

    renderMazesPage()
    await waitFor(() => expect(screen.getByText(mockMazeAlpha.name)).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: `Delete ${mockMazeAlpha.name}` }))
    await userEvent.click(screen.getByRole('button', { name: /^delete$/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(screen.getByRole('alert')).toHaveTextContent('Maze is locked')
    expect(screen.getByRole('dialog', { name: 'Delete Maze' })).toBeInTheDocument()
  })
})
