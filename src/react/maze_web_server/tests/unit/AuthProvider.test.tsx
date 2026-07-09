import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { AuthProvider } from '../../src/context/AuthProvider'
import { useAuth } from '../../src/context/AuthContext'
import { getCachedImageUrl } from '../../src/utils/imageCache'

function LogoutButton() {
  const { logout } = useAuth()
  return <button type="button" onClick={() => void logout()}>do logout</button>
}

beforeEach(() => {
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:x')
  globalThis.URL.revokeObjectURL = vi.fn()
})

describe('AuthProvider logout', () => {
  it('clears the guarded-image cache on sign-out (client-side, no reload)', async () => {
    const fetchBlob = vi.fn().mockResolvedValue(new Blob([new Uint8Array([1])], { type: 'image/png' }))
    // Seed the cache with one entry.
    await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)
    expect(fetchBlob).toHaveBeenCalledTimes(1)

    render(<AuthProvider><LogoutButton /></AuthProvider>)
    await userEvent.click(screen.getByRole('button', { name: 'do logout' }))

    // The cache was cleared, so the same request re-fetches rather than reusing it.
    await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)
    expect(fetchBlob).toHaveBeenCalledTimes(2)
  })
})
