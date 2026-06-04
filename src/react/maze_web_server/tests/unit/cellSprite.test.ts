import { describe, it, expect } from 'vitest'
import { cellSprite } from '../../src/utils/cellSprite'

describe('cellSprite', () => {
  it('returns null for an empty or unknown cell', () => {
    expect(cellSprite(' ')).toBeNull()
    expect(cellSprite('?')).toBeNull()
  })

  it('returns the generic sprite for a plain feature cell', () => {
    expect(cellSprite('E')).toEqual({ src: '/images/maze/enemy.svg', alt: 'Enemy' })
    expect(cellSprite('H')).toEqual({ src: '/images/maze/health.svg', alt: 'Health' })
    expect(cellSprite('K')).toEqual({ src: '/images/maze/key.svg', alt: 'Key' })
    expect(cellSprite('D')).toEqual({ src: '/images/maze/door.svg', alt: 'Door' })
    expect(cellSprite('W')).toEqual({ src: '/images/maze/wall.png', alt: 'Wall' })
  })

  it('returns the variant sprite for a ghost enemy override', () => {
    expect(cellSprite('E', { type: 'E', enemyType: 'ghost' })).toEqual({
      src: '/images/maze/ghost.svg',
      alt: 'Enemy',
    })
  })

  it('returns the variant sprite for a potion health override', () => {
    expect(cellSprite('H', { type: 'H', healthStyle: 'potion' })).toEqual({
      src: '/images/maze/potion.svg',
      alt: 'Health',
    })
  })

  it('returns the generic sprite for the default rig (goblin / heart)', () => {
    expect(cellSprite('E', { type: 'E', enemyType: 'goblin' })?.src).toBe('/images/maze/enemy.svg')
    expect(cellSprite('H', { type: 'H', healthStyle: 'heart' })?.src).toBe('/images/maze/health.svg')
  })

  it('returns the generic sprite for an override with no visual field', () => {
    expect(cellSprite('E', { type: 'E', damage: 3 })?.src).toBe('/images/maze/enemy.svg')
  })

  it('returns the generic sprite for key/door rigs (no 2D variant)', () => {
    expect(cellSprite('K', { type: 'K', keyHolder: 'chest' })?.src).toBe('/images/maze/key.svg')
    expect(cellSprite('D', { type: 'D', doorStyle: 'portcullis' })?.src).toBe('/images/maze/door.svg')
  })
})
