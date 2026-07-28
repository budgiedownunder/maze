// localStorage key the `/game/` host page reads to launch an editor preview.
// Mirrors `PREVIEW_STORAGE_KEY` in public/game/index.html — keep them in sync.
const PREVIEW_STORAGE_KEY = 'gameDefinitionPreview'

// One-off, non-persisted preview of an in-progress game-definition config. Stashes
// the config (the Bevy `StartConfig` blob that `buildDefinitionConfig` produces)
// plus whether the definition is already seeded (saved) in localStorage, then
// opens the `/game/` host in a NEW TAB (`?preview=1`) so the editor's unsaved
// state survives; the new tab inherits the opener's sessionStorage auth. The host
// reads the payload, starts Bevy directly (no fetch, no score submission), and —
// when `seeded` is false — shows a banner noting the layout is indicative.
export function launchDefinitionPreview(config: Record<string, unknown>, seeded: boolean): void {
  try {
    localStorage.setItem(PREVIEW_STORAGE_KEY, JSON.stringify({ config, seeded }))
  } catch {
    /* storage disabled / quota — the host shows a "no preview" message. */
  }
  window.open('/game/?preview=1', '_blank')
}
