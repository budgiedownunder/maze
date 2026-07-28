// Pure (de)serialization between the game-definition editor's form state and the
// opaque `config` blob a definition stores. The config is the Bevy `StartConfig`
// shape (camelCase; every field optional/defaulted on the Bevy side), with the
// generation + scene/objects/decor + advanced + multi-level settings. The editor
// works in a flatter, string-per-numeric-input form; these functions bridge the
// two and supply defaults for anything a stored config is missing.
//
// The config carries no authored maze grid — a definition is generated from its
// seed — and none of the run-time fields the host injects at launch (the
// leaderboard flags / record thresholds).

import {
  MAZE_GAME_SETTINGS_DEFAULTS,
  normalizeMazeGameSettings,
  SKY_TYPES,
  type MazeGameSettings,
  type SkyType,
  type WallType,
} from './mazeGameSettings'
import type { DoorStyle, EnemyType, HealthStyle, KeyHolderStyle } from '../types/cellEntities'
import {
  DIFFICULTY_CHANGES,
  FINISH_TYPES,
  LEVEL_ALIGNMENTS,
  type DifficultyChange,
  type FinishType,
  type LevelAlignment,
} from './gameDefinitions'
import type { MazeGenerationFieldsValue } from '../components/MazeGenerationFields'
import type { SceneFieldsValue, ObjectsFieldsValue, DecorFieldsValue } from '../components/GameSettingsFields'
import type { GameDefinitionRequest, Rotation, Visibility } from '../types/api'

// ── The stored config blob ──────────────────────────────────────────────────

// The config's nested `landmarks` group — the wall-appearance + decorative
// toggles. In the editor these are split across two tabs (Scene owns `wallTint`
// and `wallMaterialVariation`; Decor owns the other three); build folds them
// back together here, and parse splits them out again.
export interface DefinitionLandmarksConfig {
  wallTint: boolean
  deadEndObjects: boolean
  wallDecorations: boolean
  floorAccents: boolean
  wallMaterialVariation: boolean
}

// The `levels.top` field: an optional scene override for the **top (final)
// level** of a multi-level run — the last level the player reaches when the
// levels are stacked. It overrides only that level's `skyType` / `perimeterWalls`;
// a field left absent/null inherits the base game's value, and `null` for the
// whole object (the usual case) means the final level looks like the rest.
export interface DefinitionTopLevelConfig {
  skyType?: SkyType | null
  perimeterWalls?: boolean | null
}

// The config's `levels` group — the multi-level run settings. `count === 1` (the
// default) is a single-level game, and every other field is then inert.
export interface DefinitionLevelsConfig {
  count: number
  finishType: FinishType
  difficultyChange: DifficultyChange
  resetBag: boolean
  alignment: LevelAlignment
  taper: boolean
  perimeterRandom: boolean
  hideCompletedEnemies: boolean
  top: DefinitionTopLevelConfig | null
}

// The whole stored `config` blob: the flattened generation + scene/objects/decor
// + timing/advanced settings, with the nested `landmarks` + `levels` groups. This
// is what the editor produces, a definition stores, and the host forwards to Bevy
// verbatim as its `StartConfig` (see the module header for the shape's origin).
export interface DefinitionConfig {
  rows: number
  cols: number
  timerSeconds: number
  seed: number
  minSolutionLength: number
  minimapCellPx: number
  minimapRadius: number
  title: string
  mode: string
  landmarks: DefinitionLandmarksConfig
  skyType: SkyType
  wallType: WallType
  perimeterWalls: boolean
  doorStyle: DoorStyle
  keyHolder: KeyHolderStyle
  doorCount: number
  spareDoors: number
  spareKeys: number
  enemyCount: number
  healthCount: number
  treasureCount: number
  enemyType: EnemyType
  healthStyle: HealthStyle
  enemyMovePeriodMs: number
  maxHp: number
  levels: DefinitionLevelsConfig
}

// ── The editor's working form state ─────────────────────────────────────────

