import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockResetTokens } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { ResetPasswordPage } from '../../src/pages/ResetPasswordPage'

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

function renderResetPasswordPage(initialEntry: string) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <ThemeProvider>
        <ResetPasswordPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

async function fillNewPassword(value: string, confirm = value) {
  const inputs = screen.getAllByLabelText(/password/i)
  await userEvent.type(inputs[0], value)
  await userEvent.type(inputs[1], confirm)
}

beforeEach(() => {
  mockNavigate.mockReset()
})

describe('ResetPasswordPage', () => {
  it('shows the invalid-link state when no token is in the query string', () => {
    renderResetPasswordPage('/reset-password')
    expect(screen.getByRole('alert')).toHaveTextContent(/invalid/i)
    expect(screen.queryByRole('button', { name: /set new password/i })).not.toBeInTheDocument()
  })

  it('disables Set New Password until both fields are filled', async () => {
    renderResetPasswordPage('/reset-password?token=abc')
    const submit = screen.getByRole('button', { name: /set new password/i })
    expect(submit).toBeDisabled()

    await fillNewPassword('Password1!')
    expect(submit).toBeEnabled()
  })

  it('shows local validation error when passwords do not match', async () => {
    renderResetPasswordPage('/reset-password?token=abc')
    await fillNewPassword('Password1!', 'Different1!')
    await userEvent.click(screen.getByRole('button', { name: /set new password/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/match/i)
  })

  it('shows local validation error for a weak new password', async () => {
    renderResetPasswordPage('/reset-password?token=abc')
    await fillNewPassword('weak')
    await userEvent.click(screen.getByRole('button', { name: /set new password/i }))
    expect(screen.getByRole('alert')).toBeInTheDocument()
  })

  it('navigates to /login with a success flash on a valid token', async () => {
    // Mint a real reset token via the mock so the confirm handler accepts it.
    mockResetTokens.set('valid-token', 'test@example.com')

    renderResetPasswordPage('/reset-password?token=valid-token')
    await fillNewPassword('Password1!')
    await userEvent.click(screen.getByRole('button', { name: /set new password/i }))

    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith(
      expect.stringMatching(/^\/login\?message=/),
      { replace: true },
    ))
    expect(mockResetTokens.has('valid-token')).toBe(false)
  })

  it('shows an expired-link error on 400', async () => {
    renderResetPasswordPage('/reset-password?token=stale-token')
    await fillNewPassword('Password1!')
    await userEvent.click(screen.getByRole('button', { name: /set new password/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/invalid or has expired/i))
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('shows a generic error on a non-400 failure', async () => {
    server.use(
      http.post('/api/v1/password-reset/confirm', () => HttpResponse.error()),
    )
    renderResetPasswordPage('/reset-password?token=abc')
    await fillNewPassword('Password1!')
    await userEvent.click(screen.getByRole('button', { name: /set new password/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/could not reset/i))
  })

  it('navigates back to /login from Back to sign in', async () => {
    renderResetPasswordPage('/reset-password?token=abc')
    await userEvent.click(screen.getByRole('button', { name: /back to sign in/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/login')
  })
})
