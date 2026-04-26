import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, it, expect } from 'vitest'
import { ThemeProvider } from '../context/ThemeContext'
import { AuthProvider } from '../context/AuthContext'
import { AppFeaturesContext } from '../context/AppFeaturesContext'
import { LoginPage } from './LoginPage'
import type { AppFeatures } from '../types/api'

function renderLoginPage(features: AppFeatures) {
  return render(
    <AppFeaturesContext.Provider value={features}>
      <ThemeProvider>
        <AuthProvider>
          <MemoryRouter>
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
})
