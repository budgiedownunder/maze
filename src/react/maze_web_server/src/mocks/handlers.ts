import { http, HttpResponse } from 'msw'
import type { AddUserEmailRequest, AppFeatures, LoginResponse, Maze, RenewResponse, UpdateProfileRequest, UserEmail, UserEmailsResponse, UserProfile } from '../types/api'

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

export let mockMazes: Maze[] = [mockMazeAlpha, mockMazeBeta]

export function resetMockMazes(): void {
  mockMazes = [mockMazeAlpha, mockMazeBeta]
}

export const handlers = [
  http.get(`${BASE}/features`, () => {
    return HttpResponse.json<AppFeatures>({ allow_signup: true, oauth_providers: [] })
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

  http.post(`${BASE}/users/me/emails/:email/verify`, () => {
    return new HttpResponse('Email verification flow is not yet implemented', { status: 501 })
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
]