// Level fields carry strings for the numeric input + loose strings for the enum
// pickers (the level-editor UI tightens the enums to as-const unions).
export interface DefinitionLevelsFormValue {
  count: string
  finishType: FinishType
  difficultyChange: DifficultyChange
  resetBag: boolean
  alignment: LevelAlignment
  taper: boolean
  perimeterRandom: boolean
  hideCompletedEnemies: boolean
  top: DefinitionTopLevelConfig | null
}

// The editor's working state — a flatter view of the config for the form
// controls. Numeric fields are strings (raw input text); the generation /
// scene / objects / decor groups reuse the shared field-group value types. Adds
// the definition-level `name` / `description` and the pass-through fields below.
export interface DefinitionFormState {
  name: string
  description: string
  generation: MazeGenerationFieldsValue
  scene: SceneFieldsValue
  objects: ObjectsFieldsValue
  decor: DecorFieldsValue
  timerSeconds: string
  title: string
  mode: string
  minimapCellPx: string
  minimapRadius: string
  enemyMovePeriodMs: string
  maxHp: string
  levels: DefinitionLevelsFormValue
  // Pass-through: not edited in the content editor (access is managed elsewhere;
  // the seed is auto-minted + hidden), but carried so a Save never resets them.
  visibility: Visibility
  rotation: Rotation
  seed: number
}

// Starting state for a brand-new definition, and the fallback for any config key
// missing on parse. Generation / timing / advanced / levels mirror the server's
// play3d config defaults; scene/objects/decor mirror the maze-settings defaults.
export const DEFINITION_DEFAULTS: DefinitionFormState = {
  name: '',
  description: '',
  generation: {
    rows: '8',
    cols: '8',
    minSolutionLength: '1',
    doorCount: '0',
    spareDoors: '0',
    spareKeys: '0',
    enemyCount: '0',
    healthCount: '0',
    treasureCount: '0',
  },
  scene: {
    skyType: MAZE_GAME_SETTINGS_DEFAULTS.skyType,
    wallType: MAZE_GAME_SETTINGS_DEFAULTS.wallType,
    perimeterWalls: MAZE_GAME_SETTINGS_DEFAULTS.perimeterWalls,
    wallTint: MAZE_GAME_SETTINGS_DEFAULTS.wallTint,
    wallMaterialVariation: MAZE_GAME_SETTINGS_DEFAULTS.wallMaterialVariation,
  },
  objects: {
    doorStyle: MAZE_GAME_SETTINGS_DEFAULTS.doorStyle,
    keyHolder: MAZE_GAME_SETTINGS_DEFAULTS.keyHolder,
    enemyType: MAZE_GAME_SETTINGS_DEFAULTS.enemyType,
    healthStyle: MAZE_GAME_SETTINGS_DEFAULTS.healthStyle,
  },
  decor: {
    deadEndObjects: MAZE_GAME_SETTINGS_DEFAULTS.deadEndObjects,
    wallDecorations: MAZE_GAME_SETTINGS_DEFAULTS.wallDecorations,
    floorAccents: MAZE_GAME_SETTINGS_DEFAULTS.floorAccents,
  },
  timerSeconds: '120',
  title: '',
  mode: '',
  minimapCellPx: '10',
  minimapRadius: '5',
  enemyMovePeriodMs: '1500',
  maxHp: '3',
  levels: {
    count: '1',
    finishType: 'ladder',
    difficultyChange: 'easier',
    resetBag: true,
    alignment: 'edge',
    taper: false,
    perimeterRandom: false,
    hideCompletedEnemies: false,
    top: null,
  },
  visibility: 'private',
  rotation: 'static',
  seed: 0,
}

// ── Build: form state → config + request ────────────────────────────────────

// A non-negative integer from an input string; validation runs before build, so
// a malformed value only ever falls back to 0 as a safety net.
function toInt(s: string): number {
  const n = parseInt(s, 10)
  return Number.isFinite(n) ? n : 0
}

/**
 * Assembles the stored `config` blob and the `GameDefinitionRequest` from the
 * editor's form state. Scene/decor toggles fold into the nested `landmarks`
 * object; numeric input strings become numbers.
 */
