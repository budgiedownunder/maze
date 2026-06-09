import type { CellEntity, EnemyType, HealthStyle, WallType } from '../types/cellEntities'

// Resolves a grid cell (its char + optional per-cell override) to the sprite to draw.
// Shared by the editor grid and the 2D game so the char-or-override → sprite mapping
// lives in one place. Enemy/health carry distinct 2D variants (ghost/potion), and the
// special wall types (water/lava/iron_fence) have their own 2D sprites; key/door rigs
// and the solid wall textures are a 3D-only concern, so they render their generic sprite.

export interface CellImage {
  src: string
  alt: string
}

// Generic sprite per cell char. ' ' and unknown chars have no sprite (null).
const BASE: Record<string, CellImage> = {
  W: { src: '/images/maze/wall.png', alt: 'Wall' },
  S: { src: '/images/maze/start_flag.png', alt: 'Start' },
  F: { src: '/images/maze/finish_flag.png', alt: 'Finish' },
  K: { src: '/images/maze/key.svg', alt: 'Key' },
  D: { src: '/images/maze/door.svg', alt: 'Door' },
  E: { src: '/images/maze/enemy.svg', alt: 'Enemy' },
  H: { src: '/images/maze/health.svg', alt: 'Health' },
}

// 2D variant sprites for visual overrides. The default rig (goblin / heart) uses the
// generic BASE sprite, so only the non-default variants need an entry here.
const ENEMY_VARIANT_SPRITES: Partial<Record<EnemyType, string>> = {
  ghost: '/images/maze/ghost.svg',
}
const HEALTH_VARIANT_SPRITES: Partial<Record<HealthStyle, string>> = {
  potion: '/images/maze/potion.svg',
}
// Only the special (non-occluding) wall types get a 2D sprite; the solid textures
// (brick/dressed_stone/wood/cobblestone) render the generic wall (texture is a 3D concern).
const WALL_VARIANT_SPRITES: Partial<Record<WallType, string>> = {
  water: '/images/maze/water.svg',
  lava: '/images/maze/lava.svg',
  iron_fence: '/images/maze/iron_fence.svg',
}

/**
 * The sprite for a cell, honouring a per-cell override's visual variant when one
 * exists (enemy `enemyType`, health `healthStyle`). Returns the generic sprite for a
 * cell with no override, an override with no visual field, or a rig with no 2D variant
 * (key/door); `null` for an empty/unknown cell.
 */
export function cellSprite(char: string, entity?: CellEntity): CellImage | null {
  const base = BASE[char] ?? null
  if (!base || !entity) return base
  if (entity.type === 'E' && entity.enemyType) {
    const src = ENEMY_VARIANT_SPRITES[entity.enemyType]
    if (src) return { src, alt: base.alt }
  } else if (entity.type === 'H' && entity.healthStyle) {
    const src = HEALTH_VARIANT_SPRITES[entity.healthStyle]
    if (src) return { src, alt: base.alt }
  } else if (entity.type === 'W' && entity.wallType) {
    const src = WALL_VARIANT_SPRITES[entity.wallType]
    if (src) return { src, alt: base.alt }
  }
  return base
}

/**
 * Whether an enemy rig has a distinct 2D sprite (e.g. ghost) rather than rendering the
 * default goblin. Used so a cell shared by enemies of differing rigs surfaces the
 * distinctive one.
 */
export function enemyRigHasSprite(enemyType: EnemyType | undefined): boolean {
  return enemyType !== undefined && ENEMY_VARIANT_SPRITES[enemyType] !== undefined
}
