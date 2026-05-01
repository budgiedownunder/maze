import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockLoginResponse, mockProfile } from '../../src/mocks/handlers'
import { AuthProvider } from '../../src/context/AuthContext'
import { AccountModal } from '../../src/components/AccountModal'

const mockNavigate = vi.fn()
const mockLogout = vi.fn()

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
      profile: mockProfile,
      login: vi.fn(),
      logout: mockLogout,
    }),
  }
})

function renderModal(onClose = vi.fn()) {
  return render(
    <MemoryRouter>
      <AuthProvider>
        <AccountModal onClose={onClose} />
      </AuthProvider>
    </MemoryRouter>
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  mockLogout.mockReset()
  sessionStorage.setItem('auth', JSON.stringify({
    token: mockLoginResponse.login_token_id,
    issuedAt: new Date().toISOString(),
    expiry: mockLoginResponse.login_token_expires_at,
  }))
})
afterEach(() => sessionStorage.clear())

describe('AccountModal', () => {
  it('shows loading state initially', () => {
    renderModal()
    expect(screen.getByText(/loading profile/i)).toBeInTheDocument()
  })

  it('pre-populates fields from profile', async () => {
    renderModal()
    await waitFor(() => expect(screen.getByDisplayValue(mockProfile.username)).toBeInTheDocument())
    expect(screen.getByDisplayValue(mockProfile.full_name)).toBeInTheDocument()
  })

  it('does not show Administrator badge for regular users', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(screen.queryByText(/administrator/i)).not.toBeInTheDocument()
  })

  it('renders the Email Addresses panel after the profile loads', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(await screen.findByRole('heading', { name: /email addresses/i })).toBeInTheDocument()
  })

  it('shows Administrator badge for admin users', async () => {
    server.use(
      http.get('/api/v1/users/me', () => HttpResponse.json({ ...mockProfile, is_admin: true })),
    )
    renderModal()
    await waitFor(() => expect(screen.getByText(/administrator/i)).toBeInTheDocument())
  })

  it('Save Profile button is disabled when no fields have changed', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(screen.getByRole('button', { name: /save profile/i })).toBeDisabled()
  })

  it('Save Profile button is enabled when a field changes', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'newusername')
    expect(screen.getByRole('button', { name: /save profile/i })).toBeEnabled()
  })

  it('Save Profile calls PUT and updates fields on success', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'updateduser')
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))
    await waitFor(() => expect(screen.getByDisplayValue('updateduser')).toBeInTheDocument())
  })

  it('shows 409 error when username already in use', async () => {
    server.use(
      http.put('/api/v1/users/me/profile', () => HttpResponse.json(null, { status: 409 })),
    )
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'takenuser')
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/already in use/i))
  })

  it('shows server error message on non-409 save failure', async () => {
    server.use(
      http.put('/api/v1/users/me/profile', () => HttpResponse.text('Username format is invalid', { status: 400 })),
    )
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'newname')
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/username format is invalid/i))
  })

  it('opens ChangePasswordModal when Change Password is clicked', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    expect(screen.getByRole('dialog', { name: /change password/i })).toBeInTheDocument()
  })

  it('shows delete confirmation step when Delete Account is clicked', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    expect(screen.getByText(/cannot be undone/i)).toBeInTheDocument()
  })

  it('returns to normal state when Cancel is clicked in delete confirmation', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(screen.queryByText(/cannot be undone/i)).not.toBeInTheDocument()
  })

  it('calls DELETE and navigates to /login on confirm delete', async () => {
    renderModal()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    await userEvent.click(screen.getByRole('button', { name: /^delete$/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/login', { replace: true }))
  })

  it('calls onClose when Close button is clicked', async () => {
    const onClose = vi.fn()
    renderModal(onClose)
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})
