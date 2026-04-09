import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { SignUpPage } from '../../src/pages/SignUpPage'

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

async function fillForm(overrides: Partial<Record<string, string>> = {}) {
  const fields = {
    username: 'newuser',
    fullName: 'New User',
    email: 'new@example.com',
    password: 'Password1!',
    confirmPassword: 'Password1!',
    ...overrides,
  }
  if (fields.username) await userEvent.type(screen.getByLabelText(/username/i), fields.username)
  if (fields.fullName) await userEvent.type(screen.getByLabelText(/full name/i), fields.fullName)
  if (fields.email) await userEvent.type(screen.getByLabelText(/email/i), fields.email)
  const passwordInputs = screen.getAllByLabelText(/password/i)
  if (fields.password) await userEvent.type(passwordInputs[0], fields.password)
  if (fields.confirmPassword) await userEvent.type(passwordInputs[1], fields.confirmPassword)
}

function renderSignUpPage() {
  return render(
    <MemoryRouter>
      <SignUpPage />
    </MemoryRouter>
  )
}

describe('SignUpPage', () => {
  it('disables Sign Up button when fields are empty', () => {
    renderSignUpPage()
    expect(screen.getByRole('button', { name: /sign up/i })).toBeDisabled()
  })

  it('shows validation error for mismatched passwords before API call', async () => {
    renderSignUpPage()
    await fillForm({ confirmPassword: 'Different1!' })
    await userEvent.click(screen.getByRole('button', { name: /sign up/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/match/)
  })

  it('shows validation error for weak password', async () => {
    renderSignUpPage()
    await fillForm({ password: 'weak', confirmPassword: 'weak' })
    await userEvent.click(screen.getByRole('button', { name: /sign up/i }))
    expect(screen.getByRole('alert')).toBeInTheDocument()
  })

  it('navigates to /login on successful signup', async () => {
    renderSignUpPage()
    await fillForm()
    await userEvent.click(screen.getByRole('button', { name: /sign up/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/login', { replace: true }))
  })

  it('shows error on 409 conflict', async () => {
    server.use(
      http.post('/api/v1/signup', () => HttpResponse.json(null, { status: 409 })),
    )
    renderSignUpPage()
    await fillForm()
    await userEvent.click(screen.getByRole('button', { name: /sign up/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/already in use/i))
  })
})
