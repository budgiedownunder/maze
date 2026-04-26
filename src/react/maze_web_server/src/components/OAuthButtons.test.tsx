import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { OAuthButtons } from './OAuthButtons'
import * as api from '../api/client'

describe('OAuthButtons', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('renders nothing when the providers list is empty', () => {
    const { container } = render(<OAuthButtons providers={[]} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders one button per provider with the display_name', () => {
    render(<OAuthButtons providers={[
      { name: 'google', display_name: 'Google' },
      { name: 'github', display_name: 'GitHub' },
    ]} />)
    expect(screen.getByRole('button', { name: /continue with google/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /continue with github/i })).toBeInTheDocument()
  })

  it('clicking a button calls api.startOAuth with the provider name', async () => {
    const startOAuth = vi.spyOn(api, 'startOAuth').mockImplementation(() => {})
    render(<OAuthButtons providers={[{ name: 'google', display_name: 'Google' }]} />)
    await userEvent.click(screen.getByRole('button', { name: /continue with google/i }))
    expect(startOAuth).toHaveBeenCalledWith('google')
  })

  it('respects the disabled prop', () => {
    render(<OAuthButtons providers={[{ name: 'google', display_name: 'Google' }]} disabled />)
    expect(screen.getByRole('button', { name: /continue with google/i })).toBeDisabled()
  })
})
