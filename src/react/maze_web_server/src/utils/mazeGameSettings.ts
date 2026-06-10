// Per-launch customisation values for the Play 3D button on user-edited
// mazes. The modal at `components/MazeGameSettingsModal.tsx` writes
// the user's chosen values to localStorage on Play; the host page at
// `public/game/index.html` reads the same key to build the StartConfig
// it sends to the wasm boundary.

import {
  DOOR_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  KEY_HOLDER_STYLES,
} from './cellEntityStyles'
import type {
  DoorStyle,
  EnemyType,
  HealthStyle,
  KeyHolderStyle,
} from '../types/cellEntities'

// Sky and wall textures are 3D-scene decor specific to the Play 3D launch; the
// entity rig styles (door / key-holder / enemy / health) are the shared cell-entity
// vocabulary and live in `cellEntities.ts`.
export const SKY_TYPES = ['night', 'sunrise', 'day', 'sunset', 'dungeon', 'chamber'] as const
export type SkyType = (typeof SKY_TYPES)[number]

// The four solid textures plus the three non-occluding types (water / lava /
// iron fence) — a whole maze can be any of them.
export const WALL_TYPES = [
  'brick',
  'dressed_stone',
  'wood',
  'cobblestone',
  'water',
  'lava',
  'iron_fence',
] as const
export type WallType = (typeof WALL_TYPES)[number]

export interface MazeGameSettings {
  skyType: SkyType
  wallType: WallType
  /** Whether the maze perimeter is walled at the grid edge under an open sky.
   * Enclosed skies (dungeon / chamber) always wall it regardless. */
  perimeterWalls: boolean
  doorStyle: DoorStyle
  keyHolder: KeyHolderStyle
  enemyType: EnemyType
  healthStyle: HealthStyle
  wallTint: boolean
  wallMaterialVariation: boolean
  deadEndObjects: boolean
  wallDecorations: boolean
  floorAccents: boolean
  timerSeconds: number
}

export const MAZE_GAME_SETTINGS_DEFAULTS: MazeGameSettings = {
  skyType: 'night',
  wallType: 'brick',
  // The maze is walled at its perimeter by default (even under an open sky).
  perimeterWalls: true,
  // Door / key-holder styles default to the topology-driven swing and the
  // stone pedestal — the look the 3D game shipped with.
  doorStyle: 'swing',
  keyHolder: 'pedestal',
  // Enemy / health styles default to the same variants the Bevy crate's
  // `EnemyType::default()` / `HealthStyle::default()` and the server's
  // `EnemyTypeConfig::default()` / `HealthStyleConfig::default()` use.
  enemyType: 'goblin',
  healthStyle: 'heart',
  // Match the prior hard-coded "clean look" overrides for user-edited
  // mazes — `wall_tint` and `wall_material_variation` off so the user's
  // layout is the visual focus by default. The user can still flip
  // them on per-launch.
  wallTint: false,
  wallMaterialVariation: false,
  // Other landmarks default on, matching `Landmarks::default()` in the
  // Bevy crate.
  deadEndObjects: true,
  wallDecorations: true,
  floorAccents: true,
  // 60 seconds, matching `GameConfig::default().timer_seconds`.
  timerSeconds: 60,
}

export const MAZE_GAME_SETTINGS_STORAGE_KEY = 'mazeGameSettings'

/// Loads the user's last-used custom launch settings from localStorage,
/// or returns the defaults if nothing is stored or the stored value is
/// invalid. Validates enums; falls back to the default value for any
/// stored field that doesn't match the current schema.
export function loadMazeGameSettings(): MazeGameSettings {
  try {
    const raw = localStorage.getItem(MAZE_GAME_SETTINGS_STORAGE_KEY)
    if (!raw) return MAZE_GAME_SETTINGS_DEFAULTS
    const parsed = JSON.parse(raw) as Partial<MazeGameSettings>
    const skyType: SkyType = (SKY_TYPES as readonly string[]).includes(parsed.skyType ?? '')
      ? (parsed.skyType as SkyType)
      : MAZE_GAME_SETTINGS_DEFAULTS.skyType
    const wallType: WallType = (WALL_TYPES as readonly string[]).includes(parsed.wallType ?? '')
      ? (parsed.wallType as WallType)
      : MAZE_GAME_SETTINGS_DEFAULTS.wallType
    const doorStyle: DoorStyle = (DOOR_STYLES as readonly string[]).includes(parsed.doorStyle ?? '')
      ? (parsed.doorStyle as DoorStyle)
      : MAZE_GAME_SETTINGS_DEFAULTS.doorStyle
    const keyHolder: KeyHolderStyle = (KEY_HOLDER_STYLES as readonly string[]).includes(
      parsed.keyHolder ?? '',
    )
      ? (parsed.keyHolder as KeyHolderStyle)
      : MAZE_GAME_SETTINGS_DEFAULTS.keyHolder
    const enemyType: EnemyType = (ENEMY_TYPES as readonly string[]).includes(parsed.enemyType ?? '')
      ? (parsed.enemyType as EnemyType)
      : MAZE_GAME_SETTINGS_DEFAULTS.enemyType
    const healthStyle: HealthStyle = (HEALTH_STYLES as readonly string[]).includes(
      parsed.healthStyle ?? '',
    )
      ? (parsed.healthStyle as HealthStyle)
      : MAZE_GAME_SETTINGS_DEFAULTS.healthStyle
    const timer = Number(parsed.timerSeconds)
    return {
      skyType,
      wallType,
      perimeterWalls: parsed.perimeterWalls ?? MAZE_GAME_SETTINGS_DEFAULTS.perimeterWalls,
      doorStyle,
      keyHolder,
      enemyType,
      healthStyle,
      wallTint: parsed.wallTint ?? MAZE_GAME_SETTINGS_DEFAULTS.wallTint,
      wallMaterialVariation:
        parsed.wallMaterialVariation ?? MAZE_GAME_SETTINGS_DEFAULTS.wallMaterialVariation,
      deadEndObjects: parsed.deadEndObjects ?? MAZE_GAME_SETTINGS_DEFAULTS.deadEndObjects,
      wallDecorations: parsed.wallDecorations ?? MAZE_GAME_SETTINGS_DEFAULTS.wallDecorations,
      floorAccents: parsed.floorAccents ?? MAZE_GAME_SETTINGS_DEFAULTS.floorAccents,
      timerSeconds:
        Number.isFinite(timer) && timer > 0 ? timer : MAZE_GAME_SETTINGS_DEFAULTS.timerSeconds,
    }
  } catch {
    return MAZE_GAME_SETTINGS_DEFAULTS
  }
}

export function saveMazeGameSettings(settings: MazeGameSettings): void {
  try {
    localStorage.setItem(MAZE_GAME_SETTINGS_STORAGE_KEY, JSON.stringify(settings))
  } catch {
    /* localStorage unavailable / quota — ignore; the launch still works
       via the in-memory settings the user just submitted. */
  }
}
