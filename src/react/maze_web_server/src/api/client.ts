import type { AddUserEmailRequest, AppFeatures, ChangePasswordRequest, LoginResponse, Maze, Play3dConfig, RenewResponse, SaveMazeRequest, ScoreBoardResponse, ScoreMetric, SortDirection, UpdateProfileRequest, UserEmailsResponse, UserProfile } from '../types/api'

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
export function getLeaderboard(token: string, query: LeaderboardQuery): Promise<ScoreBoardResponse> {
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
  return request<ScoreBoardResponse>(`/scores?${params.toString()}`, {
    headers: authHeaders(token),
  })
}

// Reads a page of the authenticated player's own run history (most recent first).
export function getScoreHistory(token: string, query: HistoryQuery = {}): Promise<ScoreBoardResponse> {
  const params = new URLSearchParams()
  if (query.limit != null) params.set('limit', String(query.limit))
  if (query.offset != null) params.set('offset', String(query.offset))
  const qs = params.toString()
  return request<ScoreBoardResponse>(`/scores/me${qs ? `?${qs}` : ''}`, {
    headers: authHeaders(token),
  })
}

// Reads a curated difficulty's preset (unauthenticated). The leaderboard UI
// uses its fixed `seed` to build the challenge board key.
export function getPlay3dConfig(difficulty: string): Promise<Play3dConfig> {
  return request<Play3dConfig>(`/game/play3d-config?difficulty=${encodeURIComponent(difficulty)}`)
}
