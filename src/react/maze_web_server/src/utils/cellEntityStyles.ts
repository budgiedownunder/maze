// Single source of truth for the per-type rig style vocabularies: each `as const`
// value list and the union derived from it via `(typeof X)[number]`, so the array and
// its type can never drift. Plus a wire-string display helper. The structural
// cell-entity types that build on these unions live in `types/cellEntities.ts`, which
// re-exports the unions so every cell-entity type is still importable from one place.
//
// The string values are byte-identical to the `data_model` Rust enum wire values and
// the C# `CellEntityInfo` rig enums, so the same vocabulary holds end-to-end.

export const ENEMY_TYPES = ['goblin', 'ghost'] as const
export type EnemyType = (typeof ENEMY_TYPES)[number]

export const HEALTH_STYLES = ['heart', 'potion'] as const
export type HealthStyle = (typeof HEALTH_STYLES)[number]

export const KEY_HOLDER_STYLES = ['pedestal', 'chest', 'floating_key'] as const
export type KeyHolderStyle = (typeof KEY_HOLDER_STYLES)[number]

export const DOOR_STYLES = ['swing', 'slide', 'portcullis', 'dissolve'] as const
export type DoorStyle = (typeof DOOR_STYLES)[number]

export const WALL_SOLID_TEXTURES = ['brick', 'dressed_stone', 'wood', 'cobblestone'] as const
export const WALL_SPECIAL_TYPES = ['water', 'lava', 'iron_fence'] as const
export const WALL_TYPES = [...WALL_SOLID_TEXTURES, ...WALL_SPECIAL_TYPES] as const
export type WallType = (typeof WALL_TYPES)[number]

/**
 * Title-cases a wire string for display, replacing underscores with spaces (so e.g.
 * `dressed_stone` reads as "Dressed Stone" and `floating_key` as "Floating Key").
 */
export function titleCaseWire(s: string): string {
  return s
    .replace(/_/g, ' ')
    .split(' ')
    .map(w => (w.length === 0 ? w : w.charAt(0).toUpperCase() + w.slice(1)))
    .join(' ')
}
