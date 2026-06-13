import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { getLeaderboard, getScoreHistory } from '../../src/api/client'
import type { ScoreBoardResponse } from '../../src/types/api'

const TOKEN = 'test-token'

const EMPTY_BOARD: ScoreBoardResponse = { scores: [], limit: 20, offset: 0, has_more: false }

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
  })

  it('parses the board response', async () => {
    const board: ScoreBoardResponse = {
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
