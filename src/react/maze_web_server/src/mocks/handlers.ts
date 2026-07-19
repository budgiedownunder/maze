import { http, HttpResponse } from 'msw'
import type { AddUserEmailRequest, AppFeatures, BoardDatesResponse, CompletedChallengesResponse, FeaturedGameItem, FeaturedGameItemEntry, FeaturedGameItemsListResponse, GameCollection, GameCollectionDetailResponse, GameCollectionListResponse, GameCollectionRequest, GameDefinition, GameDefinitionListResponse, GameDefinitionRequest, GamePlayResponse, GranteeSummary, LoginResponse, Maze, Play3dConfig, RenewResponse, ScoreboardResponse, ScoreEntry, UpdateProfileRequest, UserEmail, UserEmailsResponse, UserLookupEntry, UserLookupResponse, UserProfile } from '../types/api'

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

// Dev-mock admin toggle: signing in with this email makes the mock user an admin
// (so the admin-only Manage Features surface is reachable in dev:mock / e2e). Any
// other email signs in as the default non-admin `mockProfile`. Persisted in
// sessionStorage so the flag survives the post-login navigation + `/users/me`
// refetch. Unit tests mock `useAuth` directly and never touch this.
const ADMIN_LOGIN_EMAIL = 'admin@example.com'
const ADMIN_FLAG_KEY = 'mock-signed-in-is-admin'

function setSignedInAdmin(isAdmin: boolean): void {
  if (typeof sessionStorage === 'undefined') return
  try { sessionStorage.setItem(ADMIN_FLAG_KEY, isAdmin ? '1' : '0') } catch { /* ignore */ }
}

