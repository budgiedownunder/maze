import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@testing-library/react'
import { Avatar } from '../../src/components/Avatar'
import { fetchUserAvatar } from '../../src/api/client'

// Isolate the component's render logic from the network: the authenticated
// fetch + bearer plumbing is covered by avatarApi.test.ts.
vi.mock('../../src/api/client', () => ({ fetchUserAvatar: vi.fn() }))
vi.mock('../../src/context/AuthContext', () => ({ useToken: () => 'test-token' }))

const mockFetchAvatar = vi.mocked(fetchUserAvatar)
const PLACEHOLDER = '/images/avatar-placeholder.png'

// The avatar img is decorative (alt=""), so it has no "img" ARIA role — query
// the element directly rather than by role.
function avatarSrc(container: HTMLElement): string | null {
  return container.querySelector('img')?.getAttribute('src') ?? null
}

beforeEach(() => {
  mockFetchAvatar.mockReset()
  // jsdom doesn't implement object URLs — stub them.
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:mock-url')
  globalThis.URL.revokeObjectURL = vi.fn()
})

describe('Avatar', () => {
  it('shows the placeholder and makes no request when there is no avatar marker', () => {
    const { container } = render(<Avatar userId="u1" avatarUpdatedAt={null} />)
    expect(avatarSrc(container)).toBe(PLACEHOLDER)
    expect(mockFetchAvatar).not.toHaveBeenCalled()
  })

  it('fetches the avatar and renders it from an object URL when the marker is set', async () => {
    mockFetchAvatar.mockResolvedValue(new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' }))
    const { container } = render(<Avatar userId="u1" avatarUpdatedAt="2025-04-01T12:00:00Z" />)
    await waitFor(() => expect(avatarSrc(container)).toBe('blob:mock-url'))
    expect(mockFetchAvatar).toHaveBeenCalledWith('test-token', 'u1', '2025-04-01T12:00:00Z')
  })

  it('falls back to the placeholder when the fetch fails (e.g. 404)', async () => {
    mockFetchAvatar.mockRejectedValue(Object.assign(new Error('not found'), { status: 404 }))
    const { container } = render(<Avatar userId="u1" avatarUpdatedAt="2025-04-01T12:00:00Z" />)
    await waitFor(() => expect(mockFetchAvatar).toHaveBeenCalled())
    expect(avatarSrc(container)).toBe(PLACEHOLDER)
  })

  it('does not revoke the object URL on unmount (the shared cache owns it)', async () => {
    mockFetchAvatar.mockResolvedValue(new Blob([new Uint8Array([1])], { type: 'image/png' }))
    const { container, unmount } = render(<Avatar userId="u1" avatarUpdatedAt="2025-04-01T12:00:00Z" />)
    await waitFor(() => expect(avatarSrc(container)).toBe('blob:mock-url'))
    unmount()
    expect(globalThis.URL.revokeObjectURL).not.toHaveBeenCalled()
  })
})
