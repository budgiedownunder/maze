import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { AuthProvider } from '../../src/context/AuthContext'
import { ChangePasswordModal } from '../../src/components/ChangePasswordModal'
import { mockLoginResponse } from '../../src/mocks/handlers'

beforeEach(() => {
  sessionStorage.setItem('auth', JSON.stringify({
    token: mockLoginResponse.login_token_id,
    issuedAt: new Date().toISOString(),
    expiry: mockLoginResponse.login_token_expires_at,
  }))
})
afterEach(() => sessionStorage.clear())

function renderModal(onClose = vi.fn()) {
  return render(
    <MemoryRouter>
      <AuthProvider>
        <ChangePasswordModal onClose={onClose} />
      </AuthProvider>
    </MemoryRouter>
  )
}

async function fillForm(current = 'OldPass1!', newPass = 'NewPass1!', confirm = 'NewPass1!') {
  const inputs = screen.getAllByLabelText(/password/i)
  await userEvent.type(inputs[0], current)
  await userEvent.type(inputs[1], newPass)
  await userEvent.type(inputs[2], confirm)
}

describe('ChangePasswordModal', () => {
  it('shows error when Change Password is clicked with empty fields', async () => {
    renderModal()
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/all fields are required/i)
  })

  it('shows error when new passwords do not match', async () => {
    renderModal()
    await fillForm('OldPass1!', 'NewPass1!', 'Different1!')
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/do not match/i)
  })

  it('shows error when new password fails complexity', async () => {
    renderModal()
    await fillForm('OldPass1!', 'weak', 'weak')
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/password must/i)
  })

  it('calls PUT and closes modal on success', async () => {
    const onClose = vi.fn()
    renderModal(onClose)
    await fillForm()
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce())
  })

  it('shows error on 401 (wrong current password)', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => HttpResponse.json(null, { status: 401 })),
    )
    renderModal()
    await fillForm()
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/current password is incorrect/i))
  })

  it('calls onClose without API call when Back is clicked', async () => {
    const onClose = vi.fn()
    renderModal(onClose)
    await userEvent.click(screen.getByRole('button', { name: /back/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})
