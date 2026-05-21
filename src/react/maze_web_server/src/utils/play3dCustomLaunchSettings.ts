// Per-launch customisation values for the Play 3D button on user-edited
// mazes. The modal at `components/Play3dCustomLaunchModal.tsx` writes
// the user's chosen values to localStorage on Play; the host page at
// `public/game/index.html` reads the same key to build the StartConfig
// it sends to the wasm boundary.

export const SKY_TYPES = ['night', 'sunrise', 'day', 'sunset'] as const
export type SkyType = (typeof SKY_TYPES)[number]

export const WALL_TYPES = ['brick', 'dressed_stone', 'wood', 'cobblestone'] as const
export type WallType = (typeof WALL_TYPES)[number]

export const DOOR_STYLES = ['swing', 'slide', 'portcullis', 'dissolve'] as const
export type DoorStyle = (typeof DOOR_STYLES)[number]

export const KEY_HOLDER_STYLES = ['pedestal', 'chest', 'floating_key'] as const
export type KeyHolderStyle = (typeof KEY_HOLDER_STYLES)[number]

export interface Play3dCustomLaunchSettings {
  skyType: SkyType
  wallType: WallType
  doorStyle: DoorStyle
  keyHolder: KeyHolderStyle
  wallTint: boolean
  wallMaterialVariation: boolean
  deadEndObjects: boolean
  wallDecorations: boolean
  floorAccents: boolean
  timerSeconds: number
}

export const PLAY3D_CUSTOM_LAUNCH_DEFAULTS: Play3dCustomLaunchSettings = {
  skyType: 'night',
  wallType: 'brick',
  // Door / key-holder styles default to the topology-driven swing and the
  // stone pedestal — the look the 3D game shipped with.
  doorStyle: 'swing',
  keyHolder: 'pedestal',
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

export const PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY = 'play3dCustomLaunchSettings'

/// Loads the user's last-used custom launch settings from localStorage,
/// or returns the defaults if nothing is stored or the stored value is
/// invalid. Validates enums; falls back to the default value for any
/// stored field that doesn't match the current schema.
export function loadPlay3dCustomLaunchSettings(): Play3dCustomLaunchSettings {
  try {
    const raw = localStorage.getItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY)
    if (!raw) return PLAY3D_CUSTOM_LAUNCH_DEFAULTS
    const parsed = JSON.parse(raw) as Partial<Play3dCustomLaunchSettings>
    const skyType: SkyType = (SKY_TYPES as readonly string[]).includes(parsed.skyType ?? '')
      ? (parsed.skyType as SkyType)
      : PLAY3D_CUSTOM_LAUNCH_DEFAULTS.skyType
    const wallType: WallType = (WALL_TYPES as readonly string[]).includes(parsed.wallType ?? '')
      ? (parsed.wallType as WallType)
      : PLAY3D_CUSTOM_LAUNCH_DEFAULTS.wallType
    const doorStyle: DoorStyle = (DOOR_STYLES as readonly string[]).includes(parsed.doorStyle ?? '')
      ? (parsed.doorStyle as DoorStyle)
      : PLAY3D_CUSTOM_LAUNCH_DEFAULTS.doorStyle
    const keyHolder: KeyHolderStyle = (KEY_HOLDER_STYLES as readonly string[]).includes(
      parsed.keyHolder ?? '',
    )
      ? (parsed.keyHolder as KeyHolderStyle)
      : PLAY3D_CUSTOM_LAUNCH_DEFAULTS.keyHolder
    const timer = Number(parsed.timerSeconds)
    return {
      skyType,
      wallType,
      doorStyle,
      keyHolder,
      wallTint: parsed.wallTint ?? PLAY3D_CUSTOM_LAUNCH_DEFAULTS.wallTint,
      wallMaterialVariation:
        parsed.wallMaterialVariation ?? PLAY3D_CUSTOM_LAUNCH_DEFAULTS.wallMaterialVariation,
      deadEndObjects: parsed.deadEndObjects ?? PLAY3D_CUSTOM_LAUNCH_DEFAULTS.deadEndObjects,
      wallDecorations: parsed.wallDecorations ?? PLAY3D_CUSTOM_LAUNCH_DEFAULTS.wallDecorations,
      floorAccents: parsed.floorAccents ?? PLAY3D_CUSTOM_LAUNCH_DEFAULTS.floorAccents,
      timerSeconds:
        Number.isFinite(timer) && timer > 0 ? timer : PLAY3D_CUSTOM_LAUNCH_DEFAULTS.timerSeconds,
    }
  } catch {
    return PLAY3D_CUSTOM_LAUNCH_DEFAULTS
  }
}

export function savePlay3dCustomLaunchSettings(settings: Play3dCustomLaunchSettings): void {
  try {
    localStorage.setItem(PLAY3D_CUSTOM_LAUNCH_STORAGE_KEY, JSON.stringify(settings))
  } catch {
    /* localStorage unavailable / quota — ignore; the launch still works
       via the in-memory settings the user just submitted. */
  }
}

/// Title-cases a wire string for display, replacing underscores with
/// spaces (so e.g. `dressed_stone` reads as "Dressed Stone" in the
/// modal's dropdowns).
export function titleCaseWire(s: string): string {
  return s
    .replace(/_/g, ' ')
    .split(' ')
    .map(w => (w.length === 0 ? w : w.charAt(0).toUpperCase() + w.slice(1)))
    .join(' ')
}
