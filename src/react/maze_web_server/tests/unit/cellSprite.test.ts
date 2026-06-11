import { describe, it, expect } from 'vitest'
import { cellSprite, enemyRigHasSprite } from '../../src/utils/cellSprite'
import { MAZE_GAME_SETTINGS_DEFAULTS } from '../../src/utils/mazeGameSettings'

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

  it('returns the variant sprite for the special wall types', () => {
    expect(cellSprite('W', { type: 'W', wallType: 'water' })).toEqual({ src: '/images/maze/water.svg', alt: 'Wall' })
    expect(cellSprite('W', { type: 'W', wallType: 'lava' })).toEqual({ src: '/images/maze/lava.svg', alt: 'Wall' })
    expect(cellSprite('W', { type: 'W', wallType: 'iron_fence' })).toEqual({ src: '/images/maze/iron_fence.svg', alt: 'Wall' })
  })

  it('returns the generic wall for a solid-texture override (texture is a 3D concern)', () => {
    expect(cellSprite('W', { type: 'W', wallType: 'brick' })?.src).toBe('/images/maze/wall.png')
    expect(cellSprite('W', { type: 'W', wallType: 'cobblestone' })?.src).toBe('/images/maze/wall.png')
  })
})

describe('cellSprite with maze game settings', () => {
  const lavaMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, wallType: 'lava' as const }
  const ghostMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, enemyType: 'ghost' as const }
  const potionMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, healthStyle: 'potion' as const }

  it('uses the maze default for a cell with no per-cell override', () => {
    expect(cellSprite('W', undefined, lavaMaze)?.src).toBe('/images/maze/lava.svg')
    expect(cellSprite('E', undefined, ghostMaze)?.src).toBe('/images/maze/ghost.svg')
    expect(cellSprite('H', undefined, potionMaze)?.src).toBe('/images/maze/potion.svg')
  })

  it('uses the maze default for an override with no matching visual field', () => {
    expect(cellSprite('E', { type: 'E', damage: 3 }, ghostMaze)?.src).toBe('/images/maze/ghost.svg')
  })

  it('lets a per-cell override win over the maze default', () => {
    // A water override beats a lava maze; a goblin override beats a ghost maze.
    expect(cellSprite('W', { type: 'W', wallType: 'water' }, lavaMaze)?.src).toBe('/images/maze/water.svg')
    expect(cellSprite('E', { type: 'E', enemyType: 'goblin' }, ghostMaze)?.src).toBe('/images/maze/enemy.svg')
  })

  it('falls back to the generic sprite when the maze default has no distinct 2D sprite', () => {
    const solidMaze = { ...MAZE_GAME_SETTINGS_DEFAULTS, wallType: 'dressed_stone' as const }
    expect(cellSprite('W', undefined, solidMaze)?.src).toBe('/images/maze/wall.png')
    expect(cellSprite('E', undefined, MAZE_GAME_SETTINGS_DEFAULTS)?.src).toBe('/images/maze/enemy.svg')
    expect(cellSprite('H', undefined, MAZE_GAME_SETTINGS_DEFAULTS)?.src).toBe('/images/maze/health.svg')
  })
})

describe('enemyRigHasSprite', () => {
  it('is true only for a rig with a distinct 2D sprite (ghost)', () => {
    expect(enemyRigHasSprite('ghost')).toBe(true)
    expect(enemyRigHasSprite('goblin')).toBe(false)
    expect(enemyRigHasSprite(undefined)).toBe(false)
  })
})
