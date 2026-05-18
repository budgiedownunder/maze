import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockVerificationTokens } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { VerifyEmailPage } from '../../src/pages/VerifyEmailPage'

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

let mockIsAuthenticated = false
vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useAuth: () => ({
      isLoading: false,
      isAuthenticated: mockIsAuthenticated,
      profile: null,
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

function renderVerifyEmailPage(initialEntry: string) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <ThemeProvider>
        <VerifyEmailPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  mockIsAuthenticated = false
})

describe('VerifyEmailPage', () => {
  it('shows the invalid state immediately when no token is in the query string', () => {
    renderVerifyEmailPage('/verify-email')
    expect(screen.getByRole('alert')).toHaveTextContent(/invalid or has already been used/i)
  })

  it('shows the success state on a valid token', async () => {
    mockVerificationTokens.set('valid-verify-token', 'test@example.com')

    renderVerifyEmailPage('/verify-email?token=valid-verify-token')

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(/email verified/i))
    expect(mockVerificationTokens.has('valid-verify-token')).toBe(false)
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
  })

  it('shows Continue to your account when the user is authenticated', async () => {
    mockIsAuthenticated = true
    mockVerificationTokens.set('valid-verify-token', 'test@example.com')

    renderVerifyEmailPage('/verify-email?token=valid-verify-token')

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(/email verified/i))
    await userEvent.click(screen.getByRole('button', { name: /continue to your account/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/')
  })

  it('shows the invalid state on a 400 response', async () => {
    renderVerifyEmailPage('/verify-email?token=stale-verify-token')

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/invalid or has already been used/i))
  })

  it('shows the invalid state on a transport failure', async () => {
    server.use(
      http.post('/api/v1/email-verifications/confirm', () => HttpResponse.error()),
    )
    renderVerifyEmailPage('/verify-email?token=any-token')

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/invalid or has already been used/i))
  })

  it('navigates to /login from the invalid Back-to-sign-in button', async () => {
    renderVerifyEmailPage('/verify-email')
    await userEvent.click(screen.getByRole('button', { name: /back to sign in/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/login')
  })

  it('does not re-submit the token on re-render (StrictMode-safe via submittedRef)', async () => {
    mockVerificationTokens.set('rerender-token', 'test@example.com')
    let confirmCalls = 0
    server.use(
      http.post('/api/v1/email-verifications/confirm', () => {
        confirmCalls += 1
        mockVerificationTokens.delete('rerender-token')
        return new HttpResponse(null, { status: 200 })
      }),
    )

    const { rerender } = renderVerifyEmailPage('/verify-email?token=rerender-token')
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(/email verified/i))
    rerender(
      <MemoryRouter initialEntries={['/verify-email?token=rerender-token']}>
        <ThemeProvider>
          <VerifyEmailPage />
        </ThemeProvider>
      </MemoryRouter>,
    )
    expect(confirmCalls).toBe(1)
  })
})
