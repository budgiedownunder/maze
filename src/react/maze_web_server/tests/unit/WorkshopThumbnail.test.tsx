import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, waitFor } from '@testing-library/react'
import { WorkshopThumbnail } from '../../src/components/WorkshopListPage'
import { fetchGameImage } from '../../src/api/client'

// Isolate render logic from the network; the authenticated fetch is covered by
// gameImageApi.test.ts. The cache resolves through fetchGameImage.
vi.mock('../../src/api/client', () => ({ fetchGameImage: vi.fn() }))
vi.mock('../../src/context/AuthContext', () => ({ useToken: () => 'test-token' }))

const mockFetch = vi.mocked(fetchGameImage)
const GAME_PLACEHOLDER = '/images/workshop/workshop-game.svg'

function baseSrc(container: HTMLElement): string | null {
  return container.querySelector('.game-thumb-base')?.getAttribute('src') ?? null
}

beforeEach(async () => {
  mockFetch.mockReset()
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:mock-url')
  globalThis.URL.revokeObjectURL = vi.fn()
  // The cache is a module singleton; clear it so a prior test's entry doesn't
  // satisfy this one (lazy import so the mocked client is already in place).
  const { resetImageCache } = await import('../../src/utils/imageCache')
  resetImageCache()
})

describe('WorkshopThumbnail image display', () => {
  it('shows the placeholder art and makes no request without an imageSubject', () => {
    const { container } = render(<WorkshopThumbnail baseSrc={GAME_PLACEHOLDER} visibility="public" />)
    expect(baseSrc(container)).toBe(GAME_PLACEHOLDER)
    expect(mockFetch).not.toHaveBeenCalled()
  })

  it('shows the placeholder when the subject has no image marker', () => {
    const { container } = render(
      <WorkshopThumbnail baseSrc={GAME_PLACEHOLDER} visibility="public" imageSubject={{ kind: 'definition', id: 'g1' }} />,
    )
    expect(baseSrc(container)).toBe(GAME_PLACEHOLDER)
    expect(mockFetch).not.toHaveBeenCalled()
  })

  it('renders the uploaded image (via the cache) when a marker is set', async () => {
    mockFetch.mockResolvedValue(new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' }))
    const { container } = render(
      <WorkshopThumbnail baseSrc={GAME_PLACEHOLDER} visibility="public" imageSubject={{ kind: 'definition', id: 'g1', imageUpdatedAt: '2026-03-01T00:00:00Z' }} />,
    )
    await waitFor(() => expect(baseSrc(container)).toBe('blob:mock-url'))
    expect(mockFetch).toHaveBeenCalledWith('test-token', 'definition', 'g1', '2026-03-01T00:00:00Z')
  })

  it('falls back to the placeholder when the image fetch fails', async () => {
    mockFetch.mockRejectedValue(Object.assign(new Error('gone'), { status: 404 }))
    const { container } = render(
      <WorkshopThumbnail baseSrc={GAME_PLACEHOLDER} visibility="public" imageSubject={{ kind: 'collection', id: 'c1', imageUpdatedAt: '2026-03-01T00:00:00Z' }} />,
    )
    await waitFor(() => expect(mockFetch).toHaveBeenCalled())
    expect(baseSrc(container)).toBe(GAME_PLACEHOLDER)
  })
})
