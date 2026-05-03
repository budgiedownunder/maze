import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, it, expect } from 'vitest'
import { ThemeProvider } from '../context/ThemeProvider'
import { AuthProvider } from '../context/AuthProvider'
import { AppFeaturesContext } from '../context/AppFeaturesContext'
import { LoginPage } from './LoginPage'
import type { AppFeatures } from '../types/api'

function renderLoginPage(features: AppFeatures, initialEntry: string = '/login') {
  return render(
    <AppFeaturesContext.Provider value={features}>
      <ThemeProvider>
        <AuthProvider>
          <MemoryRouter initialEntries={[initialEntry]}>
            <LoginPage />
          </MemoryRouter>
        </AuthProvider>
      </ThemeProvider>
    </AppFeaturesContext.Provider>
  )
}

describe('LoginPage', () => {
  it('shows signup button when allow_signup is true', async () => {
    renderLoginPage({ allow_signup: true, oauth_providers: [] })
    expect(await screen.findByRole('button', { name: /sign up/i })).toBeInTheDocument()
  })

  it('hides signup button when allow_signup is false', () => {
    renderLoginPage({ allow_signup: false, oauth_providers: [] })
    expect(screen.queryByRole('button', { name: /sign up/i })).not.toBeInTheDocument()
  })

  it('shows an OAuth button per configured provider', async () => {
    renderLoginPage({
      allow_signup: false,
      oauth_providers: [
        { name: 'google', display_name: 'Google' },
        { name: 'github', display_name: 'GitHub' },
      ],
    })
    expect(await screen.findByRole('button', { name: /continue with google/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /continue with github/i })).toBeInTheDocument()
  })

  it('hides the OAuth section when no providers are configured', () => {
    renderLoginPage({ allow_signup: true, oauth_providers: [] })
    expect(screen.queryByRole('button', { name: /continue with/i })).not.toBeInTheDocument()
  })

  it('surfaces ?error=signup_disabled to the user as a friendly alert', async () => {
    // Locks in the fix for the silent-failure bug when allow_signup=false and
    // a Google sign-in tries to create a new user. The server redirects to
    // /login?error=signup_disabled; the page must show, not swallow.
    renderLoginPage(
      { allow_signup: false, oauth_providers: [{ name: 'google', display_name: 'Google' }] },
      '/login?error=signup_disabled',
    )
    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(/sign-up is disabled/i)
  })

  it('surfaces ?error=email_not_verified', async () => {
    renderLoginPage({ allow_signup: true, oauth_providers: [] }, '/login?error=email_not_verified')
    expect(await screen.findByRole('alert')).toHaveTextContent(/verified email/i)
  })

  it('shows nothing for unknown error codes that look intentional but are not echoed raw', async () => {
    renderLoginPage({ allow_signup: true, oauth_providers: [] }, '/login?error=made_up_code')
    const alert = await screen.findByRole('alert')
    // The raw code must NOT leak into the UI.
    expect(alert).not.toHaveTextContent(/made_up_code/)
    expect(alert).toHaveTextContent(/could not sign you in/i)
  })
})
