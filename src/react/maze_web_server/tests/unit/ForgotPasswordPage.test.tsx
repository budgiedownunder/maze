import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockResetTokens } from '../../src/mocks/handlers'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { ForgotPasswordPage } from '../../src/pages/ForgotPasswordPage'

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

function renderForgotPasswordPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <ForgotPasswordPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

const SUCCESS_COPY = /if that email is registered/i

describe('ForgotPasswordPage', () => {
  it('disables Send Reset Link until a valid email is entered', async () => {
    renderForgotPasswordPage()
    const submit = screen.getByRole('button', { name: /send reset link/i })
    expect(submit).toBeDisabled()

    await userEvent.type(screen.getByLabelText(/email/i), 'not-an-email')
    expect(submit).toBeDisabled()

    await userEvent.clear(screen.getByLabelText(/email/i))
    await userEvent.type(screen.getByLabelText(/email/i), 'someone@example.com')
    expect(submit).toBeEnabled()
  })

  it('shows the anti-enumeration success copy after submitting a known email', async () => {
    renderForgotPasswordPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(SUCCESS_COPY))
    expect([...mockResetTokens.values()]).toContain('test@example.com')
  })

  it('shows the same success copy for an unknown email (no enumeration)', async () => {
    renderForgotPasswordPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'stranger@example.com')
    await userEvent.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(SUCCESS_COPY))
    expect([...mockResetTokens.values()]).not.toContain('stranger@example.com')
  })

  it('shows a retry error on transport failure (no leak of email existence)', async () => {
    server.use(
      http.post('/api/v1/password-reset/request', () => HttpResponse.error()),
    )
    renderForgotPasswordPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/could not send/i))
    expect(screen.queryByRole('status')).not.toBeInTheDocument()
  })

  it('navigates back to /login from the form', async () => {
    renderForgotPasswordPage()
    await userEvent.click(screen.getByRole('button', { name: /back to sign in/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/login')
  })

  it('navigates back to /login from the success state', async () => {
    renderForgotPasswordPage()
    await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com')
    await userEvent.click(screen.getByRole('button', { name: /send reset link/i }))

    await waitFor(() => expect(screen.getByRole('status')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: /back to sign in/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/login')
  })
})
