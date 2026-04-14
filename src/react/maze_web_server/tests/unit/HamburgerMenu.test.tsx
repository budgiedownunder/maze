import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { AuthProvider } from '../../src/context/AuthContext'
import { HamburgerMenu } from '../../src/components/HamburgerMenu'
import { mockLoginResponse } from '../../src/mocks/handlers'

beforeEach(() => {
  sessionStorage.setItem('auth', JSON.stringify({
    token: mockLoginResponse.login_token_id,
    issuedAt: new Date().toISOString(),
    expiry: mockLoginResponse.login_token_expires_at,
  }))
})
afterEach(() => sessionStorage.clear())

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

function renderMenu() {
  return render(
    <MemoryRouter>
      <AuthProvider>
        <HamburgerMenu />
      </AuthProvider>
    </MemoryRouter>
  )
}

describe('HamburgerMenu', () => {
  it('menu items are not visible by default', () => {
    renderMenu()
    expect(screen.queryByRole('menu')).not.toBeInTheDocument()
  })

  it('opens menu on button click', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    expect(screen.getByRole('menu')).toBeInTheDocument()
  })

  it('closes menu on outside click', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(document.body)
    await waitFor(() => expect(screen.queryByRole('menu')).not.toBeInTheDocument())
  })

  it('opens About modal when About is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /about/i }))
    expect(screen.getByRole('dialog', { name: /about/i })).toBeInTheDocument()
  })

  it('opens Account modal when My Account is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /my account/i }))
    expect(screen.getByRole('dialog', { name: /my account/i })).toBeInTheDocument()
  })

  it('calls logout and navigates to /login on Sign Out', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /sign out/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/login', { replace: true }))
  })
})
