import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { AuthProvider } from '../../src/context/AuthProvider'
import { ChangePasswordModal } from '../../src/components/ChangePasswordModal'
import { mockLoginResponse } from '../../src/mocks/handlers'

const VALID_NEW_PASSWORD = 'NewPass1!'

beforeEach(() => {
  sessionStorage.setItem('auth', JSON.stringify({
    token: mockLoginResponse.login_token_id,
    issuedAt: new Date().toISOString(),
    expiry: mockLoginResponse.login_token_expires_at,
  }))
})
afterEach(() => sessionStorage.clear())

function renderChangeModal(onClose = vi.fn()) {
  return render(
    <MemoryRouter>
      <AuthProvider>
        <ChangePasswordModal onClose={onClose} hasPassword={true} />
      </AuthProvider>
    </MemoryRouter>
  )
}

function renderSetModal(onClose = vi.fn()) {
  return render(
    <MemoryRouter>
      <AuthProvider>
        <ChangePasswordModal onClose={onClose} hasPassword={false} />
      </AuthProvider>
    </MemoryRouter>
  )
}

async function fillChangeForm(current = 'OldPass1!', newPass = VALID_NEW_PASSWORD, confirm = VALID_NEW_PASSWORD) {
  await userEvent.type(screen.getByLabelText(/current password/i), current)
  await userEvent.type(screen.getByLabelText(/^new password$/i), newPass)
  await userEvent.type(screen.getByLabelText(/confirm new password/i), confirm)
}

async function fillSetForm(newPass = VALID_NEW_PASSWORD, confirm = VALID_NEW_PASSWORD) {
  await userEvent.type(screen.getByLabelText(/^new password$/i), newPass)
  await userEvent.type(screen.getByLabelText(/confirm new password/i), confirm)
}

describe('ChangePasswordModal — Change variant (hasPassword=true)', () => {
  it('renders the Change heading and three password fields', () => {
    renderChangeModal()
    expect(screen.getByRole('dialog', { name: /change password/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /change password/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/current password/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/^new password$/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/confirm new password/i)).toBeInTheDocument()
  })

  it('shows error when Change Password is clicked with empty fields', async () => {
    renderChangeModal()
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/all fields are required/i)
  })

  it('shows error when new passwords do not match', async () => {
    renderChangeModal()
    await fillChangeForm('OldPass1!', VALID_NEW_PASSWORD, 'Different1!')
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/do not match/i)
  })

  it('shows error when new password fails complexity', async () => {
    renderChangeModal()
    await fillChangeForm('OldPass1!', 'weak', 'weak')
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))
    expect(screen.getByRole('alert')).toHaveTextContent(/password must/i)
  })

  it('sends current_password + new_password and closes on success', async () => {
    let observedBody: unknown = null
    server.use(
      http.put('/api/v1/users/me/password', async ({ request }) => {
        observedBody = await request.json()
        return new HttpResponse(null, { status: 204 })
      }),
    )
    const onClose = vi.fn()
    renderChangeModal(onClose)
    await fillChangeForm()
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce())
    expect(observedBody).toEqual({
      current_password: 'OldPass1!',
      new_password: VALID_NEW_PASSWORD,
    })
  })

  it('fires onSuccess before onClose on a successful change', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => new HttpResponse(null, { status: 204 })),
    )
    const onClose = vi.fn()
    const onSuccess = vi.fn()
    render(
      <MemoryRouter>
        <AuthProvider>
          <ChangePasswordModal onClose={onClose} hasPassword={true} onSuccess={onSuccess} />
        </AuthProvider>
      </MemoryRouter>
    )
    await fillChangeForm()
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce())
    expect(onSuccess).toHaveBeenCalledOnce()
    // onSuccess fires before onClose — important for the parent to update
    // its `saved.has_password` state synchronously with the close transition.
    expect(onSuccess.mock.invocationCallOrder[0]).toBeLessThan(onClose.mock.invocationCallOrder[0])
  })

  it('does not fire onSuccess when the request fails', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => new HttpResponse(null, { status: 401 })),
    )
    const onSuccess = vi.fn()
    render(
      <MemoryRouter>
        <AuthProvider>
          <ChangePasswordModal onClose={vi.fn()} hasPassword={true} onSuccess={onSuccess} />
        </AuthProvider>
      </MemoryRouter>
    )
    await fillChangeForm()
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(onSuccess).not.toHaveBeenCalled()
  })

  it('shows "Current password is incorrect" on 401', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => HttpResponse.json(null, { status: 401 })),
    )
    renderChangeModal()
    await fillChangeForm()
    await userEvent.click(screen.getByRole('button', { name: /^change password$/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/current password is incorrect/i))
  })

  it('calls onClose without API call when Back is clicked', async () => {
    const onClose = vi.fn()
    renderChangeModal(onClose)
    await userEvent.click(screen.getByRole('button', { name: /back/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})

describe('ChangePasswordModal — Set variant (hasPassword=false)', () => {
  it('renders the Set heading and only two password fields (no Current Password)', () => {
    renderSetModal()
    expect(screen.getByRole('dialog', { name: /set password/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /set password/i })).toBeInTheDocument()
    expect(screen.queryByLabelText(/current password/i)).not.toBeInTheDocument()
    expect(screen.getByLabelText(/^new password$/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/confirm new password/i)).toBeInTheDocument()
  })

  it('sends only new_password (omits current_password) and closes on success', async () => {
    let observedBody: Record<string, unknown> | null = null
    server.use(
      http.put('/api/v1/users/me/password', async ({ request }) => {
        observedBody = await request.json() as Record<string, unknown>
        return new HttpResponse(null, { status: 204 })
      }),
    )
    const onClose = vi.fn()
    renderSetModal(onClose)
    await fillSetForm()
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))

    await waitFor(() => expect(onClose).toHaveBeenCalledOnce())
    expect(observedBody).toEqual({ new_password: VALID_NEW_PASSWORD })
    expect(observedBody).not.toHaveProperty('current_password')
  })

  it('fires onSuccess on a successful set', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => new HttpResponse(null, { status: 204 })),
    )
    const onSuccess = vi.fn()
    render(
      <MemoryRouter>
        <AuthProvider>
          <ChangePasswordModal onClose={vi.fn()} hasPassword={false} onSuccess={onSuccess} />
        </AuthProvider>
      </MemoryRouter>
    )
    await fillSetForm()
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))

    await waitFor(() => expect(onSuccess).toHaveBeenCalledOnce())
  })

  it('shows a generic failure message on 4xx', async () => {
    server.use(
      http.put('/api/v1/users/me/password', () => new HttpResponse(null, { status: 400 })),
    )
    renderSetModal()
    await fillSetForm()
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/failed to set password/i))
  })

  it('rejects mismatched new and confirm before sending', async () => {
    let networkCalled = false
    server.use(
      http.put('/api/v1/users/me/password', () => {
        networkCalled = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    renderSetModal()
    await fillSetForm(VALID_NEW_PASSWORD, 'Different1!')
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))

    expect(screen.getByRole('alert')).toHaveTextContent(/do not match/i)
    expect(networkCalled).toBe(false)
  })

  it('calls onClose without API call when Back is clicked', async () => {
    const onClose = vi.fn()
    renderSetModal(onClose)
    await userEvent.click(screen.getByRole('button', { name: /back/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})
