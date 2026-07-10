import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { WorkshopHubPage } from '../../src/pages/WorkshopHubPage'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

let isAdmin = false
vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: true,
      profile: { id: 'me', username: 'me', is_admin: isAdmin },
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

function renderHub() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <WorkshopHubPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  isAdmin = false
})

describe('WorkshopHubPage', () => {
  it('renders the Games and Game Collections tiles', () => {
    renderHub()
    expect(screen.getByRole('heading', { name: /^games$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^game collections$/i })).toBeInTheDocument()
  })

  it('hides the Features tile for a non-admin', () => {
    renderHub()
    expect(screen.queryByRole('heading', { name: /features/i })).not.toBeInTheDocument()
  })

  it('shows the Features tile for an admin', () => {
    isAdmin = true
    renderHub()
    expect(screen.getByRole('heading', { name: /features \[admin\]/i })).toBeInTheDocument()
  })

  it('clicking Games navigates to /workshop/games', async () => {
    renderHub()
    await userEvent.click(screen.getByRole('button', { name: /create, edit, publish/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop/games')
  })

  it('clicking Game Collections navigates to /workshop/game-collections', async () => {
    renderHub()
    await userEvent.click(screen.getByRole('button', { name: /group your games/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop/game-collections')
  })

  it('clicking Features navigates to /workshop/features', async () => {
    isAdmin = true
    renderHub()
    await userEvent.click(screen.getByRole('button', { name: /manage the featured games/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop/features')
  })
})
