// Vocabulary shared across the game-definition / collection API client and UI.
// The visibility / rotation string unions are the single as-const source; the
// server accepts and emits the same lowercase wire values (see data_model's
// `Visibility` / `Rotation`).

// Access tier of a game definition or collection, in ascending-openness order:
// "private" = owner only, "shared" = explicit grantees, "public" = any signed-in
// user, "curated" = admin-featured.
export const VISIBILITIES = ['private', 'shared', 'public', 'curated'] as const
export type Visibility = (typeof VISIBILITIES)[number]

// User-facing label for a visibility tier, matching the wording of the Access
// picker (Just me / Specific people / Everyone / Featured). Used for the
// read-only access badge on the workshop games list.
const ACCESS_LABELS: Record<Visibility, string> = {
  private: 'Just me',
  shared: 'Specific people',
  public: 'Everyone',
  curated: 'Featured',
}

export function accessLabel(visibility: Visibility): string {
  return ACCESS_LABELS[visibility]
}

// Confirm-dialog body for a layout reshuffle. A reshuffle changes the generated
// maze, so a definition that already has scores loses its (now-incomparable)
// leaderboard — say so more strongly when scores exist. Shared by the editor's
// in-tab Reshuffle action and the workshop list's per-row Reshuffle.
export function reshuffleConfirmMessage(hasScores: boolean): string {
  return hasScores
    ? "This generates a new maze layout, replacing the current one, and permanently clears this game's leaderboard — every recorded score was set on the old layout. This can't be undone."
    : "This generates a new maze layout for the game, replacing the current one. This can't be undone."
}

// Whether a definition's layout — and thus its leaderboard — is fixed ("static")
// or rotates by UTC date ("daily", a daily challenge).
export const ROTATIONS = ['static', 'daily'] as const
export type Rotation = (typeof ROTATIONS)[number]

// ── Multi-level run vocabulary ──────────────────────────────────────────────
// The three enum-valued fields of a definition's `levels` config. Lowercase wire
// values mirror the server's `FinishTypeConfig` / `DifficultyChangeConfig` /
// `LayeredAlignmentConfig`; the runtime degrades an unrecognised value to the
// first (default) entry of each.

// Interim-finish transition rig between stacked levels.
export const FINISH_TYPES = ['ladder', 'portal', 'random'] as const
export type FinishType = (typeof FINISH_TYPES)[number]

// How difficulty shifts as the player ascends the level stack.
export const DIFFICULTY_CHANGES = ['same', 'easier', 'harder'] as const
export type DifficultyChange = (typeof DIFFICULTY_CHANGES)[number]

// How a reduced upper level is positioned over the level below it.
export const LEVEL_ALIGNMENTS = ['edge', 'centre', 'random_base', 'random_level'] as const
export type LevelAlignment = (typeof LEVEL_ALIGNMENTS)[number]

// Upper bound on a multi-level run's level count. Mirrors `MAX_LEVEL_COUNT` in
// `src/rust/maze_game_bevy/src/world/levels.rs`; the runtime clamps `count` to
// `[1, MAX_LEVEL_COUNT]` regardless, and the editor uses it to bound the input.
export const MAX_LEVEL_COUNT = 20
