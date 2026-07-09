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
  addCollectionItem,
  removeCollectionItem,
  reorderCollectionItems,
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
    id: 'c1', ownerId: 'o1', name: 'Campaign', visibility: 'private', items: [],
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

  it('addCollectionItem POSTs { definitionId } and returns the updated collection', async () => {
    let path: string | null = null
    let body: unknown
    server.use(
      http.post('/api/v1/game-collections/:id/items', async ({ request }) => {
        path = new URL(request.url).pathname
        body = await request.json()
        return HttpResponse.json(coll({ items: [{ definitionId: 'd9', sortOrder: 0 }] }))
      }),
    )
    const updated = await addCollectionItem(TOKEN, 'c1', 'd9')
    expect(path).toBe('/api/v1/game-collections/c1/items')
    expect(body).toEqual({ definitionId: 'd9' })
    expect(updated.items).toEqual([{ definitionId: 'd9', sortOrder: 0 }])
  })

  it('removeCollectionItem DELETEs the nested item path', async () => {
    let path: string | null = null
    server.use(
      http.delete('/api/v1/game-collections/:id/items/:definitionId', ({ request }) => {
        path = new URL(request.url).pathname
        return HttpResponse.json(coll())
      }),
    )
    await removeCollectionItem(TOKEN, 'c1', 'd9')
    expect(path).toBe('/api/v1/game-collections/c1/items/d9')
  })

  it('reorderCollectionItems PUTs { ordered } to the reorder path', async () => {
    let path: string | null = null
    let body: unknown
    server.use(
      http.put('/api/v1/game-collections/:id/items/reorder', async ({ request }) => {
        path = new URL(request.url).pathname
        body = await request.json()
        return HttpResponse.json(coll())
      }),
    )
    await reorderCollectionItems(TOKEN, 'c1', ['d3', 'd1', 'd2'])
    expect(path).toBe('/api/v1/game-collections/c1/items/reorder')
    expect(body).toEqual({ ordered: ['d3', 'd1', 'd2'] })
  })
})
