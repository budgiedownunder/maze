import type { CellEntity, EnemyType, HealthStyle, WallType } from '../types/cellEntities'
import type { MazeGameSettings } from './mazeGameSettings'

// Resolves a grid cell (its char + optional per-cell override) to the sprite to draw.
// Shared by the editor grid and the 2D game so the char-or-override → sprite mapping
// lives in one place. Enemy/health carry distinct 2D variants (ghost/potion), and the
// special wall types (water/lava/iron_fence) have their own 2D sprites; key/door rigs
// and the solid wall textures are a 3D-only concern, so they render their generic sprite.
//
// A cell with no per-cell visual override falls back to the maze's `game_settings`
// default (wallType / enemyType / healthStyle), so a maze authored as e.g. all-lava or
// ghost enemies shows that look in 2D. A per-cell override always wins over the maze
// default, and the default still resolves to the generic sprite when it has no distinct
// 2D variant (the solid wall textures, the goblin / heart rigs).

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
 * exists (enemy `enemyType`, health `healthStyle`, wall `wallType`) and otherwise the
 * maze's `game_settings` default for that family. Returns the generic sprite for a cell
 * whose effective variant has no distinct 2D sprite (the goblin / heart rigs, the solid
 * wall textures), or a rig with no 2D variant at all (key/door); `null` for an
 * empty/unknown cell. A per-cell override always wins over the maze default.
 */
export function cellSprite(
  char: string,
  entity?: CellEntity,
  settings?: MazeGameSettings,
): CellImage | null {
  const base = BASE[char] ?? null
  if (!base) return base
  // Per-cell override field wins; otherwise inherit the maze default for this family.
  if (char === 'E') {
    const enemyType = (entity?.type === 'E' ? entity.enemyType : undefined) ?? settings?.enemyType
    const src = enemyType ? ENEMY_VARIANT_SPRITES[enemyType] : undefined
    if (src) return { src, alt: base.alt }
  } else if (char === 'H') {
    const healthStyle =
      (entity?.type === 'H' ? entity.healthStyle : undefined) ?? settings?.healthStyle
    const src = healthStyle ? HEALTH_VARIANT_SPRITES[healthStyle] : undefined
    if (src) return { src, alt: base.alt }
  } else if (char === 'W') {
    const wallType = (entity?.type === 'W' ? entity.wallType : undefined) ?? settings?.wallType
    const src = wallType ? WALL_VARIANT_SPRITES[wallType] : undefined
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