function signedInIsAdmin(): boolean {
  if (typeof sessionStorage === 'undefined') return false
  try { return sessionStorage.getItem(ADMIN_FLAG_KEY) === '1' } catch { return false }
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

// A single treasure on the S→F path: walk onto it to auto-collect, verifying the
// in-grid treasure sprite disappears (rendered from the runtime's live treasure
// list, not the static grid char) and the collected style shows in the bag. Kept
// out of `mockMazes`, served by id directly.
export const mockMazeTreasure: Maze = {
  id: 'maze-treasure',
  name: 'Treasure',
  definition: {
    grid: [
      ['S', 'T', 'F'],
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

// In-memory mock game definitions, mirrored into sessionStorage for the same
// reason the token maps are: an e2e reload re-imports this module, and a created
// definition has to survive that to be seen in the reloaded list.
const GAME_DEFINITION_STORAGE_KEY = '__msw_mock_game_definitions'

function loadGameDefinitions(): GameDefinition[] {
  if (typeof sessionStorage === 'undefined') return []
  try {
    const raw = sessionStorage.getItem(GAME_DEFINITION_STORAGE_KEY)
    return raw ? (JSON.parse(raw) as GameDefinition[]) : []
  } catch {
    return []
  }
}

function saveGameDefinitions(): void {
  if (typeof sessionStorage === 'undefined') return
  try {
    sessionStorage.setItem(GAME_DEFINITION_STORAGE_KEY, JSON.stringify(mockGameDefinitions))
  } catch { /* ignore quota / serialization errors */ }
}

export let mockGameDefinitions: GameDefinition[] = loadGameDefinitions()

export function resetMockGameDefinitions(): void {
  mockGameDefinitions = []
  if (typeof sessionStorage !== 'undefined') sessionStorage.removeItem(GAME_DEFINITION_STORAGE_KEY)
}

// Mock game collections — same sessionStorage-mirrored store as definitions, so a
// created collection survives an e2e reload.
const GAME_COLLECTION_STORAGE_KEY = '__msw_mock_game_collections'

function loadGameCollections(): GameCollection[] {
  if (typeof sessionStorage === 'undefined') return []
  try {
    const raw = sessionStorage.getItem(GAME_COLLECTION_STORAGE_KEY)
    return raw ? (JSON.parse(raw) as GameCollection[]) : []
  } catch {
    return []
  }
}

function saveGameCollections(): void {
  if (typeof sessionStorage === 'undefined') return
  try {
    sessionStorage.setItem(GAME_COLLECTION_STORAGE_KEY, JSON.stringify(mockGameCollections))
  } catch { /* ignore quota / serialization errors */ }
}

export let mockGameCollections: GameCollection[] = loadGameCollections()

export function resetMockGameCollections(): void {
  mockGameCollections = []
  if (typeof sessionStorage !== 'undefined') sessionStorage.removeItem(GAME_COLLECTION_STORAGE_KEY)
}

// The admin-ordered featured catalogue — a list of `(kind, id)` references, the
// index carrying the order (mirrors the server's `featured_game_items`). GET
// hydrates each entry from the mock defs/collections and filters to those still
// curated (the server's projection of the curated tier); PUT /order rewrites it.
export let mockFeaturedGameItems: FeaturedGameItemEntry[] = []

export function resetMockFeaturedGameItems(): void {
  mockFeaturedGameItems = []
}

// The featured catalogue is a projection of the curated tier: every curated
// definition + collection, ordered by `mockFeaturedGameItems` (the admin order),
// with any not-yet-ordered curated item appended. Deriving it from the persisted
// def/collection stores (rather than a standalone list) means a newly-curated
// item auto-appears, an un-curated one drops, and the set survives a reload —
// mirroring the server's storage-side reconcile without tracking it by hand.
function hydrateFeaturedItems(): FeaturedGameItem[] {
  const curatedDefs = mockGameDefinitions.filter(d => d.visibility === 'curated')
  const curatedCols = mockGameCollections.filter(c => c.visibility === 'curated')
  // Resolve an owner id to a username (server-side in production); the signed-in
  // mock user, else the searchable directory, else "unknown".
  const ownerName = (ownerId: string): string =>
    ownerId === mockProfile.id ? mockProfile.username : (mockUserDirectory.find(u => u.id === ownerId)?.username ?? 'unknown')
  const out: FeaturedGameItem[] = []
  const seen = new Set<string>()
  const pushDef = (definition: GameDefinition) => { if (!seen.has(`definition:${definition.id}`)) { seen.add(`definition:${definition.id}`); out.push({ kind: 'definition', ownerUsername: ownerName(definition.ownerId), definition }) } }
  const pushCol = (collection: GameCollection) => { if (!seen.has(`collection:${collection.id}`)) { seen.add(`collection:${collection.id}`); out.push({ kind: 'collection', ownerUsername: ownerName(collection.ownerId), collection }) } }
  // Known order first…
  for (const entry of mockFeaturedGameItems) {
    if (entry.kind === 'definition') {
      const d = curatedDefs.find(x => x.id === entry.id)
      if (d) pushDef(d)
    } else {
      const c = curatedCols.find(x => x.id === entry.id)
      if (c) pushCol(c)
    }
  }
  // …then any curated item not yet ordered (newly featured).
  curatedDefs.forEach(pushDef)
  curatedCols.forEach(pushCol)
  return out
}

// The searchable user directory the share people-picker's username lookup matches
// against, plus per-subject grantee state. The share endpoints store only ids;
// the list handler resolves them back to `{ id, username }` via this directory,
// ordered by username — mirroring the server. In-memory (reset between tests).
export const mockUserDirectory: UserLookupEntry[] = [
  { id: 'user-ann', username: 'ann' },
  { id: 'user-anna', username: 'anna' },
  { id: 'user-bob', username: 'bob' },
  { id: 'user-cleo', username: 'cleo' },
  // A larger block sharing the "user" prefix so the people-picker's paging +
  // "keep typing to narrow" hint and the scrollable grantee list are exercisable
  // in the dev:mock run — type "user" to see a capped page + the hint, then add
  // many to watch the "Shared with" list scroll.
  ...Array.from({ length: 24 }, (_, i) => {
    const n = i + 1
    return { id: `user-${n}`, username: `user${n}` }
  }),
]

let mockDefinitionShares: Record<string, string[]> = {}
let mockCollectionShares: Record<string, string[]> = {}

export function resetMockShares(): void {
  mockDefinitionShares = {}
  mockCollectionShares = {}
}

// Seed one game + one collection **owned by another user and shared with** the
// signed-in mock user, so the "Shared with me" Play-3D page has content in
// dev:mock (and the e2e can drive it). Owned by a directory user with visibility
// `shared`, so they're excluded from `scope=mine` and only surface under
// `scope=shared`/`visible`. Idempotent by id; the grant lives in the in-memory
// share maps (re-applied on each module load, so a page reload keeps it).
const SHARED_WITH_ME_GAME_ID = 'def-shared-with-me'
const SHARED_WITH_ME_COLLECTION_ID = 'col-shared-with-me'

function seedSharedWithMe(): void {
  const owner = mockUserDirectory.find(u => u.username === 'bob')!.id
  const now = '2026-01-01T00:00:00.000Z'
  if (!mockGameDefinitions.some(d => d.id === SHARED_WITH_ME_GAME_ID)) {
    mockGameDefinitions = [...mockGameDefinitions, {
      id: SHARED_WITH_ME_GAME_ID,
      ownerId: owner,
      name: 'Shared Adventure',
      description: 'A game a friend shared with you.',
      visibility: 'shared',
      seed: 909090,
      rotation: 'static',
      config: { rows: 6, cols: 6, seed: 909090 },
      createdAt: now,
      updatedAt: now,
    }]
  }
  if (!mockGameCollections.some(c => c.id === SHARED_WITH_ME_COLLECTION_ID)) {
    mockGameCollections = [...mockGameCollections, {
      id: SHARED_WITH_ME_COLLECTION_ID,
      ownerId: owner,
      name: 'Shared Journey',
      description: 'A collection a friend shared with you.',
      visibility: 'shared',
      playMode: 'arcade',
      items: [{ definitionId: SHARED_WITH_ME_GAME_ID, sortOrder: 0 }],
      createdAt: now,
      updatedAt: now,
    }]
  }
  mockDefinitionShares[SHARED_WITH_ME_GAME_ID] = [mockProfile.id]
  mockCollectionShares[SHARED_WITH_ME_COLLECTION_ID] = [mockProfile.id]
}
seedSharedWithMe()

// Seed a **public** game + collection owned by another user, so the Community
// scope has content in dev:mock. Owned by a directory user with visibility
// `public`, so they surface only under `scope=public` (and `visible`) — never
// under `mine` (not owned by the signed-in user) or `shared` (no grant).
// Idempotent by id.
const COMMUNITY_GAME_ID = 'def-community'
const COMMUNITY_GAME_2_ID = 'def-community-2'
const COMMUNITY_COLLECTION_ID = 'col-community'

function seedCommunity(): void {
  const owner = mockUserDirectory.find(u => u.username === 'cleo')!.id
  const now = '2026-01-01T00:00:00.000Z'
  const later = '2026-02-01T00:00:00.000Z'
  if (!mockGameDefinitions.some(d => d.id === COMMUNITY_GAME_ID)) {
    mockGameDefinitions = [...mockGameDefinitions, {
      id: COMMUNITY_GAME_ID,
      ownerId: owner,
      name: 'Community Classic',
      description: 'A game published for everyone.',
      visibility: 'public',
      seed: 707070,
      rotation: 'static',
      config: { rows: 8, cols: 8, seed: 707070 },
      createdAt: now,
      updatedAt: now,
    }]
  }
  if (!mockGameDefinitions.some(d => d.id === COMMUNITY_GAME_2_ID)) {
    mockGameDefinitions = [...mockGameDefinitions, {
      id: COMMUNITY_GAME_2_ID,
      ownerId: owner,
      name: 'Zephyr Heights',
      description: 'A newer game published for everyone.',
      visibility: 'public',
      seed: 606060,
      rotation: 'static',
      config: { rows: 10, cols: 10, seed: 606060 },
      createdAt: later,
      updatedAt: later,
    }]
  }
  if (!mockGameCollections.some(c => c.id === COMMUNITY_COLLECTION_ID)) {
    mockGameCollections = [...mockGameCollections, {
      id: COMMUNITY_COLLECTION_ID,
      ownerId: owner,
      name: 'Community Picks',
      description: 'A collection published for everyone.',
      visibility: 'public',
      playMode: 'arcade',
      items: [{ definitionId: COMMUNITY_GAME_ID, sortOrder: 0 }],
      createdAt: now,
      updatedAt: now,
    }]
  }
}
seedCommunity()

// Seed a **daily** curated game + a curated "Daily Challenges" collection holding
// it, so the Today's Challenge Home tile (which client-resolves that collection)
// and the daily leaderboard date picker have content in dev:mock. Curated ⇒ both
// surface under Featured. The daily game reports a couple of past days with runs
// (see the board-dates handler) so the quick-pick chips are exercisable.
// Idempotent by id. Mirrors the server's G-phase bootstrap seed.
const DAILY_GAME_ID = 'def-daily'
const DAILY_COLLECTION_ID = 'col-daily-challenges'
// Past days this daily game has a non-empty board — literal dates so the fixture
// is clock-independent (today is always browsable via the date input regardless).
const DAILY_BOARD_DATES = ['2026-07-10', '2026-07-05']

function seedDailyChallenges(): void {
  const owner = mockUserDirectory.find(u => u.username === 'cleo')!.id
  const now = '2026-01-01T00:00:00.000Z'
  if (!mockGameDefinitions.some(d => d.id === DAILY_GAME_ID)) {
    mockGameDefinitions = [...mockGameDefinitions, {
      id: DAILY_GAME_ID,
      ownerId: owner,
      name: 'Daily Maze',
      description: 'A fresh maze every day.',
      visibility: 'curated',
      seed: 505050,
      rotation: 'daily',
      config: { rows: 9, cols: 9, seed: 505050 },
      createdAt: now,
      updatedAt: now,
    }]
  }
  if (!mockGameCollections.some(c => c.id === DAILY_COLLECTION_ID)) {
    mockGameCollections = [...mockGameCollections, {
      id: DAILY_COLLECTION_ID,
      ownerId: owner,
      name: 'Daily Challenges',
      description: 'A new challenge every day.',
      visibility: 'curated',
      playMode: 'arcade',
      items: [{ definitionId: DAILY_GAME_ID, sortOrder: 0 }],
      createdAt: now,
      updatedAt: now,
    }]
  }
}
seedDailyChallenges()

// A few directory users have an avatar (id → marker) so the grantee list's
// `<Avatar>` fetch path is exercisable in the dev:mock run; the rest fall back
// to the placeholder. The marker doubles as the has-avatar gate + cache-buster,
// exactly as the server's `GranteeSummary.avatar_updated_at`.
const mockGranteeAvatars: Record<string, string> = {
  'user-bob': '2026-01-03T00:00:00.000Z',
  'user-1': '2026-01-01T00:00:00.000Z',
  'user-2': '2026-01-02T00:00:00.000Z',
}

// Resolves grantee ids to `{ id, username, avatar_updated_at? }` summaries via
// the directory, dropping unknown ids and ordering by username (as the server's
// JOIN does).
function granteeSummaries(ids: string[]): GranteeSummary[] {
  return ids
    .map(uid => mockUserDirectory.find(u => u.id === uid))
    .filter((u): u is UserLookupEntry => u !== undefined)
    .map(u => ({ id: u.id, username: u.username, avatar_updated_at: mockGranteeAvatars[u.id] }))
    .sort((a, b) => a.username.localeCompare(b.username))
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

// In-memory avatar state: a tiny valid 1x1 PNG served as the stored image, plus
// the `avatar_updated_at` marker. Upload stamps the marker (so GET /users/me
// reflects it and the avatar GET serves bytes); remove clears it. Stateful so
// the upload -> display flow works end-to-end through the service worker (e2e).
const MOCK_AVATAR_PNG = Uint8Array.from(
  atob('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M8AAAMEAYHEr8+/AAAAAElFTkSuQmCC'),
  c => c.charCodeAt(0),
)
let mockAvatarUpdatedAt: string | null = null

export function resetMockAvatar(): void {
  mockAvatarUpdatedAt = null
}

export const handlers = [
  http.get(`${BASE}/features`, () => {
    return HttpResponse.json<AppFeatures>({ allow_signup: true, oauth_providers: [], email_enabled: true, max_maze_cells: null })
  }),

  http.put(`${BASE}/admin/features`, async ({ request }) => {
    const body = await request.json() as AppFeatures
    return HttpResponse.json<AppFeatures>(body)
  }),

  http.post(`${BASE}/login`, async ({ request }) => {
    const body = await request.json().catch(() => ({})) as { email?: string }
    setSignedInAdmin((body.email ?? '').toLowerCase() === ADMIN_LOGIN_EMAIL)
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
    return HttpResponse.json({ ...mockProfile, is_admin: signedInIsAdmin(), avatar_updated_at: mockAvatarUpdatedAt })
  }),

  // Avatar serve — returns the stored bytes for a seeded grantee avatar, or the
  // signed-in user's own uploaded avatar (that branch ignores the id, mirroring
  // the single mock user); else 404 (the client then shows the placeholder).
  http.get(`${BASE}/users/:id/avatar`, ({ params }) => {
    const hasSeededAvatar = mockGranteeAvatars[String(params.id)] != null
    if (!hasSeededAvatar && mockAvatarUpdatedAt == null) return new HttpResponse(null, { status: 404 })
    return new HttpResponse(MOCK_AVATAR_PNG, { headers: { 'Content-Type': 'image/png' } })
  }),

  http.post(`${BASE}/users/me/avatar`, () => {
    mockAvatarUpdatedAt = new Date().toISOString()
    return HttpResponse.json({ avatar_updated_at: mockAvatarUpdatedAt })
  }),

  http.delete(`${BASE}/users/me/avatar`, () => {
    mockAvatarUpdatedAt = null
    return new HttpResponse(null, { status: 204 })
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
      ?? (params.id === mockMazeTreasure.id ? mockMazeTreasure : undefined)
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

  // Game definitions — the list the caller may see, and create. `seed` is
  // server-minted, so the create handler stamps one.
  http.get(`${BASE}/game-definitions`, ({ request }) => {
    const url = new URL(request.url)
    const scope = url.searchParams.get('scope')
    const q = (url.searchParams.get('q') ?? '').trim().toLowerCase()
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    const excludeDefinitions = url.searchParams.get('excludeDefinitions') === 'true'
    let defs = [...mockGameDefinitions].sort((a, b) => a.name.localeCompare(b.name))
    if (scope === 'mine') defs = defs.filter(d => d.ownerId === mockProfile.id)
    if (scope === 'shared') {
      defs = defs.filter(d =>
        d.ownerId !== mockProfile.id && d.visibility === 'shared' && (mockDefinitionShares[d.id] ?? []).includes(mockProfile.id))
    }
    if (scope === 'public') {
      defs = defs.filter(d => d.visibility === 'public' && d.ownerId !== mockProfile.id)
      // The server honours `sort` for the public scope only; every other scope
      // stays name-ordered.
      if (url.searchParams.get('sort') === 'newest') {
        defs = [...defs].sort((a, b) => b.createdAt.localeCompare(a.createdAt) || a.id.localeCompare(b.id))
      }
    }
    if (q !== '') defs = defs.filter(d => d.name.toLowerCase().includes(q))
    let page = defs.slice(offset, offset + limit)
    if (excludeDefinitions) page = page.map(d => ({ ...d, config: {} }))
    return HttpResponse.json<GameDefinitionListResponse>({
      definitions: page,
      limit,
      offset,
      hasMore: offset + limit < defs.length,
    })
  }),

  // Play-fetch of one definition. The real server splices the effective seed into
  // `config` and computes the subject key; a Static mock needs neither mixed.
  http.get(`${BASE}/game-definitions/:id`, ({ params }) => {
    const def = mockGameDefinitions.find(d => d.id === params.id)
    if (!def) return new HttpResponse(null, { status: 404 })
    return HttpResponse.json<GamePlayResponse>({
      ...def,
      config: { ...def.config, seed: def.seed },
      challengeKey: `def:${def.id}`,
      // Every game is tracked; a private game's board is just owner-only.
      leaderboardTracked: true,
    })
  }),

  http.put(`${BASE}/game-definitions/:id`, async ({ params, request }) => {
    const body = await request.json() as GameDefinitionRequest
    const index = mockGameDefinitions.findIndex(d => d.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    // A rename can't collide with another game's name (per-owner uniqueness).
    if (mockGameDefinitions.some(d => d.id !== params.id && d.name.toLowerCase() === body.name.toLowerCase())) {
      return new HttpResponse(`A game definition named '${body.name}' already exists`, { status: 409 })
    }
    // `seed`, `ownerId` and `createdAt` are server-owned and preserved.
    const updated: GameDefinition = {
      ...mockGameDefinitions[index],
      name: body.name,
      description: body.description ?? undefined,
      visibility: body.visibility ?? mockGameDefinitions[index].visibility,
      rotation: body.rotation ?? mockGameDefinitions[index].rotation,
      config: body.config,
      updatedAt: new Date().toISOString(),
    }
    mockGameDefinitions = mockGameDefinitions.map((d, i) => (i === index ? updated : d))
    saveGameDefinitions()
    return HttpResponse.json(updated)
  }),

  // Reshuffle re-mints the seed (and, on the real server, resets the board).
  http.post(`${BASE}/game-definitions/:id/reshuffle`, ({ params }) => {
    const index = mockGameDefinitions.findIndex(d => d.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    const reshuffled: GameDefinition = {
      ...mockGameDefinitions[index],
      seed: mockGameDefinitions[index].seed + 1,
      updatedAt: new Date().toISOString(),
    }
    mockGameDefinitions = mockGameDefinitions.map((d, i) => (i === index ? reshuffled : d))
    saveGameDefinitions()
    return HttpResponse.json(reshuffled)
  }),

  http.delete(`${BASE}/game-definitions/:id`, ({ params }) => {
    const index = mockGameDefinitions.findIndex(d => d.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    mockGameDefinitions = mockGameDefinitions.filter(d => d.id !== params.id)
    saveGameDefinitions()
    return new HttpResponse(null, { status: 204 })
  }),

  http.post(`${BASE}/game-definitions`, async ({ request }) => {
    const body = await request.json() as GameDefinitionRequest
    // Names are unique per owner (all mock definitions share one owner), matching
    // the server's 409 so dev:mock can't create colliding games.
    if (mockGameDefinitions.some(d => d.name.toLowerCase() === body.name.toLowerCase())) {
      return new HttpResponse(`A game definition named '${body.name}' already exists`, { status: 409 })
    }
    const now = new Date().toISOString()
    const created: GameDefinition = {
      id: `def-${Date.now()}`,
      ownerId: mockProfile.id,
      name: body.name,
      description: body.description ?? undefined,
      visibility: body.visibility ?? 'private',
      seed: 424242,
      rotation: body.rotation ?? 'static',
      config: body.config,
      createdAt: now,
      updatedAt: now,
    }
    mockGameDefinitions = [...mockGameDefinitions, created]
    saveGameDefinitions()
    return HttpResponse.json(created, { status: 201 })
  }),

  // User lookup — username-prefix search for the share people-picker. A blank
  // prefix returns an empty page (never enumerates every user).
  http.get(`${BASE}/users/lookup`, ({ request }) => {
    const url = new URL(request.url)
    const prefix = (url.searchParams.get('username') ?? '').trim().toLowerCase()
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    const matches = prefix === ''
      ? []
      : mockUserDirectory
          .filter(u => u.username.toLowerCase().startsWith(prefix))
          .sort((a, b) => a.username.localeCompare(b.username))
    const users = matches.slice(offset, offset + limit)
    return HttpResponse.json<UserLookupResponse>({
      users,
      limit,
      offset,
      has_more: offset + limit < matches.length,
    })
  }),

  // Definition shares — list / set. PUT replaces the whole grantee list with the
  // supplied set (owner's own id filtered); visibility is set separately.
  http.get(`${BASE}/game-definitions/:id/shares`, ({ params }) =>
    HttpResponse.json({ grantees: granteeSummaries(mockDefinitionShares[String(params.id)] ?? []) }),
  ),
  http.put(`${BASE}/game-definitions/:id/shares`, async ({ params, request }) => {
    const { userIds } = await request.json() as { userIds: string[] }
    const id = String(params.id)
    const owner = mockGameDefinitions.find(d => d.id === id)?.ownerId
    mockDefinitionShares[id] = [...new Set(userIds.filter(u => u !== owner))]
    return HttpResponse.json({ grantees: granteeSummaries(mockDefinitionShares[id]) })
  }),

  // Collections — list (own + accessible) and create. Membership item + detail
  // endpoints land with the membership editor step.
  http.get(`${BASE}/game-collections`, ({ request }) => {
    const url = new URL(request.url)
    const scope = url.searchParams.get('scope')
    const q = (url.searchParams.get('q') ?? '').trim().toLowerCase()
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    let cols = [...mockGameCollections].sort((a, b) => a.name.localeCompare(b.name))
    if (scope === 'mine') cols = cols.filter(c => c.ownerId === mockProfile.id)
    if (scope === 'shared') {
      cols = cols.filter(c =>
        c.ownerId !== mockProfile.id && c.visibility === 'shared' && (mockCollectionShares[c.id] ?? []).includes(mockProfile.id))
    }
    if (scope === 'public') {
      cols = cols.filter(c => c.visibility === 'public' && c.ownerId !== mockProfile.id)
      if (url.searchParams.get('sort') === 'newest') {
        cols = [...cols].sort((a, b) => b.createdAt.localeCompare(a.createdAt) || a.id.localeCompare(b.id))
      }
    }
    if (q !== '') cols = cols.filter(c => c.name.toLowerCase().includes(q))
    return HttpResponse.json<GameCollectionListResponse>({
      collections: cols.slice(offset, offset + limit),
      limit,
      offset,
      hasMore: offset + limit < cols.length,
    })
  }),

  http.post(`${BASE}/game-collections`, async ({ request }) => {
    const body = await request.json() as GameCollectionRequest
    const now = new Date().toISOString()
    const created: GameCollection = {
      id: `col-${Date.now()}`,
      ownerId: mockProfile.id,
      name: body.name,
      description: body.description ?? undefined,
      visibility: body.visibility ?? 'private',
      playMode: body.playMode ?? 'arcade',
      items: [],
      createdAt: now,
      updatedAt: now,
    }
    mockGameCollections = [...mockGameCollections, created]
    saveGameCollections()
    return HttpResponse.json(created, { status: 201 })
  }),

  // Collection detail — metadata plus its member definitions, hydrated in order
  // from the definition store (missing refs dropped, mirroring the server).
  http.get(`${BASE}/game-collections/:id`, ({ params }) => {
    const collection = mockGameCollections.find(c => c.id === params.id)
    if (!collection) return new HttpResponse(null, { status: 404 })
    const definitions = [...collection.items]
      .sort((a, b) => a.sortOrder - b.sortOrder)
      .map(item => mockGameDefinitions.find(d => d.id === item.definitionId))
      .filter((d): d is GameDefinition => d !== undefined)
    return HttpResponse.json<GameCollectionDetailResponse>({
      id: collection.id,
      ownerId: collection.ownerId,
      name: collection.name,
      description: collection.description,
      visibility: collection.visibility,
      playMode: collection.playMode,
      imageUpdatedAt: collection.imageUpdatedAt,
      createdAt: collection.createdAt,
      updatedAt: collection.updatedAt,
      definitions,
    })
  }),

  http.put(`${BASE}/game-collections/:id`, async ({ params, request }) => {
    const body = await request.json() as GameCollectionRequest
    const index = mockGameCollections.findIndex(c => c.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    // `ownerId`, `items` and `createdAt` are server-owned / managed elsewhere.
    const updated: GameCollection = {
      ...mockGameCollections[index],
      name: body.name,
      description: body.description ?? undefined,
      visibility: body.visibility ?? mockGameCollections[index].visibility,
      // Mirror the server: the update overwrites playMode, defaulting to arcade
      // when the body omits it.
      playMode: body.playMode ?? 'arcade',
      updatedAt: new Date().toISOString(),
    }
    mockGameCollections = mockGameCollections.map((c, i) => (i === index ? updated : c))
    saveGameCollections()
    return HttpResponse.json(updated)
  }),

  http.delete(`${BASE}/game-collections/:id`, ({ params }) => {
    const index = mockGameCollections.findIndex(c => c.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    mockGameCollections = mockGameCollections.filter(c => c.id !== params.id)
    saveGameCollections()
    return new HttpResponse(null, { status: 204 })
  }),

  // Membership reconcile — replaces the whole ordered list (deduped) in one op.
  http.put(`${BASE}/game-collections/:id/items`, async ({ params, request }) => {
    const { definitionIds } = await request.json() as { definitionIds: string[] }
    const index = mockGameCollections.findIndex(c => c.id === params.id)
    if (index === -1) return new HttpResponse(null, { status: 404 })
    const items = [...new Set(definitionIds)].map((definitionId, sortOrder) => ({ definitionId, sortOrder }))
    const updated: GameCollection = {
      ...mockGameCollections[index],
      items,
      updatedAt: new Date().toISOString(),
    }
    mockGameCollections = mockGameCollections.map((c, i) => (i === index ? updated : c))
    saveGameCollections()
    return HttpResponse.json(updated)
  }),

  // Collection shares — mirror of the definition share endpoints.
  http.get(`${BASE}/game-collections/:id/shares`, ({ params }) =>
    HttpResponse.json({ grantees: granteeSummaries(mockCollectionShares[String(params.id)] ?? []) }),
  ),
  http.put(`${BASE}/game-collections/:id/shares`, async ({ params, request }) => {
    const { userIds } = await request.json() as { userIds: string[] }
    const id = String(params.id)
    mockCollectionShares[id] = [...new Set(userIds)]
    return HttpResponse.json({ grantees: granteeSummaries(mockCollectionShares[id]) })
  }),

  // Featured catalogue — the admin-ordered curated defs + collections. GET
  // hydrates + filters to still-curated entries (the server's projection); the
  // /order PUT rewrites the order, rejecting a non-curated / unknown entry (400).
  http.get(`${BASE}/featured-game-items`, ({ request }) => {
    const url = new URL(request.url)
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    const hydrated = hydrateFeaturedItems()
    return HttpResponse.json<FeaturedGameItemsListResponse>({
      items: hydrated.slice(offset, offset + limit),
      limit,
      offset,
      hasMore: offset + limit < hydrated.length,
    })
  }),

  http.put(`${BASE}/featured-game-items/order`, async ({ request }) => {
    const { entries } = await request.json() as { entries: FeaturedGameItemEntry[] }
    const seen = new Set<string>()
    const deduped: FeaturedGameItemEntry[] = []
    for (const e of entries) {
      const key = `${e.kind}:${e.id}`
      if (seen.has(key)) continue
      seen.add(key)
      const entity = e.kind === 'definition'
        ? mockGameDefinitions.find(d => d.id === e.id)
        : mockGameCollections.find(c => c.id === e.id)
      if (!entity || entity.visibility !== 'curated') {
        return new HttpResponse(`Cannot feature a non-curated ${e.kind} '${e.id}'`, { status: 400 })
      }
      deduped.push({ kind: e.kind, id: e.id })
    }
    mockFeaturedGameItems = deduped
    const hydrated = hydrateFeaturedItems()
    return HttpResponse.json<FeaturedGameItemsListResponse>({ items: hydrated, limit: hydrated.length, offset: 0, hasMore: false })
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

  // Campaign progress: which of the requested challenges the caller has scored on.
  // Default: none. Tests override with server.use to mark specific games complete.
  http.post(`${BASE}/scores/me/completed`, () =>
    HttpResponse.json<CompletedChallengesResponse>({ completed: [] })),

  // Dated boards a daily game has runs on (most recent first) — the quick-picks
  // for the daily leaderboard date picker. Only the seeded daily game reports
  // days; every other game (or an unplayed daily one) is empty.
  http.get(`${BASE}/scores/board-dates`, ({ request }) => {
    const definitionId = new URL(request.url).searchParams.get('definition_id')
    const dates = definitionId === DAILY_GAME_ID ? DAILY_BOARD_DATES : []
    return HttpResponse.json<BoardDatesResponse>({ dates })
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
