import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { launchDefinitionPreview } from '../../src/utils/definitionPreview'

const KEY = 'gameDefinitionPreview'

describe('launchDefinitionPreview', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.stubGlobal('open', vi.fn())
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('stashes {config, seeded} in localStorage and opens /game/?preview=1 in a new tab', () => {
    const config = { rows: 8, cols: 8, seed: 0 }
    launchDefinitionPreview(config, false)

    const stored = JSON.parse(localStorage.getItem(KEY)!)
    expect(stored).toEqual({ config, seeded: false })
    expect(window.open).toHaveBeenCalledWith('/game/?preview=1', '_blank')
  })

  it('records the seeded flag for a saved definition', () => {
    launchDefinitionPreview({ rows: 5, cols: 5 }, true)
    expect(JSON.parse(localStorage.getItem(KEY)!).seeded).toBe(true)
  })

  it('still opens the tab even if localStorage write throws', () => {
    const spy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('quota') })
    expect(() => launchDefinitionPreview({ rows: 3, cols: 3 }, false)).not.toThrow()
    expect(window.open).toHaveBeenCalledWith('/game/?preview=1', '_blank')
    spy.mockRestore()
  })
})