export function buildDefinitionConfig(form: DefinitionFormState): {
  config: DefinitionConfig
  request: GameDefinitionRequest
} {
  const g = form.generation
  const config: DefinitionConfig = {
    rows: toInt(g.rows),
    cols: toInt(g.cols),
    timerSeconds: toInt(form.timerSeconds),
    seed: form.seed,
    minSolutionLength: toInt(g.minSolutionLength),
    minimapCellPx: toInt(form.minimapCellPx),
    minimapRadius: toInt(form.minimapRadius),
    title: form.title,
    mode: form.mode,
    landmarks: {
      wallTint: form.scene.wallTint,
      deadEndObjects: form.decor.deadEndObjects,
      wallDecorations: form.decor.wallDecorations,
      floorAccents: form.decor.floorAccents,
      wallMaterialVariation: form.scene.wallMaterialVariation,
    },
    skyType: form.scene.skyType,
    wallType: form.scene.wallType,
    perimeterWalls: form.scene.perimeterWalls,
    doorStyle: form.objects.doorStyle,
    keyHolder: form.objects.keyHolder,
    doorCount: toInt(g.doorCount),
    spareDoors: toInt(g.spareDoors),
    spareKeys: toInt(g.spareKeys),
    enemyCount: toInt(g.enemyCount),
    healthCount: toInt(g.healthCount),
    treasureCount: toInt(g.treasureCount),
    enemyType: form.objects.enemyType,
    healthStyle: form.objects.healthStyle,
    enemyMovePeriodMs: toInt(form.enemyMovePeriodMs),
    maxHp: toInt(form.maxHp),
    levels: {
      count: toInt(form.levels.count),
      finishType: form.levels.finishType,
      difficultyChange: form.levels.difficultyChange,
      resetBag: form.levels.resetBag,
      alignment: form.levels.alignment,
      taper: form.levels.taper,
      perimeterRandom: form.levels.perimeterRandom,
      hideCompletedEnemies: form.levels.hideCompletedEnemies,
      top: form.levels.top,
    },
  }

  const request: GameDefinitionRequest = {
    name: form.name,
    // An empty description is stored as "unset" rather than an empty string.
    description: form.description.trim() === '' ? null : form.description,
    visibility: form.visibility,
    rotation: form.rotation,
    // Widen the concrete config to the request's opaque config type.
    config: config as unknown as Record<string, unknown>,
  }

  return { config, request }
}

// ── Parse: config + meta → form state ───────────────────────────────────────

// The name / description / visibility / rotation live on the definition, not in
// the config blob, so `parse` takes them separately.
export interface DefinitionMeta {
  name: string
  description?: string | null
  visibility: Visibility
  rotation: Rotation
}

function numStr(v: unknown, fallback: string): string {
  return typeof v === 'number' && Number.isFinite(v) ? String(v) : fallback
}
function numOr(v: unknown, fallback: number): number {
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback
}
function strOr(v: unknown, fallback: string): string {
  return typeof v === 'string' ? v : fallback
}
function boolOr(v: unknown, fallback: boolean): boolean {
  return typeof v === 'boolean' ? v : fallback
}
function objOr(v: unknown): Record<string, unknown> {
  return typeof v === 'object' && v !== null ? (v as Record<string, unknown>) : {}
}
// Coerces an unknown to one of a restricted set, falling back when it is not a
// recognised member (mirrors how the runtime degrades an unknown enum value).
function oneOf<T extends string>(v: unknown, allowed: readonly T[], fallback: T): T {
  return typeof v === 'string' && (allowed as readonly string[]).includes(v) ? (v as T) : fallback
}
function parseTop(v: unknown): DefinitionTopLevelConfig | null {
  if (typeof v !== 'object' || v === null) return null
  const t = v as Record<string, unknown>
  const top: DefinitionTopLevelConfig = {}
  // Only set an override when it is a recognised value; anything else inherits.
  if (typeof t.skyType === 'string' && (SKY_TYPES as readonly string[]).includes(t.skyType)) {
    top.skyType = t.skyType as SkyType
  }
  if (typeof t.perimeterWalls === 'boolean') top.perimeterWalls = t.perimeterWalls
  return top
}

