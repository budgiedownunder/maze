import { http, HttpResponse } from 'msw'
import type { LoginResponse, RenewResponse, UpdateProfileRequest, UserProfile } from '../types/api'

const BASE = '/api/v1'

export const mockProfile: UserProfile = {
  id: '00000000-0000-0000-0000-000000000001',
  username: 'testuser',
  full_name: 'Test User',
  email: 'test@example.com',
  is_admin: false,
}

export const mockLoginResponse: LoginResponse = {
  login_token_id: 'aaaaaaaa-0000-0000-0000-000000000001',
  login_token_expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
}

export const handlers = [
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
]
