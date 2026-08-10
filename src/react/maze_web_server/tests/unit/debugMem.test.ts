import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

// `DEBUG_MEM` is resolved from `import.meta.env` at module load, so each test
// stubs the variable and re-imports the module to observe the effect. Testing
// `withDebugMem` on both sides of the flag is the point: the disabled path is
// what every ordinary build ships, and it must leave launch URLs untouched.
async function loadWithFlag(value: string | undefined) {
  vi.resetModules()
  if (value === undefined) {
    vi.stubEnv('VITE_DEBUG_MEM', '')
  } else {
    vi.stubEnv('VITE_DEBUG_MEM', value)
  }
  return await import('../../src/utils/debugMem')
}

describe('debugMem', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  afterEach(() => {
    vi.unstubAllEnvs()
    vi.resetModules()
  })

  it('leaves URLs untouched in an ordinary build', async () => {
    const { DEBUG_MEM, withDebugMem } = await loadWithFlag(undefined)
    expect(DEBUG_MEM).toBe(false)
    expect(withDebugMem('/game/?id=abc')).toBe('/game/?id=abc')
    expect(withDebugMem('/game/')).toBe('/game/')
  })

  it('ignores any value other than the literal "true"', async () => {
    // Vite exposes env vars as strings, so a truthy-looking "1" must not count —
    // only what the dev:debug / build:debug scripts actually set.
    const { DEBUG_MEM, withDebugMem } = await loadWithFlag('1')
    expect(DEBUG_MEM).toBe(false)
    expect(withDebugMem('/game/?id=abc')).toBe('/game/?id=abc')
  })

  it('appends mem=1 with an ampersand when the URL already has a query', async () => {
    const { DEBUG_MEM, withDebugMem } = await loadWithFlag('true')
    expect(DEBUG_MEM).toBe(true)
    expect(withDebugMem('/game/?id=abc')).toBe('/game/?id=abc&mem=1')
    expect(withDebugMem('/game/?def=g1')).toBe('/game/?def=g1&mem=1')
    expect(withDebugMem('/game/?preview=1')).toBe('/game/?preview=1&mem=1')
  })

  it('appends mem=1 with a question mark when the URL has no query', async () => {
    const { withDebugMem } = await loadWithFlag('true')
    expect(withDebugMem('/game/')).toBe('/game/?mem=1')
  })
})
