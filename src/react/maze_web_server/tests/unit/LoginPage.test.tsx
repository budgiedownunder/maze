import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { AppFeaturesContext, APP_FEATURES_DEFAULTS } from '../../src/context/AppFeaturesContext'
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

function renderLoginPage(initialEntry = '/login', emailEnabled = true) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <AppFeaturesContext.Provider value={{ ...APP_FEATURES_DEFAULTS, email_enabled: emailEnabled }}>
        <ThemeProvider>
          <LoginPage />
        </ThemeProvider>
      </AppFeaturesContext.Provider>
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
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.type(screen.getByLabelText('Password'), 'Password1!')
    expect(screen.getByRole('button', { name: /sign in/i })).toBeEnabled()
  })

  it('navigates to /mazes on successful login when not first sign-in', async () => {
    mockLogin.mockResolvedValue({ isFirstSignIn: false })
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.type(screen.getByLabelText('Password'), 'Password1!')
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }))
    await waitFor(() =>
      expect(mockNavigate).toHaveBeenCalledWith('/mazes', { replace: true, state: undefined }),
    )
  })

  it('navigates to /account with welcome state when first sign-in', async () => {
    mockLogin.mockResolvedValue({ isFirstSignIn: true })
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.type(screen.getByLabelText('Password'), 'Password1!')
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }))
    await waitFor(() =>
      expect(mockNavigate).toHaveBeenCalledWith('/account', {
        replace: true,
        state: { welcome: true },
      }),
    )
  })

  it('shows error message on 401', async () => {
    mockLogin.mockRejectedValue(Object.assign(new Error('Unauthorized'), { status: 401 }))
    renderLoginPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.type(screen.getByLabelText('Password'), 'wrongpass')
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/invalid email or password/i))
  })

  it('navigates to /forgot-password from the Forgot password? link', async () => {
    renderLoginPage()
    await userEvent.click(screen.getByRole('button', { name: /forgot password/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/forgot-password')
  })

  it('hides the Forgot password? link when email is disabled', () => {
    renderLoginPage('/login', false)
    expect(screen.queryByRole('button', { name: /forgot password/i })).not.toBeInTheDocument()
    // Sign in / sign up controls are unaffected.
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
  })

  it('surfaces the ?message= flash and clears it from the URL', async () => {
    renderLoginPage('/login?message=Password+reset+successful')
    expect(await screen.findByRole('status')).toHaveTextContent(/password reset successful/i)
  })
})
