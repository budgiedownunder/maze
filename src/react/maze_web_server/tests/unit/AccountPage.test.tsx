import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { mockLoginResponse, mockProfile } from '../../src/mocks/handlers'
import { AuthProvider } from '../../src/context/AuthProvider'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { AccountPage } from '../../src/pages/AccountPage'

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
      refreshProfile: vi.fn(),
    }),
  }
})

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <AuthProvider>
          <AccountPage />
        </AuthProvider>
      </ThemeProvider>
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

describe('AccountPage', () => {
  it('shows loading state initially', () => {
    renderPage()
    expect(screen.getByText(/loading profile/i)).toBeInTheDocument()
  })

  it('pre-populates fields from profile', async () => {
    renderPage()
    await waitFor(() => expect(screen.getByDisplayValue(mockProfile.username)).toBeInTheDocument())
    expect(screen.getByDisplayValue(mockProfile.full_name)).toBeInTheDocument()
  })

  it('does not show Administrator badge for regular users', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(screen.queryByText(/administrator/i)).not.toBeInTheDocument()
  })

  it('renders the Email Addresses panel after the profile loads', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(await screen.findByRole('heading', { name: /email addresses/i })).toBeInTheDocument()
  })

  it('shows Administrator badge for admin users', async () => {
    server.use(
      http.get('/api/v1/users/me', () => HttpResponse.json({ ...mockProfile, is_admin: true })),
    )
    renderPage()
    await waitFor(() => expect(screen.getByText(/administrator/i)).toBeInTheDocument())
  })

  it('Save Profile button is disabled when no fields have changed', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(screen.getByRole('button', { name: /save profile/i })).toBeDisabled()
  })

  it('Save Profile button is enabled when a field changes', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'newusername')
    expect(screen.getByRole('button', { name: /save profile/i })).toBeEnabled()
  })

  it('Save Profile calls PUT and updates fields on success', async () => {
    renderPage()
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
    renderPage()
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
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.clear(screen.getByDisplayValue(mockProfile.username))
    await userEvent.type(screen.getByLabelText(/username/i), 'newname')
    await userEvent.click(screen.getByRole('button', { name: /save profile/i }))
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/username format is invalid/i))
  })

  it('opens ChangePasswordModal when Change Password is clicked', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /change password/i }))
    expect(screen.getByRole('dialog', { name: /change password/i })).toBeInTheDocument()
  })

  it('shows "Set Password" trigger and opens the Set variant when has_password is false', async () => {
    server.use(
      http.get('/api/v1/users/me', () => HttpResponse.json({ ...mockProfile, has_password: false })),
    )
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    const trigger = screen.getByRole('button', { name: /^set password$/i })
    expect(trigger).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^change password$/i })).not.toBeInTheDocument()
    await userEvent.click(trigger)
    expect(screen.getByRole('dialog', { name: /set password/i })).toBeInTheDocument()
  })

  it('flips trigger button text from Set Password to Change Password after a successful set', async () => {
    server.use(
      http.get('/api/v1/users/me', () => HttpResponse.json({ ...mockProfile, has_password: false })),
      http.put('/api/v1/users/me/password', () => new HttpResponse(null, { status: 204 })),
    )
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    expect(screen.getByRole('button', { name: /^set password$/i })).toBeInTheDocument()

    // Open the Set variant, fill it, submit.
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))
    await userEvent.type(screen.getByLabelText(/^new password$/i), 'NewPass1!')
    await userEvent.type(screen.getByLabelText(/confirm new password/i), 'NewPass1!')
    await userEvent.click(screen.getByRole('button', { name: /^set password$/i }))

    // After the password modal closes, the trigger should read Change Password —
    // the parent flipped its local has_password optimistically on success.
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /^change password$/i })).toBeInTheDocument()
    )
    expect(screen.queryByRole('button', { name: /^set password$/i })).not.toBeInTheDocument()
  })

  it('shows delete confirmation step when Delete Account is clicked', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    expect(screen.getByText(/cannot be undone/i)).toBeInTheDocument()
  })

  it('returns to normal state when Cancel is clicked in delete confirmation', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))
    expect(screen.queryByText(/cannot be undone/i)).not.toBeInTheDocument()
  })

  it('calls DELETE and navigates to /login on confirm delete', async () => {
    renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.click(screen.getByRole('button', { name: /delete account/i }))
    await userEvent.click(screen.getByRole('button', { name: /^delete$/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/login', { replace: true }))
  })

  // --- Avatar upload / change / remove -------------------------------------

  function accountFileInput(container: HTMLElement): HTMLInputElement {
    return container.querySelector('.account-avatar input[type="file"]') as HTMLInputElement
  }

  it('uploads an avatar, then shows Remove and renders the fetched image', async () => {
    const { container } = renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    // Pre-upload: no avatar → placeholder, no Remove.
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument()

    const file = new File([new Uint8Array([1, 2, 3])], 'me.png', { type: 'image/png' })
    await userEvent.upload(accountFileInput(container), file)

    // The stateful mock now reports an avatar → Remove appears and the account
    // avatar renders the fetched image (object URL), not the placeholder.
    await waitFor(() => expect(screen.getByRole('button', { name: /remove/i })).toBeInTheDocument())
    const img = container.querySelector('.account-avatar img') as HTMLImageElement
    await waitFor(() => expect(img.getAttribute('src')).toBe('blob:mock'))
  })

  it('rejects a non-image file client-side without uploading', async () => {
    const { container } = renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    // Use fireEvent to bypass the input's `accept` filter (which userEvent
    // honours) and exercise the handler's own defence-in-depth type guard.
    const file = new File(['hello'], 'notes.txt', { type: 'text/plain' })
    fireEvent.change(accountFileInput(container), { target: { files: [file] } })
    expect(await screen.findByText(/png or jpeg/i)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument()
  })

  it('rejects an oversize file client-side', async () => {
    const { container } = renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    const big = new File([new Uint8Array(2 * 1024 * 1024 + 1)], 'big.png', { type: 'image/png' })
    await userEvent.upload(accountFileInput(container), big)
    expect(await screen.findByText(/2 mb or smaller/i)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument()
  })

  it('removes an existing avatar', async () => {
    const { container } = renderPage()
    await waitFor(() => screen.getByDisplayValue(mockProfile.username))
    await userEvent.upload(accountFileInput(container), new File([new Uint8Array([1])], 'me.png', { type: 'image/png' }))
    const removeBtn = await screen.findByRole('button', { name: /remove/i })
    await userEvent.click(removeBtn)
    await waitFor(() => expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument())
    const img = container.querySelector('.account-avatar img') as HTMLImageElement
    await waitFor(() => expect(img.getAttribute('src')).toBe('/images/avatar-placeholder.png'))
  })
})
