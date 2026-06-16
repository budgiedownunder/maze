import { http, HttpResponse } from 'msw'
import type { AddUserEmailRequest, AppFeatures, LoginResponse, Maze, Play3dConfig, RenewResponse, ScoreboardResponse, ScoreEntry, UpdateProfileRequest, UserEmail, UserEmailsResponse, UserProfile } from '../types/api'

const BASE = '/api/v1'

export const mockProfile: UserProfile = {
  id: '00000000-0000-0000-0000-000000000001',
  username: 'testuser',
  full_name: 'Test User',
  email: 'test@example.com',
  emails: [
    { email: 'test@example.com', is_primary: true, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
  ],
  is_admin: false,
  has_password: true,
}

export let mockEmails: UserEmail[] = [
  { email: 'test@example.com', is_primary: true, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
]

export function resetMockEmails(): void {
  mockEmails = [
    { email: 'test@example.com', is_primary: true, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
  ]
}

export const mockLoginResponse: LoginResponse = {
  login_token_id: 'aaaaaaaa-0000-0000-0000-000000000001',
  login_token_expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
  // Default to false so the seeded "test" user (signed in by every test
  // helper) doesn't take the welcome-banner branch. Tests that exercise
  // the first-sign-in path override this with `server.use(...)`.
  is_first_sign_in: false,
}

export const mockMazeAlpha: Maze = {
  id: 'maze-0001',
  name: 'Alpha',
  definition: {
    grid: [
      ['S', ' ', ' '],
      [' ', 'W', ' '],
      [' ', ' ', 'F'],
    ],
  },
}

export const mockMazeBeta: Maze = {
  id: 'maze-0002',
  name: 'Beta',
  definition: {
    grid: [
      ['S', ' ', ' ', ' ', ' '],
      [' ', 'W', ' ', 'W', ' '],
      [' ', ' ', ' ', ' ', ' '],
      [' ', 'W', ' ', 'W', ' '],
      [' ', ' ', ' ', ' ', 'F'],
    ],
  },
}

// A keys-&-doors maze for game play. Kept out of `mockMazes` (and thus the maze
// list) so list-count assertions stay at two; the GET /mazes/:id handler serves it
// directly by id for deep-linked /play/maze-keydoor.
export const mockMazeKeyDoor: Maze = {
  id: 'maze-keydoor',
  name: 'KeyDoor',
  definition: {
    grid: [
      ['S', 'K', 'D', 'F'],
    ],
  },
}

// An enemies-only maze for game play. Three enemies in a row let the e2e walk
// the player into successive collisions, verifying HP decrement, damage flash,
// and the death popup on the final hit. Health pickups intentionally absent
// so each collision deterministically loses one HP.
export const mockMazeEnemyGauntlet: Maze = {
  id: 'maze-enemy-gauntlet',
  name: 'EnemyGauntlet',
  definition: {
    grid: [
      ['S', 'E', 'E', 'E', 'F'],
    ],
  },
}

// An enemy followed by a health pickup: collide with the enemy to drop below
// max HP, then walk onto the pickup to heal and consume it. Used to assert that
// a consumed pickup's in-grid symbol disappears.
export const mockMazeEnemyHealth: Maze = {
  id: 'maze-enemy-health',
  name: 'EnemyHealth',
  definition: {
    grid: [
      ['S', 'E', 'H', 'F'],
    ],
  },
}

// A maze persisted WITH a per-cell override: the enemy cell at (0,1) carries the
// canonical array form. Used to verify the editor loads + renders a saved override.
// Kept out of `mockMazes`, served by id directly. The array-form cell isn't expressible
// in the simplified `string[][]` client type, hence the cast.
export const mockMazeOverride: Maze = {
  id: 'maze-override',
  name: 'Override',
  definition: {
    grid: [['S', [{ type: 'E', enemyType: 'ghost', damage: 2 }], 'F']],
  } as unknown as Maze['definition'],
}

// A maze whose STATIC feature cells (health/key/door) carry overrides, in the
// canonical array form. Used to verify the 2D game renders the static variant sprites
// (the regression where MazeGamePage handed the raw array-form grid to MazeGrid showed
// these cells empty). Layout: S, potion-health, pedestal-key, swing-door, F.
export const mockMazeOverrideStatic: Maze = {
  id: 'maze-override-static',
  name: 'OverrideStatic',
  definition: {
    grid: [[
      'S',
      [{ type: 'H', healthStyle: 'potion' }],
      [{ type: 'K', keyHolder: 'pedestal' }],
      [{ type: 'D', doorStyle: 'swing' }],
      'F',
    ]],
  } as unknown as Maze['definition'],
}

// A maze persisted WITH per-maze 3D game settings. Used to verify the editor
// seeds the settings modal from the maze's saved settings (not localStorage).
// Kept out of `mockMazes`, served by id directly.
export const mockMazeWithSettings: Maze = {
  id: 'maze-settings',
  name: 'WithSettings',
  definition: { grid: [['S', ' ', 'F']] },
  game_settings: {
    skyType: 'day',
    wallType: 'wood',
    perimeterWalls: true,
    doorStyle: 'swing',
    keyHolder: 'pedestal',
    enemyType: 'goblin',
    healthStyle: 'heart',
    wallTint: false,
    wallMaterialVariation: false,
    deadEndObjects: true,
    wallDecorations: true,
    floorAccents: true,
    timerSeconds: 222,
  },
}

// A maze whose per-maze game settings make ghost the default enemy and lava the
// default wall, with NO per-cell overrides. Used to verify the 2D game renders those
// maze-default bases (the 'E' enemy as a ghost, the 'W' wall as lava). Kept out of
// `mockMazes`, served by id directly. Layout: an enemy on the S→F path and a wall
// off-path.
export const mockMazeSettingsDisplay: Maze = {
  id: 'maze-settings-display',
  name: 'SettingsDisplay',
  definition: {
    grid: [
      ['S', 'E', 'F'],
      ['W', ' ', ' '],
    ],
  },
  game_settings: {
    skyType: 'night',
    wallType: 'lava',
    perimeterWalls: true,
    doorStyle: 'swing',
    keyHolder: 'pedestal',
    enemyType: 'ghost',
    healthStyle: 'heart',
    wallTint: false,
    wallMaterialVariation: false,
    deadEndObjects: true,
    wallDecorations: true,
    floorAccents: true,
    timerSeconds: 60,
  },
}

export let mockMazes: Maze[] = [mockMazeAlpha, mockMazeBeta]

export function resetMockMazes(): void {
  mockMazes = [mockMazeAlpha, mockMazeBeta]
}

// In-memory mock token stores (token → target email). Confirm endpoints look
// up the token, mutate the matching mock state, and delete the entry on use.
//
// Mirrored into sessionStorage so the e2e specs survive `page.goto`
// navigations — without it, every reload re-imports this module and wipes
// the Maps before the confirm endpoint can validate the token. jsdom and
// real browsers both have sessionStorage; the typeof guard keeps the
// module portable to environments that don't.
const RESET_TOKEN_STORAGE_KEY = '__msw_mock_reset_tokens'
const VERIFICATION_TOKEN_STORAGE_KEY = '__msw_mock_verification_tokens'

function loadTokenMap(key: string): Map<string, string> {
  if (typeof sessionStorage === 'undefined') return new Map()
  try {
    const raw = sessionStorage.getItem(key)
    if (!raw) return new Map()
    return new Map(JSON.parse(raw) as [string, string][])
  } catch {
    return new Map()
  }
}

function saveTokenMap(key: string, map: Map<string, string>): void {
  if (typeof sessionStorage === 'undefined') return
  try {
    sessionStorage.setItem(key, JSON.stringify([...map.entries()]))
  } catch { /* ignore quota / serialization errors */ }
}

export const mockResetTokens = loadTokenMap(RESET_TOKEN_STORAGE_KEY)
export const mockVerificationTokens = loadTokenMap(VERIFICATION_TOKEN_STORAGE_KEY)

export function resetMockTokens(): void {
  mockResetTokens.clear()
  mockVerificationTokens.clear()
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.removeItem(RESET_TOKEN_STORAGE_KEY)
    sessionStorage.removeItem(VERIFICATION_TOKEN_STORAGE_KEY)
  }
}

let tokenCounter = 0
function mintToken(prefix: string): string {
  tokenCounter += 1
  return `${prefix}-${tokenCounter}-${Date.now()}`
}

export const handlers = [
  http.get(`${BASE}/features`, () => {
    return HttpResponse.json<AppFeatures>({ allow_signup: true, oauth_providers: [], email_enabled: true, max_maze_cells: null })
  }),

  http.put(`${BASE}/admin/features`, async ({ request }) => {
    const body = await request.json() as AppFeatures
    return HttpResponse.json<AppFeatures>(body)
  }),

  http.post(`${BASE}/login`, () => {
    return HttpResponse.json(mockLoginResponse)
  }),

  http.post(`${BASE}/logout`, () => {
    return new HttpResponse(null, { status: 204 })
  }),

  http.post(`${BASE}/login/renew`, () => {
    const renewed: RenewResponse = {
      login_token_id: 'bbbbbbbb-0000-0000-0000-000000000001',
      login_token_expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }
    return HttpResponse.json(renewed)
  }),

  http.post(`${BASE}/signup`, () => {
    return new HttpResponse(null, { status: 200 })
  }),

  http.get(`${BASE}/users/me`, () => {
    return HttpResponse.json(mockProfile)
  }),

  http.get(`${BASE}/users/:id/avatar`, () => {
    return new HttpResponse(null, { status: 404 })
  }),

  http.put(`${BASE}/users/me/profile`, async ({ request }) => {
    const body = await request.json() as UpdateProfileRequest
    return HttpResponse.json({ ...mockProfile, ...body })
  }),

  http.put(`${BASE}/users/me/password`, () => {
    return new HttpResponse(null, { status: 200 })
  }),

  http.delete(`${BASE}/users/me`, () => {
    return new HttpResponse(null, { status: 204 })
  }),

  http.get(`${BASE}/users/me/emails`, () => {
    return HttpResponse.json<UserEmailsResponse>({ emails: mockEmails })
  }),

  http.post(`${BASE}/users/me/emails`, async ({ request }) => {
    const body = await request.json() as AddUserEmailRequest
    if (mockEmails.some(e => e.email.toLowerCase() === body.email.toLowerCase())) {
      return new HttpResponse('Email is already taken', { status: 409 })
    }
    mockEmails = [
      ...mockEmails,
      { email: body.email, is_primary: false, verified: true, verified_at: new Date().toISOString() },
    ]
    return HttpResponse.json<UserEmailsResponse>({ emails: mockEmails }, { status: 201 })
  }),

  http.delete(`${BASE}/users/me/emails/:email`, ({ params }) => {
    const target = decodeURIComponent(params.email as string)
    const row = mockEmails.find(e => e.email === target)
    if (!row) return new HttpResponse(null, { status: 404 })
    if (mockEmails.length === 1) return new HttpResponse('Cannot remove last email', { status: 409 })
    if (row.is_primary) return new HttpResponse('Cannot remove primary email', { status: 409 })
    mockEmails = mockEmails.filter(e => e.email !== target)
    return HttpResponse.json<UserEmailsResponse>({ emails: mockEmails })
  }),

  http.put(`${BASE}/users/me/emails/:email/primary`, ({ params }) => {
    const target = decodeURIComponent(params.email as string)
    const row = mockEmails.find(e => e.email === target)
    if (!row) return new HttpResponse(null, { status: 404 })
    if (!row.verified) return new HttpResponse('Cannot promote unverified email', { status: 409 })
    mockEmails = mockEmails.map(e => ({ ...e, is_primary: e.email === target }))
    return HttpResponse.json<UserEmailsResponse>({ emails: mockEmails })
  }),

  http.post(`${BASE}/password-reset/request`, async ({ request }) => {
    const body = await request.json() as { email: string }
    const known = body.email.toLowerCase() === mockProfile.email.toLowerCase()
      || mockEmails.some(e => e.email.toLowerCase() === body.email.toLowerCase())
    if (known) {
      mockResetTokens.set(mintToken('reset'), body.email)
      saveTokenMap(RESET_TOKEN_STORAGE_KEY, mockResetTokens)
    }
    return new HttpResponse(null, { status: 200 })
  }),

  http.post(`${BASE}/password-reset/confirm`, async ({ request }) => {
    const body = await request.json() as { token: string; new_password: string }
    if (!mockResetTokens.has(body.token)) {
      return new HttpResponse('Invalid or expired token', { status: 400 })
    }
    mockResetTokens.delete(body.token)
    saveTokenMap(RESET_TOKEN_STORAGE_KEY, mockResetTokens)
    return new HttpResponse(null, { status: 200 })
  }),

  http.post(`${BASE}/email-verifications/request`, async ({ request }) => {
    const body = await request.json() as { email: string }
    const onUser = mockEmails.some(e => e.email.toLowerCase() === body.email.toLowerCase())
    if (onUser) {
      mockVerificationTokens.set(mintToken('verify'), body.email)
      saveTokenMap(VERIFICATION_TOKEN_STORAGE_KEY, mockVerificationTokens)
    }
    return new HttpResponse(null, { status: 200 })
  }),

  http.post(`${BASE}/email-verifications/confirm`, async ({ request }) => {
    const body = await request.json() as { token: string }
    const target = mockVerificationTokens.get(body.token)
    if (!target) return new HttpResponse('Invalid or expired token', { status: 400 })
    mockVerificationTokens.delete(body.token)
    saveTokenMap(VERIFICATION_TOKEN_STORAGE_KEY, mockVerificationTokens)
    mockEmails = mockEmails.map(e =>
      e.email.toLowerCase() === target.toLowerCase()
        ? { ...e, verified: true, verified_at: new Date().toISOString() }
        : e,
    )
    return new HttpResponse(null, { status: 200 })
  }),

  http.get(`${BASE}/mazes`, ({ request }) => {
    const url = new URL(request.url)
    const includeDefinitions = url.searchParams.get('includeDefinitions') === 'true'
    return HttpResponse.json(mockMazes.map(maze => ({
      id: maze.id,
      name: maze.name,
      definition: includeDefinitions ? JSON.stringify(maze) : null,
    })))
  }),

  http.get(`${BASE}/mazes/:id`, ({ params }) => {
    const maze = mockMazes.find(m => m.id === params.id)
      ?? (params.id === mockMazeKeyDoor.id ? mockMazeKeyDoor : undefined)
      ?? (params.id === mockMazeEnemyGauntlet.id ? mockMazeEnemyGauntlet : undefined)
      ?? (params.id === mockMazeEnemyHealth.id ? mockMazeEnemyHealth : undefined)
      ?? (params.id === mockMazeOverride.id ? mockMazeOverride : undefined)
      ?? (params.id === mockMazeOverrideStatic.id ? mockMazeOverrideStatic : undefined)
      ?? (params.id === mockMazeWithSettings.id ? mockMazeWithSettings : undefined)
      ?? (params.id === mockMazeSettingsDisplay.id ? mockMazeSettingsDisplay : undefined)
    if (!maze) return new HttpResponse(null, { status: 404 })
    return HttpResponse.json(maze)
  }),

  http.post(`${BASE}/mazes`, async ({ request }) => {
    const body = await request.json() as Maze
    const isDuplicate = mockMazes.some(m => m.name.toLowerCase() === body.name.toLowerCase())
    if (isDuplicate) return new HttpResponse('A maze with that name already exists.', { status: 409 })
    const created: Maze = { ...body, id: `maze-${Date.now()}` }
    mockMazes = [...mockMazes, created]
    return HttpResponse.json(created, { status: 201 })
  }),

  http.put(`${BASE}/mazes/:id`, async ({ params, request }) => {
    const body = await request.json() as Maze
    const index = mockMazes.findIndex(m => m.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    const isDuplicate = mockMazes.some(m => m.id !== params.id && m.name.toLowerCase() === body.name.toLowerCase())
    if (isDuplicate) return new HttpResponse('A maze with that name already exists.', { status: 409 })
    mockMazes = mockMazes.map((m, i) => i === index ? { ...m, ...body } : m)
    return HttpResponse.json(mockMazes[index])
  }),

  http.delete(`${BASE}/mazes/:id`, ({ params }) => {
    const exists = mockMazes.some(m => m.id === params.id)
    if (!exists) return new HttpResponse(null, { status: 404 })
    mockMazes = mockMazes.filter(m => m.id !== params.id)
    return new HttpResponse(null, { status: 200 })
  }),

  // Scores — curated preset (for the leaderboard seed), personal history, and
  // the leaderboard itself.
  http.get(`${BASE}/game/play3d-config`, ({ request }) => {
    const difficulty = new URL(request.url).searchParams.get('difficulty') ?? 'easy'
    const seeds: Record<string, number> = { easy: 111, tricky: 222, hard: 333 }
    return HttpResponse.json<Play3dConfig>({ difficulty, seed: seeds[difficulty] ?? 999 })
  }),

  http.get(`${BASE}/scores/me`, () => {
    // Most recent first — the page picks scores[0] as the default subject.
    const scores: ScoreEntry[] = [
      { id: 'sh1', user_id: mockProfile.id, maze_id: 'maze-0001', challenge: null, score: 7, elapsed_ms: 42137, recorded_at: '2025-04-02T10:00:00.000Z' },
      { id: 'sh2', user_id: mockProfile.id, maze_id: null, challenge: 'easy:111', score: 5, elapsed_ms: 51020, recorded_at: '2025-04-01T10:00:00.000Z' },
    ]
    return HttpResponse.json<ScoreboardResponse>({ scores, limit: 100, offset: 0, has_more: false })
  }),

  http.get(`${BASE}/scores`, ({ request }) => {
    const url = new URL(request.url)
    const mazeId = url.searchParams.get('maze_id')
    const challenge = url.searchParams.get('challenge')
    const withNames = url.searchParams.get('include_usernames') !== 'false'
    const subject = { maze_id: mazeId, challenge }
    const scores: ScoreEntry[] = [
      { id: 'lb1', user_id: 'other-1', ...subject, score: 9, elapsed_ms: 31204, recorded_at: '2025-04-02T09:00:00.000Z', username: withNames ? 'alice' : undefined },
      { id: 'lb2', user_id: mockProfile.id, ...subject, score: 7, elapsed_ms: 42137, recorded_at: '2025-04-02T10:00:00.000Z', username: withNames ? mockProfile.username : undefined },
    ]
    return HttpResponse.json<ScoreboardResponse>({ scores, limit: 20, offset: 0, has_more: false })
  }),
]
