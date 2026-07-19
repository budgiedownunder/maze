import { describe, it, expect, vi, beforeEach } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { launchTodaysChallenge } from '../../src/utils/dailyChallenge'
import { launchDefinition } from '../../src/utils/play3dLaunch'

vi.mock('../../src/utils/play3dLaunch', () => ({ launchDefinition: vi.fn() }))

const dailyGame = {
  id: 'dg1', ownerId: 'a', name: 'Daily Maze', visibility: 'curated',
  seed: 1, rotation: 'daily', config: {}, createdAt: 'x', updatedAt: 'x',
}
const dailyCollection = {
  id: 'col-daily', ownerId: 'a', name: 'Daily Challenges', visibility: 'curated',
  playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x',
}

function featured(items: unknown[]) {
  return http.get('/api/v1/featured-game-items', () =>
    HttpResponse.json({ items, limit: 20, offset: 0, hasMore: false }))
}
function collectionDetail(definitions: unknown[]) {
  return http.get('/api/v1/game-collections/:id', () =>
    HttpResponse.json({ ...dailyCollection, definitions }))
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('launchTodaysChallenge', () => {
  it('launches the daily member of the Daily Challenges collection', async () => {
    server.use(
      featured([{ kind: 'collection', ownerUsername: 'a', collection: dailyCollection }]),
      collectionDetail([dailyGame]),
    )
    await expect(launchTodaysChallenge('t')).resolves.toBe(true)
    expect(launchDefinition).toHaveBeenCalledWith('dg1')
  })

  it('picks the first member when none is explicitly daily', async () => {
    const staticGame = { ...dailyGame, id: 'sg1', rotation: 'static' }
    server.use(
      featured([{ kind: 'collection', ownerUsername: 'a', collection: dailyCollection }]),
      collectionDetail([staticGame]),
    )
    await expect(launchTodaysChallenge('t')).resolves.toBe(true)
    expect(launchDefinition).toHaveBeenCalledWith('sg1')
  })

  it('returns false when no Daily Challenges collection is featured', async () => {
    server.use(featured([]))
    await expect(launchTodaysChallenge('t')).resolves.toBe(false)
    expect(launchDefinition).not.toHaveBeenCalled()
  })

  it('returns false when the collection has no accessible members', async () => {
    server.use(
      featured([{ kind: 'collection', ownerUsername: 'a', collection: dailyCollection }]),
      collectionDetail([]),
    )
    await expect(launchTodaysChallenge('t')).resolves.toBe(false)
    expect(launchDefinition).not.toHaveBeenCalled()
  })

  it('propagates a network failure to the caller', async () => {
    server.use(http.get('/api/v1/featured-game-items', () => new HttpResponse(null, { status: 500 })))
    await expect(launchTodaysChallenge('t')).rejects.toThrow()
    expect(launchDefinition).not.toHaveBeenCalled()
  })
})
