import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { getPlay3dConfig } from '../../src/api/client'

describe('getPlay3dConfig', () => {
  it('fetches the preset for a difficulty and returns its seed', async () => {
    let capturedUrl: string | null = null
    server.use(
      http.get('/api/v1/game/play3d-config', ({ request }) => {
        capturedUrl = request.url
        return HttpResponse.json({ difficulty: 'easy', seed: 12345 })
      }),
    )
    const config = await getPlay3dConfig('easy')
    expect(new URL(capturedUrl!).searchParams.get('difficulty')).toBe('easy')
    expect(config).toEqual({ difficulty: 'easy', seed: 12345 })
  })
})
