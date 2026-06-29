import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { AppHeader } from '../../src/components/AppHeader'
import type { UserProfile } from '../../src/types/api'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

// Read lazily inside `useAuth` so each test can set the auth state in beforeEach.
let authValue: unknown

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return { ...actual, useAuth: () => authValue }
})

const profile: UserProfile = {
  id: 'u1',
  username: 'alice',
  full_name: 'Alice Example',
  email: 'alice@example.com',
  emails: [],
  is_admin: false,
  has_password: true,
}

function renderHeader() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <AppHeader title="Home" />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  authValue = { isLoading: false, isAuthenticated: true, profile, login: vi.fn(), logout: vi.fn() }
})

describe('AppHeader', () => {
  it('shows the signed-in username and links to the account page', async () => {
    renderHeader()
    const usernameButton = screen.getByRole('button', { name: 'alice' })
    expect(usernameButton).toBeInTheDocument()
    await userEvent.click(usernameButton)
    expect(mockNavigate).toHaveBeenCalledWith('/account')
  })

  it('hides the username when not signed in', () => {
    authValue = { isLoading: false, isAuthenticated: false, profile: null, login: vi.fn(), logout: vi.fn() }
    renderHeader()
    expect(screen.queryByRole('button', { name: 'alice' })).not.toBeInTheDocument()
  })

  it('renders the title and the theme toggle', () => {
    renderHeader()
    expect(screen.getByText('Home')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /switch to (light|dark) mode/i })).toBeInTheDocument()
  })
})
