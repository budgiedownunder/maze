import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getCachedImageUrl, getAvatarObjectUrl, resetImageCache } from '../../src/utils/imageCache'
import { fetchUserAvatar } from '../../src/api/client'

vi.mock('../../src/api/client', () => ({ fetchUserAvatar: vi.fn() }))
const mockFetchAvatar = vi.mocked(fetchUserAvatar)

let urlSeq = 0
beforeEach(() => {
  resetImageCache()
  vi.clearAllMocks()
  urlSeq = 0
  globalThis.URL.createObjectURL = vi.fn(() => `blob:${++urlSeq}`)
  globalThis.URL.revokeObjectURL = vi.fn()
})

const pngBlob = () => new Blob([new Uint8Array([1])], { type: 'image/png' })

describe('imageCache — getCachedImageUrl (generic core)', () => {
  it('fetches once per (kind,id,marker) and shares the object URL', async () => {
    const fetchBlob = vi.fn().mockResolvedValue(pngBlob())
    // Concurrent callers share one in-flight fetch.
    const [a, b] = await Promise.all([
      getCachedImageUrl('user', 'u1', 'm1', fetchBlob),
      getCachedImageUrl('user', 'u1', 'm1', fetchBlob),
    ])
    expect(a).toBe('blob:1')
    expect(b).toBe('blob:1')
    // A later caller reuses the cache — no new fetch.
    expect(await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)).toBe('blob:1')
    expect(fetchBlob).toHaveBeenCalledTimes(1)
  })

  it('namespaces by kind so the same id under different kinds does not collide', async () => {
    const userFetch = vi.fn().mockResolvedValue(pngBlob())
    const gameFetch = vi.fn().mockResolvedValue(pngBlob())
    expect(await getCachedImageUrl('user', 'x', 'm', userFetch)).toBe('blob:1')
    expect(await getCachedImageUrl('game-definition', 'x', 'm', gameFetch)).toBe('blob:2')
    expect(userFetch).toHaveBeenCalledTimes(1)
    expect(gameFetch).toHaveBeenCalledTimes(1)
  })

  it('caches a null (no image / failed fetch) without re-fetching', async () => {
    const fetchBlob = vi.fn().mockRejectedValue(new Error('404'))
    expect(await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)).toBeNull()
    expect(await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)).toBeNull()
    expect(fetchBlob).toHaveBeenCalledTimes(1)
  })

  it('re-fetches and revokes the stale blob when the marker changes', async () => {
    const fetchBlob = vi.fn().mockResolvedValue(pngBlob())
    expect(await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)).toBe('blob:1')
    expect(await getCachedImageUrl('user', 'u1', 'm2', fetchBlob)).toBe('blob:2')
    expect(fetchBlob).toHaveBeenCalledTimes(2)
    await Promise.resolve()
    expect(globalThis.URL.revokeObjectURL).toHaveBeenCalledWith('blob:1')
  })

  it('resetImageCache revokes resolved URLs and clears the cache', async () => {
    const fetchBlob = vi.fn().mockResolvedValue(pngBlob())
    await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)
    resetImageCache()
    await Promise.resolve()
    expect(globalThis.URL.revokeObjectURL).toHaveBeenCalledWith('blob:1')
    await getCachedImageUrl('user', 'u1', 'm1', fetchBlob)
    expect(fetchBlob).toHaveBeenCalledTimes(2)
  })
})

describe('imageCache — getAvatarObjectUrl (user wrapper)', () => {
  it('delegates to fetchUserAvatar under the user namespace and shares the cache', async () => {
    mockFetchAvatar.mockResolvedValue(pngBlob())
    expect(await getAvatarObjectUrl('tok', 'u1', 'm1')).toBe('blob:1')
    expect(mockFetchAvatar).toHaveBeenCalledWith('tok', 'u1', 'm1')
    // The generic API for the same subject hits the same entry (no new fetch).
    const shouldNotFetch = () => Promise.reject(new Error('should not fetch'))
    expect(await getCachedImageUrl('user', 'u1', 'm1', shouldNotFetch)).toBe('blob:1')
    expect(mockFetchAvatar).toHaveBeenCalledTimes(1)
  })
})
