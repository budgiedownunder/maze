import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ThemeProvider, useTheme } from '../../src/context/ThemeContext'

function ThemeDisplay() {
  const { theme, toggleTheme } = useTheme()
  return (
    <>
      <span data-testid="theme">{theme}</span>
      <button onClick={toggleTheme}>Toggle</button>
    </>
  )
}

function renderWithProvider() {
  return render(
    <ThemeProvider>
      <ThemeDisplay />
    </ThemeProvider>
  )
}

beforeEach(() => {
  localStorage.clear()
  delete document.documentElement.dataset.theme
})

afterEach(() => {
  vi.restoreAllMocks()
  localStorage.clear()
  delete document.documentElement.dataset.theme
})

describe('ThemeContext', () => {
  it('reads stored light preference from localStorage', () => {
    localStorage.setItem('theme', 'light')
    renderWithProvider()
    expect(screen.getByTestId('theme')).toHaveTextContent('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('reads stored dark preference from localStorage', () => {
    localStorage.setItem('theme', 'dark')
    renderWithProvider()
    expect(screen.getByTestId('theme')).toHaveTextContent('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('falls back to OS dark preference when no stored value', () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    }))
    renderWithProvider()
    expect(screen.getByTestId('theme')).toHaveTextContent('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('falls back to light when OS has no dark preference and no stored value', () => {
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    }))
    renderWithProvider()
    expect(screen.getByTestId('theme')).toHaveTextContent('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('toggleTheme switches from light to dark', async () => {
    localStorage.setItem('theme', 'light')
    renderWithProvider()
    await userEvent.click(screen.getByRole('button', { name: /toggle/i }))
    expect(screen.getByTestId('theme')).toHaveTextContent('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(localStorage.getItem('theme')).toBe('dark')
  })

  it('toggleTheme switches from dark to light', async () => {
    localStorage.setItem('theme', 'dark')
    renderWithProvider()
    await userEvent.click(screen.getByRole('button', { name: /toggle/i }))
    expect(screen.getByTestId('theme')).toHaveTextContent('light')
    expect(document.documentElement.dataset.theme).toBe('light')
    expect(localStorage.getItem('theme')).toBe('light')
  })

  it('persists toggled theme to localStorage', async () => {
    localStorage.setItem('theme', 'light')
    renderWithProvider()
    await userEvent.click(screen.getByRole('button', { name: /toggle/i }))
    expect(localStorage.getItem('theme')).toBe('dark')
  })

  it('useTheme throws when used outside ThemeProvider', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    expect(() => render(<ThemeDisplay />)).toThrow('useTheme must be used within ThemeProvider')
    spy.mockRestore()
  })
})
