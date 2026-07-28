import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import {
  createGameDefinition,
  getGameDefinition,
  listGameDefinitions,
  updateGameDefinition,
  reshuffleGameDefinition,
  deleteGameDefinition,
  createGameCollection,
  getGameCollection,
  listGameCollections,
  updateGameCollection,
  deleteGameCollection,
  setGameCollectionItems,
  listGameDefinitionShares,
  setGameDefinitionShares,
  listGameCollectionShares,
  setGameCollectionShares,
  lookupUsers,
} from '../../src/api/client'
import type { GameCollection, GameDefinition, GamePlayResponse } from '../../src/types/api'

const TOKEN = 'test-token'

function def(over: Partial<GameDefinition> = {}): GameDefinition {
  return {
    id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 7,
    rotation: 'static', config: { timerSeconds: 90 },
    createdAt: '2025-04-01T12:00:00Z', updatedAt: '2025-04-01T12:00:00Z', ...over,
  }
}

function coll(over: Partial<GameCollection> = {}): GameCollection {
  return {
    id: 'c1', ownerId: 'o1', name: 'Campaign', visibility: 'private', playMode: 'arcade', items: [],
    createdAt: '2025-04-01T12:00:00Z', updatedAt: '2025-04-01T12:00:00Z', ...over,
  }
}

describe('game-definition client', () => {
  it('createGameDefinition POSTs the body with auth and returns the created definition', async () => {
    let method: string | undefined
    let auth: string | null = null
    let body: unknown
    server.use(
      http.post('/api/v1/game-definitions', async ({ request }) => {
        method = request.method
        auth = request.headers.get('Authorization')
        body = await request.json()
        return HttpResponse.json(def({ id: 'new' }), { status: 201 })
      }),
    )
    const created = await createGameDefinition(TOKEN, {
      name: 'Tower', visibility: 'public', rotation: 'static', config: { timerSeconds: 90 },
    })
    expect(method).toBe('POST')
    expect(auth).toBe(`Bearer ${TOKEN}`)
    expect(body).toEqual({ name: 'Tower', visibility: 'public', rotation: 'static', config: { timerSeconds: 90 } })
    expect(created.id).toBe('new')
  })

  it('getGameDefinition GETs the id-scoped play-fetch and returns challengeKey + tracked flag', async () => {
    let capturedPath: string | null = null
    server.use(
      http.get('/api/v1/game-definitions/:id', ({ request, params }) => {
        capturedPath = new URL(request.url).pathname
        const play: GamePlayResponse = { ...def({ id: String(params.id) }), challengeKey: 'def:d1', leaderboardTracked: true }
        return HttpResponse.json(play)
      }),
    )
    const play = await getGameDefinition(TOKEN, 'd1')
    expect(capturedPath).toBe('/api/v1/game-definitions/d1')
    expect(play.challengeKey).toBe('def:d1')
    expect(play.leaderboardTracked).toBe(true)
  })

  it('listGameDefinitions passes limit/offset and returns the page', async () => {
    let url: string | null = null
    server.use(
      http.get('/api/v1/game-definitions', ({ request }) => {
        url = request.url
        return HttpResponse.json({ definitions: [def()], limit: 5, offset: 10, hasMore: true })
      }),
    )
    const page = await listGameDefinitions(TOKEN, { limit: 5, offset: 10 })
    const params = new URL(url!).searchParams
    expect(params.get('limit')).toBe('5')
    expect(params.get('offset')).toBe('10')
    expect(page.definitions).toHaveLength(1)
    expect(page.hasMore).toBe(true)
  })

  it('listGameDefinitions omits the query string when no paging is given', async () => {
    let search: string | null = null
    server.use(
      http.get('/api/v1/game-definitions', ({ request }) => {
        search = new URL(request.url).search
        return HttpResponse.json({ definitions: [], limit: 20, offset: 0, hasMore: false })
      }),
    )
    await listGameDefinitions(TOKEN)
    expect(search).toBe('')
  })

  it('updateGameDefinition PUTs to the id-scoped path', async () => {
    let method: string | undefined
    let body: unknown
    server.use(
      http.put('/api/v1/game-definitions/:id', async ({ request }) => {
        method = request.method
        body = await request.json()
        return HttpResponse.json(def({ name: 'Renamed' }))
      }),
    )
    const updated = await updateGameDefinition(TOKEN, 'd1', {
      name: 'Renamed', config: {},
    })
    expect(method).toBe('PUT')
    expect(body).toEqual({ name: 'Renamed', config: {} })
    expect(updated.name).toBe('Renamed')
  })

  it('reshuffleGameDefinition POSTs to the reshuffle path and returns the re-minted definition', async () => {
    let method: string | undefined
    let path: string | undefined
    server.use(
      http.post('/api/v1/game-definitions/:id/reshuffle', ({ request }) => {
        method = request.method
        path = new URL(request.url).pathname
        return HttpResponse.json(def({ seed: 999 }))
      }),
    )
    const reshuffled = await reshuffleGameDefinition(TOKEN, 'd1')
    expect(method).toBe('POST')
    expect(path).toBe('/api/v1/game-definitions/d1/reshuffle')
    expect(reshuffled.seed).toBe(999)
  })

  it('deleteGameDefinition resolves on a 200 text response', async () => {
    let method: string | undefined
    server.use(
      http.delete('/api/v1/game-definitions/:id', ({ request }) => {
        method = request.method
        return new HttpResponse("definition 'd1' deleted", { status: 200 })
      }),
    )
    await expect(deleteGameDefinition(TOKEN, 'd1')).resolves.toBeUndefined()
    expect(method).toBe('DELETE')
  })
})

