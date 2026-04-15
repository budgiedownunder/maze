import { render, screen } from '@testing-library/react'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import { describe, it, expect } from 'vitest'
import { AppFeaturesContext } from './context/AppFeaturesContext'
import { SignupRoute } from './App'

describe('SignupRoute', () => {
  it('redirects to /login when allow_signup is false', async () => {
    render(
      <AppFeaturesContext.Provider value={{ allow_signup: false }}>
        <MemoryRouter initialEntries={['/signup']}>
          <Routes>
            <Route path="/signup" element={<SignupRoute />} />
            <Route path="/login" element={<div data-testid="login-page" />} />
          </Routes>
        </MemoryRouter>
      </AppFeaturesContext.Provider>
    )
    expect(await screen.findByTestId('login-page')).toBeInTheDocument()
  })
})
