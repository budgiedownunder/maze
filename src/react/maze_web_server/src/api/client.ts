import type { ChangePasswordRequest, LoginResponse, Maze, MazeDefinition, RenewResponse, SaveMazeRequest, UpdateProfileRequest, UserProfile } from '../types/api'

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

export function login(username: string, password: string): Promise<LoginResponse> {
  return request<LoginResponse>('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
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

export function signup(
  username: string,
  full_name: string,
  email: string,
  password: string,
): Promise<void> {
  return requestEmpty('/signup', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, full_name, email, password }),
  })
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
    .map(item => ({
      id: item.id,
      name: item.name,
      // definition is the full Maze JSON string: {id, name, definition: {grid:[...]}}
      definition: item.definition
        ? (JSON.parse(item.definition) as { definition: MazeDefinition }).definition
        : { grid: [] },
    }))
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
