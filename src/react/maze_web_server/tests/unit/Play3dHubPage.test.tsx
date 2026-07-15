import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dHubPage } from '../../src/pages/Play3dHubPage'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: 'me', username: 'me', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

function renderHub() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <Play3dHubPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => mockNavigate.mockReset())

describe('Play3dHubPage', () => {
  it('renders all four browse-scope tiles', () => {
    renderHub()
    expect(screen.getByRole('heading', { name: /^featured$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^my games$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^shared with me$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^community$/i })).toBeInTheDocument()
  })

  it('navigates to the Featured page', async () => {
    renderHub()
    await userEvent.click(screen.getByRole('button', { name: /featured/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d/featured')
  })

  it('navigates to each of the three coming-soon scopes', async () => {
    renderHub()
    await userEvent.click(screen.getByRole('button', { name: /my games/i }))
    await userEvent.click(screen.getByRole('button', { name: /shared with me/i }))
    await userEvent.click(screen.getByRole('button', { name: /community/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d/my-games')
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d/shared')
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d/community')
  })
})
