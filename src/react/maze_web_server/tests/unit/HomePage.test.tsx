import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { HomePage } from '../../src/pages/HomePage'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: true,
      profile: null,
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

function renderHomePage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <HomePage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('HomePage', () => {
  it('renders the tile titles', () => {
    renderHomePage()
    expect(screen.getByRole('heading', { name: /^3d games$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^3d game workshop$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^mazes$/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /^leaderboards$/i })).toBeInTheDocument()
  })

  it('clicking 3D Game Workshop navigates to /workshop', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /3d game workshop/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop')
  })

  it('clicking 3D Games navigates to the Play-3D hub', async () => {
    renderHomePage()
    // Targeted by the tile's unique description — the Workshop tile's copy also
    // mentions "3D games".
    await userEvent.click(screen.getByRole('button', { name: /browse and play 3d games/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/play-3d')
  })

  it('clicking Mazes navigates to /mazes', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /mazes/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/mazes')
  })

  it('clicking Leaderboards navigates to /leaderboards', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /your times and how you rank/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/leaderboards')
  })
})
