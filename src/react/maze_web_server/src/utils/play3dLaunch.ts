import {
  saveMazeGameSettings,
  type MazeGameSettings,
} from './mazeGameSettings'

/// Persists the user's chosen launch settings (so the modal pre-fills
/// them next time) and navigates to `/game/?id=…`. The host page
/// (`public/game/index.html`) reads the same localStorage key to build
/// the StartConfig it sends to the wasm boundary.
export function launchPlay3dWithSettings(mazeId: string, settings: MazeGameSettings): void {
  saveMazeGameSettings(settings)
  window.location.href = '/game/?id=' + encodeURIComponent(mazeId)
}