describe('game-collection client', () => {
  it('createGameCollection POSTs the metadata body', async () => {
    let body: unknown
    server.use(
      http.post('/api/v1/game-collections', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json(coll({ id: 'new' }), { status: 201 })
      }),
    )
    const created = await createGameCollection(TOKEN, { name: 'Campaign', visibility: 'public' })
    expect(body).toEqual({ name: 'Campaign', visibility: 'public' })
    expect(created.id).toBe('new')
  })

  it('getGameCollection returns the hydrated detail (member definitions in order)', async () => {
    server.use(
      http.get('/api/v1/game-collections/:id', ({ params }) =>
        HttpResponse.json({
          id: String(params.id), ownerId: 'o1', name: 'Campaign', visibility: 'public',
          createdAt: '2025-04-01T12:00:00Z', updatedAt: '2025-04-01T12:00:00Z',
          definitions: [def({ id: 'a' }), def({ id: 'b' })],
        }),
      ),
    )
    const detail = await getGameCollection(TOKEN, 'c1')
    expect(detail.definitions.map(d => d.id)).toEqual(['a', 'b'])
  })

  it('listGameCollections returns the page', async () => {
    server.use(
      http.get('/api/v1/game-collections', () =>
        HttpResponse.json({ collections: [coll()], limit: 20, offset: 0, hasMore: false }),
      ),
    )
    const page = await listGameCollections(TOKEN)
    expect(page.collections).toHaveLength(1)
  })

  it('updateGameCollection PUTs the metadata body', async () => {
    let body: unknown
    server.use(
      http.put('/api/v1/game-collections/:id', async ({ request }) => {
        body = await request.json()
        return HttpResponse.json(coll({ name: 'Renamed' }))
      }),
    )
    const updated = await updateGameCollection(TOKEN, 'c1', { name: 'Renamed' })
    expect(body).toEqual({ name: 'Renamed' })
    expect(updated.name).toBe('Renamed')
  })

  it('deleteGameCollection resolves on a 200 text response', async () => {
    server.use(
      http.delete('/api/v1/game-collections/:id', () => new HttpResponse("collection 'c1' deleted", { status: 200 })),
    )
    await expect(deleteGameCollection(TOKEN, 'c1')).resolves.toBeUndefined()
  })

  it('setGameCollectionItems PUTs { definitionIds } and returns the updated collection', async () => {
    let path: string | null = null
    let method: string | null = null
    let body: unknown
    server.use(
      http.put('/api/v1/game-collections/:id/items', async ({ request }) => {
        path = new URL(request.url).pathname
        method = request.method
        body = await request.json()
        return HttpResponse.json(coll({ items: [
          { definitionId: 'd3', sortOrder: 0 },
          { definitionId: 'd1', sortOrder: 1 },
        ] }))
      }),
    )
    const updated = await setGameCollectionItems(TOKEN, 'c1', ['d3', 'd1'])
    expect(path).toBe('/api/v1/game-collections/c1/items')
    expect(method).toBe('PUT')
    expect(body).toEqual({ definitionIds: ['d3', 'd1'] })
    expect(updated.items).toEqual([
      { definitionId: 'd3', sortOrder: 0 },
      { definitionId: 'd1', sortOrder: 1 },
    ])
  })
})

