// The 3D game settings for a user-edited maze (sky, walls, rig styles,
// landmarks, timer). Persisted per-maze (carried on `Maze.game_settings`) and
// edited via `components/MazeGameSettingsModal.tsx`. On launch,
// `saveMazeGameSettings` stashes the effective settings in localStorage and the
// host page at `public/game/index.html` reads the same key to build the
// StartConfig it sends to the wasm boundary.

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

// Sky and wall textures are 3D-scene decor specific to the 3D game; the
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
  // them on in the maze's settings.
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

/// Validates + fills a partial settings object against the current schema,
/// substituting the default for any field that is missing or doesn't match.
/// The single source of truth for turning untrusted/partial settings (from
/// localStorage or a persisted maze) into a complete, valid object.
export function normalizeMazeGameSettings(parsed: Partial<MazeGameSettings>): MazeGameSettings {
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
}

export function saveMazeGameSettings(settings: MazeGameSettings): void {
  try {
    localStorage.setItem(MAZE_GAME_SETTINGS_STORAGE_KEY, JSON.stringify(settings))
  } catch {
    /* localStorage unavailable / quota — ignore; the launch still works
       via the in-memory settings the user just submitted. */
  }
}