/**
 * Hydrates the editor form state from a stored (possibly partial or malformed)
 * config blob, filling any missing key from {@link DEFINITION_DEFAULTS}. The
 * style/scene/objects/decor enums are coerced through `normalizeMazeGameSettings`
 * so an unrecognised value degrades to its safe default rather than sticking.
 */
export function parseDefinitionConfig(config: Record<string, unknown>, meta: DefinitionMeta): DefinitionFormState {
  const landmarks = objOr(config.landmarks)
  const levels = objOr(config.levels)
  const d = DEFINITION_DEFAULTS

  // `normalizeMazeGameSettings` is built to accept untrusted input (it validates
  // each enum + defaults the rest); its parameter type is stricter than that, so
  // the cast is deliberate. Timer is handled separately (definition default 120,
  // not the maze-settings 60).
  const settings = normalizeMazeGameSettings({
    skyType: config.skyType,
    wallType: config.wallType,
    perimeterWalls: config.perimeterWalls,
    doorStyle: config.doorStyle,
    keyHolder: config.keyHolder,
    enemyType: config.enemyType,
    healthStyle: config.healthStyle,
    wallTint: landmarks.wallTint,
    wallMaterialVariation: landmarks.wallMaterialVariation,
    deadEndObjects: landmarks.deadEndObjects,
    wallDecorations: landmarks.wallDecorations,
    floorAccents: landmarks.floorAccents,
  } as unknown as Partial<MazeGameSettings>)

  return {
    name: meta.name,
    description: meta.description ?? '',
    generation: {
      rows: numStr(config.rows, d.generation.rows),
      cols: numStr(config.cols, d.generation.cols),
      minSolutionLength: numStr(config.minSolutionLength, d.generation.minSolutionLength),
      doorCount: numStr(config.doorCount, d.generation.doorCount),
      spareDoors: numStr(config.spareDoors, d.generation.spareDoors),
      spareKeys: numStr(config.spareKeys, d.generation.spareKeys),
      enemyCount: numStr(config.enemyCount, d.generation.enemyCount),
      healthCount: numStr(config.healthCount, d.generation.healthCount),
      treasureCount: numStr(config.treasureCount, d.generation.treasureCount),
    },
    scene: {
      skyType: settings.skyType,
      wallType: settings.wallType,
      perimeterWalls: settings.perimeterWalls,
      wallTint: settings.wallTint,
      wallMaterialVariation: settings.wallMaterialVariation,
    },
    objects: {
      doorStyle: settings.doorStyle,
      keyHolder: settings.keyHolder,
      enemyType: settings.enemyType,
      healthStyle: settings.healthStyle,
    },
    decor: {
      deadEndObjects: settings.deadEndObjects,
      wallDecorations: settings.wallDecorations,
      floorAccents: settings.floorAccents,
    },
    timerSeconds: numStr(config.timerSeconds, d.timerSeconds),
    title: strOr(config.title, d.title),
    mode: strOr(config.mode, d.mode),
    minimapCellPx: numStr(config.minimapCellPx, d.minimapCellPx),
    minimapRadius: numStr(config.minimapRadius, d.minimapRadius),
    enemyMovePeriodMs: numStr(config.enemyMovePeriodMs, d.enemyMovePeriodMs),
    maxHp: numStr(config.maxHp, d.maxHp),
    levels: {
      count: numStr(levels.count, d.levels.count),
      finishType: oneOf(levels.finishType, FINISH_TYPES, d.levels.finishType),
      difficultyChange: oneOf(levels.difficultyChange, DIFFICULTY_CHANGES, d.levels.difficultyChange),
      resetBag: boolOr(levels.resetBag, d.levels.resetBag),
      alignment: oneOf(levels.alignment, LEVEL_ALIGNMENTS, d.levels.alignment),
      taper: boolOr(levels.taper, d.levels.taper),
      perimeterRandom: boolOr(levels.perimeterRandom, d.levels.perimeterRandom),
      hideCompletedEnemies: boolOr(levels.hideCompletedEnemies, d.levels.hideCompletedEnemies),
      top: parseTop(levels.top),
    },
    visibility: meta.visibility,
    rotation: meta.rotation,
    seed: numOr(config.seed, d.seed),
  }
}
