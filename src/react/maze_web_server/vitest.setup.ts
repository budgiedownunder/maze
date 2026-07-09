import '@testing-library/jest-dom'
import { afterEach, beforeAll, afterAll } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './src/mocks/server'
import { resetMockAvatar, resetMockEmails, resetMockMazes, resetMockTokens } from './src/mocks/handlers'

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

// jsdom doesn't implement object URLs — the <Avatar> uses them to render a
// fetched image. A plain stub is enough; tests that assert on the value set
// their own vi.fn.
if (typeof URL !== 'undefined') {
  URL.createObjectURL = () => 'blob:mock'
  URL.revokeObjectURL = () => {}
}

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
afterEach(async () => {
  server.resetHandlers()
  resetMockEmails()
  resetMockMazes()
  resetMockTokens()
  resetMockAvatar()
  // Lazy import so this setup file doesn't eagerly load the image cache (and
  // its real `client` dependency) before a test's `vi.mock('../api/client')`
  // registers — which would make the cache call the real fetch.
  const { resetImageCache } = await import('./src/utils/imageCache')
  resetImageCache()
  cleanup()
})
afterAll(() => server.close())
