import type { ChangePasswordRequest, LoginResponse, RenewResponse, UpdateProfileRequest, UserProfile } from '../types/api'

const BASE = '/api/v1'

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE}${path}`, options)
  if (!response.ok) {
    throw Object.assign(new Error(response.statusText), { status: response.status })
  }
  return response.json() as Promise<T>
}

async function requestEmpty(path: string, options?: RequestInit): Promise<void> {
  const response = await fetch(`${BASE}${path}`, options)
  if (!response.ok) {
    throw Object.assign(new Error(response.statusText), { status: response.status })
  }
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

export async function deleteMe(token: string): Promise<void> {
  await fetch(`${BASE}/users/me`, {
    method: 'DELETE',
    headers: authHeaders(token),
  })
}