describe('share client', () => {
  it('listGameDefinitionShares GETs the shares path and returns the grantees', async () => {
    let path: string | null = null
    let auth: string | null = null
    server.use(
      http.get('/api/v1/game-definitions/:id/shares', ({ request }) => {
        path = new URL(request.url).pathname
        auth = request.headers.get('Authorization')
        return HttpResponse.json({ grantees: [{ id: 'u1', username: 'alice' }, { id: 'u2', username: 'bob' }] })
      }),
    )
    const shares = await listGameDefinitionShares(TOKEN, 'd1')
    expect(path).toBe('/api/v1/game-definitions/d1/shares')
    expect(auth).toBe(`Bearer ${TOKEN}`)
    expect(shares.grantees).toEqual([{ id: 'u1', username: 'alice' }, { id: 'u2', username: 'bob' }])
  })

  it('setGameDefinitionShares PUTs { userIds } and returns the updated grantees', async () => {
    let method: string | undefined
    let path: string | null = null
    let body: unknown
    server.use(
      http.put('/api/v1/game-definitions/:id/shares', async ({ request }) => {
        method = request.method
        path = new URL(request.url).pathname
        body = await request.json()
        return HttpResponse.json({ grantees: [{ id: 'u9', username: 'nine' }] })
      }),
    )
    const shares = await setGameDefinitionShares(TOKEN, 'd1', ['u9', 'u10'])
    expect(method).toBe('PUT')
    expect(path).toBe('/api/v1/game-definitions/d1/shares')
    expect(body).toEqual({ userIds: ['u9', 'u10'] })
    expect(shares.grantees).toEqual([{ id: 'u9', username: 'nine' }])
  })

  it('listGameCollectionShares GETs the collection shares path', async () => {
    let path: string | null = null
    server.use(
      http.get('/api/v1/game-collections/:id/shares', ({ request }) => {
        path = new URL(request.url).pathname
        return HttpResponse.json({ grantees: [{ id: 'u1', username: 'alice' }] })
      }),
    )
    const shares = await listGameCollectionShares(TOKEN, 'c1')
    expect(path).toBe('/api/v1/game-collections/c1/shares')
    expect(shares.grantees).toEqual([{ id: 'u1', username: 'alice' }])
  })

  it('setGameCollectionShares PUTs { userIds } to the collection shares path', async () => {
    let path: string | null = null
    let body: unknown
    server.use(
      http.put('/api/v1/game-collections/:id/shares', async ({ request }) => {
        path = new URL(request.url).pathname
        body = await request.json()
        return HttpResponse.json({ grantees: [] })
      }),
    )
    await setGameCollectionShares(TOKEN, 'c1', ['u9'])
    expect(path).toBe('/api/v1/game-collections/c1/shares')
    expect(body).toEqual({ userIds: ['u9'] })
  })
})

describe('user-lookup client', () => {
  it('lookupUsers passes the username prefix + paging and returns the page (snake_case has_more)', async () => {
    let url: string | null = null
    let auth: string | null = null
    server.use(
      http.get('/api/v1/users/lookup', ({ request }) => {
        url = request.url
        auth = request.headers.get('Authorization')
        return HttpResponse.json({
          users: [{ id: 'u1', username: 'alice' }, { id: 'u2', username: 'alan' }],
          limit: 2, offset: 0, has_more: true,
        })
      }),
    )
    const page = await lookupUsers(TOKEN, { username: 'al', limit: 2, offset: 0 })
    const params = new URL(url!).searchParams
    expect(auth).toBe(`Bearer ${TOKEN}`)
    expect(params.get('username')).toBe('al')
    expect(params.get('limit')).toBe('2')
    expect(params.get('offset')).toBe('0')
    expect(page.users.map(u => u.username)).toEqual(['alice', 'alan'])
    expect(page.has_more).toBe(true)
  })

  it('lookupUsers omits limit/offset when not given but always sends username', async () => {
    let url: string | null = null
    server.use(
      http.get('/api/v1/users/lookup', ({ request }) => {
        url = request.url
        return HttpResponse.json({ users: [], limit: 20, offset: 0, has_more: false })
      }),
    )
    await lookupUsers(TOKEN, { username: 'bob' })
    const params = new URL(url!).searchParams
    expect(params.get('username')).toBe('bob')
    expect(params.has('limit')).toBe(false)
    expect(params.has('offset')).toBe(false)
  })
})
