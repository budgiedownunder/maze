import '@testing-library/jest-dom'
import { afterEach, beforeAll, afterAll } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './src/mocks/server'
import { resetMockEmails, resetMockMazes, resetMockTokens } from './src/mocks/handlers'

// jsdom does not implement matchMedia — provide a light-mode default stub.
// Guard against Node environment (integration tests use @vitest-environment node).
if (typeof window !== 'undefined') Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
afterEach(() => {
  server.resetHandlers()
  resetMockEmails()
  resetMockMazes()
  resetMockTokens()
  cleanup()
})
afterAll(() => server.close())
