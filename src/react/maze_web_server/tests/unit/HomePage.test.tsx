import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { HomePage } from '../../src/pages/HomePage'

const mockNavigate = vi.fn()

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
      profile: null,
      login: vi.fn(),
      logout: vi.fn(),
    }),
  }
})

function renderHomePage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <HomePage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('HomePage', () => {
  it('renders both tile titles', () => {
    renderHomePage()
    expect(screen.getByRole('heading', { name: /play 3d/i })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /design & play/i })).toBeInTheDocument()
  })

  it('clicking Play 3D opens the difficulty modal (no navigation yet)', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /play 3d/i }))

    expect(screen.getByRole('dialog', { name: /choose difficulty/i })).toBeInTheDocument()
    expect(locationStub.href).toBe('')
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('difficulty modal defaults to Tricky and Play navigates to /game/?difficulty=tricky', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /play 3d/i }))
    expect(screen.getByRole('radio', { name: /tricky/i })).toBeChecked()
    await userEvent.click(screen.getByRole('button', { name: /^play$/i }))

    expect(locationStub.href).toBe('/game/?difficulty=tricky')
  })

  it('choosing Easy then Play navigates to /game/?difficulty=easy', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /play 3d/i }))
    await userEvent.click(screen.getByRole('radio', { name: /easy/i }))
    await userEvent.click(screen.getByRole('button', { name: /^play$/i }))

    expect(locationStub.href).toBe('/game/?difficulty=easy')
  })

  it('cancelling the difficulty modal closes it without navigating', async () => {
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /play 3d/i }))
    await userEvent.click(screen.getByRole('button', { name: /cancel/i }))

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    expect(locationStub.href).toBe('')
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('clicking Design & Play navigates to /mazes', async () => {
    renderHomePage()
    await userEvent.click(screen.getByRole('button', { name: /design & play/i }))
    expect(mockNavigate).toHaveBeenCalledWith('/mazes')
  })
})
