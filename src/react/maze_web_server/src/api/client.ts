import type { AddUserEmailRequest, AppFeatures, ChangePasswordRequest, GameCollection, GameCollectionDetailResponse, GameCollectionListResponse, GameCollectionRequest, GameDefinition, GameDefinitionListResponse, GameDefinitionRequest, GamePlayResponse, LoginResponse, Maze, Play3dConfig, RenewResponse, ResetScoresResponse, SaveMazeRequest, ScoreboardResponse, ScoreMetric, GameDefinitionSharesResponse, GameCollectionSharesResponse, SortDirection, UpdateProfileRequest, UserEmailsResponse, UserLookupResponse, UserProfile } from '../types/api'

const BASE = '/api/v1'

async function throwForStatus(response: Response): Promise<never> {
  const message = await response.text().catch(() => response.statusText)
  throw Object.assign(new Error(message || response.statusText), { status: response.status })
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE}${path}`, options)
  if (!response.ok) await throwForStatus(response)
  return response.json() as Promise<T>
}

async function requestEmpty(path: string, options?: RequestInit): Promise<void> {
  const response = await fetch(`${BASE}${path}`, options)
  if (!response.ok) await throwForStatus(response)
}

function authHeaders(token: string): Record<string, string> {
  return { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' }
}

export function getFeatures(): Promise<AppFeatures> {
  return request<AppFeatures>('/features')
}

export function updateAdminFeatures(token: string, features: AppFeatures): Promise<AppFeatures> {
  return request<AppFeatures>('/admin/features', {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify(features),
  })
}

export function login(email: string, password: string): Promise<LoginResponse> {
  return request<LoginResponse>('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  })
}

// The server reads the login ID from the Bearer token itself — no extra header needed.
export async function logout(token: string): Promise<void> {
  await fetch(`${BASE}/logout`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  })
}

export function renewToken(token: string): Promise<RenewResponse> {
  return request<RenewResponse>('/login/renew', {
    method: 'POST',
    headers: authHeaders(token),
  })
}

export function signup(email: string, password: string): Promise<void> {
  return requestEmpty('/signup', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  })
}

// Kicks off the server-mediated OAuth flow. The browser navigates away to the
// server, which 302s onward to the provider; we never see JSON. The bearer
// token comes back as a fragment on `/oauth/callback`.
export function startOAuth(provider: string): void {
  window.location.href = `${BASE}/auth/oauth/${encodeURIComponent(provider)}/start?origin=web`
}

export function getMe(token: string): Promise<UserProfile> {
  return request<UserProfile>('/users/me', {
    headers: authHeaders(token),
  })
}

// Builds the avatar request path for a user, appending the `avatar_updated_at`
// marker as a `?v=` cache-buster when known. This is the URL `fetchUserAvatar`
// requests — NOT an `<img src>`: the route is guarded, so the image is loaded
// via an authenticated fetch (a bare `<img>` can't carry the bearer token).
export function avatarUrl(userId: string, updatedAt?: string | null): string {
  const base = `${BASE}/users/${encodeURIComponent(userId)}/avatar`
  return updatedAt ? `${base}?v=${encodeURIComponent(updatedAt)}` : base
}

// Fetches a user's avatar image as a Blob over an authenticated request.
// Resolves to the Blob on success and throws (with a `.status`) on a non-OK
// response — callers treat a 404 as "no avatar" and fall back to the placeholder.
export async function fetchUserAvatar(token: string, userId: string, updatedAt?: string | null): Promise<Blob> {
  const response = await fetch(avatarUrl(userId, updatedAt), {
    headers: { Authorization: `Bearer ${token}` },
  })
  if (!response.ok) await throwForStatus(response)
  return response.blob()
}

// Uploads (or replaces) the caller's avatar. The server canonicalises the image
// to a 256x256 PNG and returns the new `avatar_updated_at` marker. Note: no
// `Content-Type` header — the browser sets `multipart/form-data` with the
// correct boundary for a `FormData` body; `authHeaders` would wrongly force JSON.
export function uploadAvatar(token: string, file: File): Promise<{ avatar_updated_at: string }> {
  const form = new FormData()
  form.append('file', file)
  return request<{ avatar_updated_at: string }>('/users/me/avatar', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
    body: form,
  })
}

// Removes the caller's avatar (idempotent server-side — 204 even if none set).
export function deleteAvatar(token: string): Promise<void> {
  return requestEmpty('/users/me/avatar', {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

export function updateProfile(token: string, body: UpdateProfileRequest): Promise<UserProfile> {
  return request<UserProfile>('/users/me/profile', {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

export function changePassword(token: string, body: ChangePasswordRequest): Promise<void> {
  return requestEmpty('/users/me/password', {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

export function deleteMe(token: string): Promise<void> {
  return requestEmpty('/users/me', {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

export function getMyEmails(token: string): Promise<UserEmailsResponse> {
  return request<UserEmailsResponse>('/users/me/emails', {
    headers: authHeaders(token),
  })
}

export function addMyEmail(token: string, email: string): Promise<UserEmailsResponse> {
  const body: AddUserEmailRequest = { email }
  return request<UserEmailsResponse>('/users/me/emails', {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

export function removeMyEmail(token: string, email: string): Promise<UserEmailsResponse> {
  return request<UserEmailsResponse>(`/users/me/emails/${encodeURIComponent(email)}`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

export function setPrimaryEmail(token: string, email: string): Promise<UserEmailsResponse> {
  return request<UserEmailsResponse>(`/users/me/emails/${encodeURIComponent(email)}/primary`, {
    method: 'PUT',
    headers: authHeaders(token),
  })
}

export function requestPasswordReset(email: string): Promise<void> {
  return requestEmpty('/password-reset/request', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email }),
  })
}

export function confirmPasswordReset(token: string, newPassword: string): Promise<void> {
  return requestEmpty('/password-reset/confirm', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token, new_password: newPassword }),
  })
}

export function requestEmailVerification(token: string, email: string): Promise<void> {
  return requestEmpty('/email-verifications/request', {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify({ email }),
  })
}

export function confirmEmailVerification(verificationToken: string): Promise<void> {
  return requestEmpty('/email-verifications/confirm', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token: verificationToken }),
  })
}

interface MazeListItem {
  id: string
  name: string
  definition: string | null  // server returns definition as a JSON string, not a nested object
}

export async function getMazes(token: string, includeDefinitions: boolean): Promise<Maze[]> {
  const qs = includeDefinitions ? '?includeDefinitions=true' : ''
  const items = await request<MazeListItem[]>(`/mazes${qs}`, {
    headers: authHeaders(token),
  })
  return items
    .map(item => {
      // definition is the full Maze JSON string: {id, name, definition:{grid}, game_settings?}
      const parsed = item.definition ? (JSON.parse(item.definition) as Maze) : null
      return {
        id: item.id,
        name: item.name,
        definition: parsed?.definition ?? { grid: [] },
        game_settings: parsed?.game_settings,
      }
    })
    .sort((a, b) => a.name.localeCompare(b.name))
}

export function getMaze(token: string, id: string): Promise<Maze> {
  return request<Maze>(`/mazes/${encodeURIComponent(id)}`, {
    headers: authHeaders(token),
  })
}

export function createMaze(token: string, body: SaveMazeRequest): Promise<Maze> {
  return request<Maze>('/mazes', {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify({ id: '', ...body }),
  })
}

export function updateMaze(token: string, id: string, body: SaveMazeRequest): Promise<Maze> {
  return request<Maze>(`/mazes/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify({ id, ...body }),
  })
}

