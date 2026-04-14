import { http, HttpResponse } from 'msw'
import type { LoginResponse, Maze, RenewResponse, UpdateProfileRequest, UserProfile } from '../types/api'

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
