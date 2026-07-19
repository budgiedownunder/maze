import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { getCompletedChallenges, getLeaderboard, getScoreHistory, resetLeaderboard } from '../../src/api/client'
import { gameChallengeKey, todayUtc } from '../../src/utils/gameDefinitions'
import type { ScoreboardResponse } from '../../src/types/api'

const TOKEN = 'test-token'

const EMPTY_BOARD: ScoreboardResponse = { scores: [], limit: 20, offset: 0, has_more: false }

// Captures the request the client makes so the test can assert on the assembled
// query string + headers, then returns a canned board.
function captureBoard(path: string): { url: () => URL; auth: () => string | null } {
  let captured: Request | null = null
  server.use(
    http.get(`/api/v1${path}`, ({ request }) => {
      captured = request
      return HttpResponse.json(EMPTY_BOARD)
    }),
  )
  return {
    url: () => new URL((captured as unknown as Request).url),
    auth: () => (captured as unknown as Request).headers.get('Authorization'),
  }
}

describe('getLeaderboard', () => {
  it('sends the maze_id subject and forwards the bearer token', async () => {
    const cap = captureBoard('/scores')
    await getLeaderboard(TOKEN, { mazeId: 'My Maze.json' })
    const url = cap.url()
    expect(url.pathname).toBe('/api/v1/scores')
    expect(url.searchParams.get('maze_id')).toBe('My Maze.json')
    expect(url.searchParams.get('challenge')).toBeNull()
    expect(cap.auth()).toBe(`Bearer ${TOKEN}`)
  })

  it('sends the challenge subject with metric/direction/paging', async () => {
    const cap = captureBoard('/scores')
    await getLeaderboard(TOKEN, {
      challenge: 'hard:12345',
      metric: 'score',
      direction: 'desc',
      limit: 10,
      offset: 20,
    })
    const url = cap.url()
    expect(url.searchParams.get('challenge')).toBe('hard:12345')
    expect(url.searchParams.get('maze_id')).toBeNull()
    expect(url.searchParams.get('metric')).toBe('score')
    expect(url.searchParams.get('direction')).toBe('desc')
    expect(url.searchParams.get('limit')).toBe('10')
    expect(url.searchParams.get('offset')).toBe('20')
  })

  it('omits unset optional params', async () => {
    const cap = captureBoard('/scores')
    await getLeaderboard(TOKEN, { mazeId: 'maze-1' })
    const url = cap.url()
    expect(url.searchParams.has('metric')).toBe(false)
    expect(url.searchParams.has('direction')).toBe(false)
    expect(url.searchParams.has('limit')).toBe(false)
    expect(url.searchParams.has('offset')).toBe(false)
    expect(url.searchParams.has('include_usernames')).toBe(false)
  })

  it('forwards includeUsernames as the include_usernames param', async () => {
    const capTrue = captureBoard('/scores')
    await getLeaderboard(TOKEN, { challenge: 'easy:1', includeUsernames: true })
    expect(capTrue.url().searchParams.get('include_usernames')).toBe('true')

    const capFalse = captureBoard('/scores')
    await getLeaderboard(TOKEN, { mazeId: 'maze-1', includeUsernames: false })
    expect(capFalse.url().searchParams.get('include_usernames')).toBe('false')
  })

  it('parses a row username off the board response', async () => {
    const board: ScoreboardResponse = {
      scores: [
        {
          id: 'a',
          user_id: 'u',
          maze_id: null,
          challenge: 'easy:1',
          score: 5,
          elapsed_ms: 1000,
          recorded_at: '2025-04-01T12:00:00Z',
          username: 'alice',
        },
      ],
      limit: 20,
      offset: 0,
      has_more: false,
    }
    server.use(http.get('/api/v1/scores', () => HttpResponse.json(board)))
    const result = await getLeaderboard(TOKEN, { challenge: 'easy:1', includeUsernames: true })
    expect(result.scores[0].username).toBe('alice')
  })

  it('parses the board response', async () => {
    const board: ScoreboardResponse = {
      scores: [
        {
          id: 'a',
          user_id: 'u',
          maze_id: 'maze-1',
          challenge: null,
          score: 5,
          elapsed_ms: 1000,
          recorded_at: '2025-04-01T12:00:00Z',
        },
      ],
      limit: 20,
      offset: 0,
      has_more: true,
    }
    server.use(http.get('/api/v1/scores', () => HttpResponse.json(board)))
    const result = await getLeaderboard(TOKEN, { mazeId: 'maze-1' })
    expect(result).toEqual(board)
  })

  it('throws when neither subject is set (fail-fast before any request)', () => {
    expect(() => getLeaderboard(TOKEN, {})).toThrow(/exactly one/)
  })

  it('throws when both subjects are set', () => {
    expect(() => getLeaderboard(TOKEN, { mazeId: 'm', challenge: 'c' })).toThrow(/exactly one/)
  })
})