export function deleteMaze(token: string, id: string): Promise<void> {
  return requestEmpty(`/mazes/${encodeURIComponent(id)}`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

// --- Scores -----------------------------------------------------------------

export interface LeaderboardQuery {
  // Exactly one subject (mazeId or challenge) must be set.
  mazeId?: string
  challenge?: string
  metric?: ScoreMetric
  direction?: SortDirection
  limit?: number
  offset?: number
  includeUsernames?: boolean
}

export interface HistoryQuery {
  limit?: number
  offset?: number
}

// Reads a page of a leaderboard, ranked by `metric` / `direction` (the server
// defaults to fastest-time-first when omitted). Exactly one subject — a stored
// `mazeId` or a curated `challenge` — must be set.
export function getLeaderboard(token: string, query: LeaderboardQuery): Promise<ScoreboardResponse> {
  if ((query.mazeId == null) === (query.challenge == null)) {
    throw new Error('getLeaderboard requires exactly one of mazeId / challenge')
  }
  const params = new URLSearchParams()
  if (query.mazeId != null) params.set('maze_id', query.mazeId)
  if (query.challenge != null) params.set('challenge', query.challenge)
  if (query.metric != null) params.set('metric', query.metric)
  if (query.direction != null) params.set('direction', query.direction)
  if (query.limit != null) params.set('limit', String(query.limit))
  if (query.offset != null) params.set('offset', String(query.offset))
  if (query.includeUsernames != null) params.set('include_usernames', String(query.includeUsernames))
  return request<ScoreboardResponse>(`/scores?${params.toString()}`, {
    headers: authHeaders(token),
  })
}

// Resets a leaderboard to empty (DELETE). Exactly one subject — a stored `mazeId`
// (maze owner only) or a curated `challenge` (admin only); the server enforces
// access and rejects otherwise. Returns the number of score rows removed.
export function resetLeaderboard(token: string, query: { mazeId?: string; challenge?: string }): Promise<ResetScoresResponse> {
  if ((query.mazeId == null) === (query.challenge == null)) {
    throw new Error('resetLeaderboard requires exactly one of mazeId / challenge')
  }
  const params = new URLSearchParams()
  if (query.mazeId != null) params.set('maze_id', query.mazeId)
  if (query.challenge != null) params.set('challenge', query.challenge)
  return request<ResetScoresResponse>(`/scores?${params.toString()}`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

// Reads a page of the authenticated player's own run history (most recent first).
export function getScoreHistory(token: string, query: HistoryQuery = {}): Promise<ScoreboardResponse> {
  const params = new URLSearchParams()
  if (query.limit != null) params.set('limit', String(query.limit))
  if (query.offset != null) params.set('offset', String(query.offset))
  const qs = params.toString()
  return request<ScoreboardResponse>(`/scores/me${qs ? `?${qs}` : ''}`, {
    headers: authHeaders(token),
  })
}

// Reads a curated difficulty's preset (unauthenticated). The leaderboard UI
// uses its fixed `seed` to build the challenge board key.
export function getPlay3dConfig(difficulty: string): Promise<Play3dConfig> {
  return request<Play3dConfig>(`/game/play3d-config?difficulty=${encodeURIComponent(difficulty)}`)
}

// --- Game definitions & collections -----------------------------------------

interface PageQuery {
  limit?: number
  offset?: number
}

function pageQuery(query: PageQuery): string {
  const params = new URLSearchParams()
  if (query.limit != null) params.set('limit', String(query.limit))
  if (query.offset != null) params.set('offset', String(query.offset))
  const qs = params.toString()
  return qs ? `?${qs}` : ''
}

export function createGameDefinition(token: string, body: GameDefinitionRequest): Promise<GameDefinition> {
  return request<GameDefinition>('/game-definitions', {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

// Play-fetch of a single definition — access-gated (a 404 hides anything the
// caller can't see). The returned `config` has the effective seed spliced in.
export function getGameDefinition(token: string, id: string): Promise<GamePlayResponse> {
  return request<GamePlayResponse>(`/game-definitions/${encodeURIComponent(id)}`, {
    headers: authHeaders(token),
  })
}

// A page of the definitions the caller may see (own ∨ shared ∨ public ∨ curated).
export function listGameDefinitions(token: string, query: PageQuery = {}): Promise<GameDefinitionListResponse> {
  return request<GameDefinitionListResponse>(`/game-definitions${pageQuery(query)}`, {
    headers: authHeaders(token),
  })
}

export function updateGameDefinition(token: string, id: string, body: GameDefinitionRequest): Promise<GameDefinition> {
  return request<GameDefinition>(`/game-definitions/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

export function deleteGameDefinition(token: string, id: string): Promise<void> {
  return requestEmpty(`/game-definitions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

// Re-mint a definition's seed to change its generated layout. The server owns
// the seed (PUT preserves it), so reshuffling is its own endpoint; it also
// resets the definition's leaderboard when published. Returns the definition
// with its new seed.
export function reshuffleGameDefinition(token: string, id: string): Promise<GameDefinition> {
  return request<GameDefinition>(`/game-definitions/${encodeURIComponent(id)}/reshuffle`, {
    method: 'POST',
    headers: authHeaders(token),
  })
}

export function createGameCollection(token: string, body: GameCollectionRequest): Promise<GameCollection> {
  return request<GameCollection>('/game-collections', {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

// Collection detail — the collection plus its accessible member definitions,
// hydrated and in order (inaccessible / dangling members dropped server-side).
export function getGameCollection(token: string, id: string): Promise<GameCollectionDetailResponse> {
  return request<GameCollectionDetailResponse>(`/game-collections/${encodeURIComponent(id)}`, {
    headers: authHeaders(token),
  })
}

// A page of the collections the caller may see.
export function listGameCollections(token: string, query: PageQuery = {}): Promise<GameCollectionListResponse> {
  return request<GameCollectionListResponse>(`/game-collections${pageQuery(query)}`, {
    headers: authHeaders(token),
  })
}

export function updateGameCollection(token: string, id: string, body: GameCollectionRequest): Promise<GameCollection> {
  return request<GameCollection>(`/game-collections/${encodeURIComponent(id)}`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify(body),
  })
}

export function deleteGameCollection(token: string, id: string): Promise<void> {
  return requestEmpty(`/game-collections/${encodeURIComponent(id)}`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}

// Appends a definition to a collection (idempotent). Returns the updated
// collection with its raw membership (`items`), not the hydrated detail.
export function addGameCollectionItem(token: string, collectionId: string, definitionId: string): Promise<GameCollection> {
  return request<GameCollection>(`/game-collections/${encodeURIComponent(collectionId)}/items`, {
    method: 'POST',
    headers: authHeaders(token),
    body: JSON.stringify({ definitionId }),
  })
}

// Removes a definition from a collection (idempotent). Returns the updated
// collection.
export function removeGameCollectionItem(token: string, collectionId: string, definitionId: string): Promise<GameCollection> {
  return request<GameCollection>(
    `/game-collections/${encodeURIComponent(collectionId)}/items/${encodeURIComponent(definitionId)}`,
    { method: 'DELETE', headers: authHeaders(token) },
  )
}

// Rewrites the member order to `ordered` (non-members ignored; members omitted
// from `ordered` keep their prior relative order after the listed ones).
export function reorderGameCollectionItems(token: string, collectionId: string, ordered: string[]): Promise<GameCollection> {
  return request<GameCollection>(`/game-collections/${encodeURIComponent(collectionId)}/items/reorder`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify({ ordered }),
  })
}

// --- Sharing & user lookup --------------------------------------------------

// The share endpoints (definition + collection) all return the updated grantee
// list; grant/revoke are idempotent server-side, and a subject owned by someone
// else returns 404. The grant body key is `userId`.

export function listGameDefinitionShares(token: string, id: string): Promise<GameDefinitionSharesResponse> {
  return request<GameDefinitionSharesResponse>(`/game-definitions/${encodeURIComponent(id)}/shares`, {
    headers: authHeaders(token),
  })
}

export function grantGameDefinitionShare(token: string, id: string, userId: string): Promise<GameDefinitionSharesResponse> {
  return request<GameDefinitionSharesResponse>(`/game-definitions/${encodeURIComponent(id)}/shares`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify({ userId }),
  })
}

export function revokeGameDefinitionShare(token: string, id: string, granteeId: string): Promise<GameDefinitionSharesResponse> {
  return request<GameDefinitionSharesResponse>(
    `/game-definitions/${encodeURIComponent(id)}/shares/${encodeURIComponent(granteeId)}`,
    { method: 'DELETE', headers: authHeaders(token) },
  )
}

export function listGameCollectionShares(token: string, id: string): Promise<GameCollectionSharesResponse> {
  return request<GameCollectionSharesResponse>(`/game-collections/${encodeURIComponent(id)}/shares`, {
    headers: authHeaders(token),
  })
}

export function grantGameCollectionShare(token: string, id: string, userId: string): Promise<GameCollectionSharesResponse> {
  return request<GameCollectionSharesResponse>(`/game-collections/${encodeURIComponent(id)}/shares`, {
    method: 'PUT',
    headers: authHeaders(token),
    body: JSON.stringify({ userId }),
  })
}

export function revokeGameCollectionShare(token: string, id: string, granteeId: string): Promise<GameCollectionSharesResponse> {
  return request<GameCollectionSharesResponse>(
    `/game-collections/${encodeURIComponent(id)}/shares/${encodeURIComponent(granteeId)}`,
    { method: 'DELETE', headers: authHeaders(token) },
  )
}

export interface UserLookupQuery {
  username: string
  limit?: number
  offset?: number
}

// Looks up users whose username starts with `username` (case-insensitive) for
// the share people-picker. A blank prefix returns an empty page — the server
// never enumerates every user. Returns only id + username per hit.
export function lookupUsers(token: string, query: UserLookupQuery): Promise<UserLookupResponse> {
  const params = new URLSearchParams()
  params.set('username', query.username)
  if (query.limit != null) params.set('limit', String(query.limit))
  if (query.offset != null) params.set('offset', String(query.offset))
  return request<UserLookupResponse>(`/users/lookup?${params.toString()}`, {
    headers: authHeaders(token),
  })
}
