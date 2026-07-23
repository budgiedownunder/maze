import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { deleteGameImage, fetchGameImage, gameImageUrl, uploadGameImage } from '../../src/api/client'

const TOKEN = 'test-token'

describe('gameImageUrl', () => {
  it('builds the image path for a definition and a collection', () => {
    expect(gameImageUrl('definition', 'g1')).toBe('/api/v1/game-definitions/g1/image')
    expect(gameImageUrl('collection', 'c1')).toBe('/api/v1/game-collections/c1/image')
  })

  it('appends the marker as a ?v= cache-buster and URL-encodes the id', () => {
    expect(gameImageUrl('definition', 'a/b', '2026-03-01T00:00:00.000Z')).toBe(
      '/api/v1/game-definitions/a%2Fb/image?v=2026-03-01T00%3A00%3A00.000Z',
    )
  })
})

describe('fetchGameImage', () => {
  it('fetches the image as a Blob, forwarding the bearer token + cache-buster', async () => {
    let captured: Request | null = null
    server.use(
      http.get('/api/v1/game-definitions/:id/image', ({ request }) => {
        captured = request
        return new HttpResponse(new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer, { headers: { 'Content-Type': 'image/png' } })
      }),
    )
    const blob = await fetchGameImage(TOKEN, 'definition', 'g1', '2026-03-01T00:00:00Z')
    expect(blob).toBeInstanceOf(Blob)
    expect(blob.size).toBe(4)
    const req = captured as unknown as Request
    expect(req.headers.get('Authorization')).toBe(`Bearer ${TOKEN}`)
    expect(new URL(req.url).searchParams.get('v')).toBe('2026-03-01T00:00:00Z')
  })

  it('throws with the status when there is no image (404)', async () => {
    server.use(http.get('/api/v1/game-collections/:id/image', () => new HttpResponse(null, { status: 404 })))
    await expect(fetchGameImage(TOKEN, 'collection', 'nope')).rejects.toMatchObject({ status: 404 })
  })
})

describe('uploadGameImage', () => {
  it('POSTs multipart to the right entity endpoint and returns the new marker', async () => {
    let captured: Request | null = null
    server.use(
      http.post('/api/v1/game-collections/:id/image', ({ request }) => {
        captured = request
        return HttpResponse.json({ imageUpdatedAt: '2026-03-02T00:00:00.000Z' })
      }),
    )
    const file = new File([new Uint8Array([1, 2, 3])], 'x.png', { type: 'image/png' })
    const res = await uploadGameImage(TOKEN, 'collection', 'c1', file)
    expect(res.imageUpdatedAt).toBe('2026-03-02T00:00:00.000Z')
    const req = captured as unknown as Request
    expect(req.method).toBe('POST')
    expect(new URL(req.url).pathname).toBe('/api/v1/game-collections/c1/image')
    expect(req.headers.get('Authorization')).toBe(`Bearer ${TOKEN}`)
    // Multipart body (the browser sets the boundary) — no forced JSON header.
    expect(req.headers.get('Content-Type')).toMatch(/multipart\/form-data/)
  })
})

describe('deleteGameImage', () => {
  it('sends a DELETE to the entity endpoint with the bearer token', async () => {
    let captured: Request | null = null
    server.use(
      http.delete('/api/v1/game-definitions/:id/image', ({ request }) => {
        captured = request
        return new HttpResponse(null, { status: 204 })
      }),
    )
    await deleteGameImage(TOKEN, 'definition', 'g1')
    const req = captured as unknown as Request
    expect(req.method).toBe('DELETE')
    expect(new URL(req.url).pathname).toBe('/api/v1/game-definitions/g1/image')
    expect(req.headers.get('Authorization')).toBe(`Bearer ${TOKEN}`)
  })
})
