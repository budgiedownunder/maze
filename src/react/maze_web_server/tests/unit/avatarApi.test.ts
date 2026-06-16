import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { avatarUrl, fetchUserAvatar } from '../../src/api/client'

const TOKEN = 'test-token'

describe('avatarUrl', () => {
  it('builds the avatar path for a user id', () => {
    expect(avatarUrl('u1')).toBe('/api/v1/users/u1/avatar')
  })

  it('appends the marker as a ?v= cache-buster when given', () => {
    expect(avatarUrl('u1', '2025-04-01T12:00:00.000Z')).toBe(
      '/api/v1/users/u1/avatar?v=2025-04-01T12%3A00%3A00.000Z',
    )
  })

  it('URL-encodes the user id (ids can be arbitrary strings)', () => {
    expect(avatarUrl('a/b c')).toBe('/api/v1/users/a%2Fb%20c/avatar')
  })
})

describe('fetchUserAvatar', () => {
  it('fetches the avatar as a Blob, forwarding the bearer token and cache-buster', async () => {
    let captured: Request | null = null
    server.use(
      http.get('/api/v1/users/:id/avatar', ({ request }) => {
        captured = request
        return new HttpResponse(new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer, {
          headers: { 'Content-Type': 'image/png' },
        })
      }),
    )
    const blob = await fetchUserAvatar(TOKEN, 'u1', '2025-04-01T12:00:00Z')
    expect(blob).toBeInstanceOf(Blob)
    expect(blob.size).toBe(4)
    const req = captured as unknown as Request
    expect(req.headers.get('Authorization')).toBe(`Bearer ${TOKEN}`)
    expect(new URL(req.url).searchParams.get('v')).toBe('2025-04-01T12:00:00Z')
  })

  it('throws with the status when the user has no avatar (404)', async () => {
    server.use(
      http.get('/api/v1/users/:id/avatar', () => new HttpResponse(null, { status: 404 })),
    )
    await expect(fetchUserAvatar(TOKEN, 'nobody')).rejects.toMatchObject({ status: 404 })
  })
})