describe('resetLeaderboard', () => {
  it('sends a DELETE with the maze_id subject + bearer token and returns the count', async () => {
    let captured: Request | null = null
    server.use(
      http.delete('/api/v1/scores', ({ request }) => {
        captured = request
        return HttpResponse.json({ deleted: 3 })
      }),
    )
    const res = await resetLeaderboard(TOKEN, { mazeId: 'My Maze.json' })
    expect(res.deleted).toBe(3)
    const req = captured as unknown as Request
    const url = new URL(req.url)
    expect(req.method).toBe('DELETE')
    expect(url.pathname).toBe('/api/v1/scores')
    expect(url.searchParams.get('maze_id')).toBe('My Maze.json')
    expect(url.searchParams.get('challenge')).toBeNull()
    expect(req.headers.get('Authorization')).toBe(`Bearer ${TOKEN}`)
  })

  it('sends the challenge subject', async () => {
    let captured: Request | null = null
    server.use(
      http.delete('/api/v1/scores', ({ request }) => {
        captured = request
        return HttpResponse.json({ deleted: 0 })
      }),
    )
    await resetLeaderboard(TOKEN, { challenge: 'hard:1' })
    const url = new URL((captured as unknown as Request).url)
    expect(url.searchParams.get('challenge')).toBe('hard:1')
    expect(url.searchParams.get('maze_id')).toBeNull()
  })

  it('throws when neither / both subjects are set (fail-fast before any request)', () => {
    expect(() => resetLeaderboard(TOKEN, {})).toThrow(/exactly one/)
    expect(() => resetLeaderboard(TOKEN, { mazeId: 'm', challenge: 'c' })).toThrow(/exactly one/)
  })
})

describe('getScoreHistory', () => {
  it('hits /scores/me with the bearer token and no params by default', async () => {
    const cap = captureBoard('/scores/me')
    await getScoreHistory(TOKEN)
    const url = cap.url()
    expect(url.pathname).toBe('/api/v1/scores/me')
    expect([...url.searchParams.keys()]).toHaveLength(0)
    expect(cap.auth()).toBe(`Bearer ${TOKEN}`)
  })

  it('forwards limit + offset when given', async () => {
    const cap = captureBoard('/scores/me')
    await getScoreHistory(TOKEN, { limit: 5, offset: 10 })
    const url = cap.url()
    expect(url.searchParams.get('limit')).toBe('5')
    expect(url.searchParams.get('offset')).toBe('10')
  })
})

describe('getCompletedChallenges', () => {
  it('POSTs the challenges and returns the completed subset', async () => {
    let body: unknown = null
    let auth: string | null = null
    server.use(
      http.post('/api/v1/scores/me/completed', async ({ request }) => {
        body = await request.json()
        auth = request.headers.get('Authorization')
        return HttpResponse.json({ completed: ['def:a'] })
      }),
    )
    const res = await getCompletedChallenges(TOKEN, ['def:a', 'def:b'])
    expect(res.completed).toEqual(['def:a'])
    expect(body).toEqual({ challenges: ['def:a', 'def:b'] })
    expect(auth).toContain(TOKEN)
  })
})

describe('gameChallengeKey', () => {
  it('is def:<id> for a Static game (the default)', () => {
    expect(gameChallengeKey('abc')).toBe('def:abc')
    expect(gameChallengeKey('abc', 'static')).toBe('def:abc')
    // The date is ignored for a Static game.
    expect(gameChallengeKey('abc', 'static', '2026-07-14')).toBe('def:abc')
  })

  it('is def:<id>:<date> for a Daily game, defaulting to today (UTC)', () => {
    expect(gameChallengeKey('abc', 'daily', '2026-07-14')).toBe('def:abc:2026-07-14')
    expect(gameChallengeKey('abc', 'daily')).toBe(`def:abc:${todayUtc()}`)
  })
})

describe('todayUtc', () => {
  it('is a yyyy-mm-dd date matching the current UTC day', () => {
    expect(todayUtc()).toMatch(/^\d{4}-\d{2}-\d{2}$/)
    expect(todayUtc()).toBe(new Date().toISOString().slice(0, 10))
  })
})
