import {
  saveMazeGameSettings,
  type MazeGameSettings,
} from './mazeGameSettings'

/// Stashes the effective launch settings in localStorage — the transient
/// handoff the `/game/` host reads — and navigates to `/game/?id=…`. The host
/// page (`public/game/index.html`) reads the same localStorage key to build
/// the StartConfig it sends to the wasm boundary.
export function launchPlay3dWithSettings(mazeId: string, settings: MazeGameSettings): void {
  saveMazeGameSettings(settings)
  window.location.href = '/game/?id=' + encodeURIComponent(mazeId)
}
