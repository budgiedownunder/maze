import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { AuthProvider } from '../../src/context/AuthProvider'
import { HamburgerMenu } from '../../src/components/HamburgerMenu'
import { mockLoginResponse } from '../../src/mocks/handlers'

beforeEach(() => {
  sessionStorage.setItem('auth', JSON.stringify({
    token: mockLoginResponse.login_token_id,
    issuedAt: new Date().toISOString(),
    expiry: mockLoginResponse.login_token_expires_at,
  }))
  mockNavigate.mockReset()
})
afterEach(() => {
  sessionStorage.clear()
  vi.unstubAllGlobals()
})

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

  it('navigates to /account when My Account is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /my account/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/account')
  })

  it('navigates to /leaderboards when Leaderboards is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /^leaderboards$/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/leaderboards')
  })

  it('navigates to / when Home is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /^home$/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/')
  })

  it('navigates to /mazes when Mazes is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /^mazes$/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/mazes')
  })

  it('navigates to /workshop when 3D Game Workshop is clicked', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /3d game workshop/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop')
  })

  it('navigates to /workshop/games from the Manage Games sub-item', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /^manage games$/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop/games')
  })

  it('navigates to /workshop/game-collections from the Manage Game Collections sub-item', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /^manage game collections$/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/workshop/game-collections')
  })

  it('hides the admin Manage Features sub-item for a non-admin', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    expect(screen.queryByRole('menuitem', { name: /^manage features$/i })).not.toBeInTheDocument()
  })

  it('Play 3D opens the difficulty modal (no navigation yet)', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /play 3d/i }))

    expect(screen.getByRole('dialog', { name: /choose difficulty/i })).toBeInTheDocument()
    expect(locationStub.href).toBe('')
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('difficulty modal defaults to Easy and Play navigates to /game/?difficulty=easy', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /play 3d/i }))
    expect(screen.getByRole('radio', { name: /easy/i })).toBeChecked()
    await userEvent.click(screen.getByRole('button', { name: /^play$/i }))

    expect(locationStub.href).toBe('/game/?difficulty=easy')
  })

  it('choosing Hard then Play navigates to /game/?difficulty=hard', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /play 3d/i }))
    await userEvent.click(screen.getByRole('radio', { name: /hard/i }))
    await userEvent.click(screen.getByRole('button', { name: /^play$/i }))

    expect(locationStub.href).toBe('/game/?difficulty=hard')
  })

  it('cancelling the difficulty modal closes it without navigating', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /play 3d/i }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    expect(locationStub.href).toBe('')
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('renders three separators dividing home / nav / account / about groups', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    expect(screen.getAllByRole('separator')).toHaveLength(3)
  })

  it('calls logout and navigates to /login on Sign Out', async () => {
    renderMenu()
    await userEvent.click(screen.getByRole('button', { name: /open menu/i }))
    await userEvent.click(screen.getByRole('menuitem', { name: /sign out/i }))
    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith('/login', { replace: true }))
  })
})
