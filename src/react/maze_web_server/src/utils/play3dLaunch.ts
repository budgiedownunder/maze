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

/// Launches a curated Play-3D game by difficulty — the server resolves the
/// fixed preset/seed. No per-maze settings handoff (presets are server-owned).
export function launchPlay3dCurated(difficulty: string): void {
  window.location.href = '/game/?difficulty=' + encodeURIComponent(difficulty)
}
