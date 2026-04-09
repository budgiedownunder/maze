import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { ThemeProvider } from '../../src/context/ThemeContext'
import { LoginPage } from '../../src/pages/LoginPage'

const mockNavigate = vi.fn()
const mockLogin = vi.fn()

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
      isAuthenticated: false,
      profile: null,
      login: mockLogin,
      logout: vi.fn(),
    }),
  }
})

function renderLoginPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <LoginPage />
      </ThemeProvider>
    </MemoryRouter>
  )
}

beforeEach(() => {
  mockLogin.mockReset()
  mockNavigate.mockReset()
})

describe('LoginPage', () => {
  it('disables Sign In button when fields are empty', () => {
    renderLoginPage()
    expect(screen.getByRole('button', { name: /sign in/i })).toBeDisabled()
  })

  it('enables Sign In button when both fields are filled', async () => {
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/username/i), 'testuser')
    await userEvent.type(screen.getByLabelText('Password'), 'Password1!')
    expect(screen.getByRole('button', { name: /sign in/i })).toBeEnabled()
  })

  it('navigates to /mazes on successful login', async () => {
    mockLogin.mockResolvedValue(undefined)
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/username/i), 'testuser')
    await userEvent.type(screen.getByLabelText('Password'), 'Password1!')
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/mazes', { replace: true }))
  })

  it('shows error message on 401', async () => {
    mockLogin.mockRejectedValue(Object.assign(new Error('Unauthorized'), { status: 401 }))
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/username/i), 'testuser')
    await userEvent.type(screen.getByLabelText('Password'), 'wrongpass')
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/invalid username or password/i))
  })
})
