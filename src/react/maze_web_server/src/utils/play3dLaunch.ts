import {
  saveMazeGameSettings,
  type MazeGameSettings,
} from './mazeGameSettings'
import { withDebugMem } from './debugMem'

/// Stashes the effective launch settings in localStorage — the transient
/// handoff the `/game/` host reads — and navigates to `/game/?id=…`. The host
/// page (`public/game/index.html`) reads the same localStorage key to build
/// the StartConfig it sends to the wasm boundary.
export function launchPlay3dWithSettings(mazeId: string, settings: MazeGameSettings): void {
  saveMazeGameSettings(settings)
  window.location.href = withDebugMem('/game/?id=' + encodeURIComponent(mazeId))
}

/// Launches a stored game definition by id — the host page fetches the
/// definition and forwards its `config` as the StartConfig. No settings handoff
/// (the config is server-owned, resolved from the definition).
export function launchDefinition(id: string): void {
  window.location.href = withDebugMem('/game/?def=' + encodeURIComponent(id))
}
