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

// Longer phrasing for the visibility marker's tooltip / accessible name.
const ACCESS_DESCRIPTIONS: Record<Visibility, string> = {
  private: 'Only you can see this game',
  shared: 'Shared with specific people',
  public: 'Visible to everyone',
  curated: 'Featured for everyone',
}

export function accessDescription(visibility: Visibility): string {
  return ACCESS_DESCRIPTIONS[visibility]
}

// The gameplay-affecting projection of a config — the config minus the cosmetic
// keys the server also ignores when deciding whether an edit resets the board
// (splash `title`, status-bar `mode`, the server-owned `seed`, and
// `levels.hideCompletedEnemies`). Two configs with the same projection play the
// same, so an edit between them doesn't invalidate the leaderboard.
function gameplaySignature(config: Record<string, unknown>): string {
  const clone = JSON.parse(JSON.stringify(config ?? {})) as Record<string, unknown>
  delete clone.title
  delete clone.mode
  delete clone.seed
  const levels = clone.levels
  if (levels && typeof levels === 'object') delete (levels as Record<string, unknown>).hideCompletedEnemies
  return JSON.stringify(clone)
}

// Whether an edit changes how the game plays — any non-cosmetic config field
// differs. Mirrors the server's board-reset rule so the editor can warn before a
// save that would wipe the leaderboard.
export function isGameplayChange(before: Record<string, unknown>, after: Record<string, unknown>): boolean {
  return gameplaySignature(before) !== gameplaySignature(after)
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

// User-facing label + one-line description for each rotation, shown by the
// definition editor's Rotation control.
const ROTATION_LABELS: Record<Rotation, string> = {
  static: 'Static',
  daily: 'Daily',
}

const ROTATION_DESCRIPTIONS: Record<Rotation, string> = {
  static: 'One fixed layout with a single, permanent leaderboard.',
  daily: 'A fresh layout and leaderboard each day (UTC).',
}

export function rotationLabel(rotation: Rotation): string {
  return ROTATION_LABELS[rotation]
}

export function rotationDescription(rotation: Rotation): string {
  return ROTATION_DESCRIPTIONS[rotation]
}

// Today's date in UTC as `yyyy-mm-dd` — the day boundary the server uses for
// Daily boards (`compute_play_subject` mixes the UTC date).
export function todayUtc(): string {
  return new Date().toISOString().slice(0, 10)
}

// The leaderboard challenge key for a game, matching the server's play-fetch
// subject (game_definitions.rs `compute_play_subject`): `def:<id>` for a Static
// game, `def:<id>:<yyyy-mm-dd>` for a Daily one. `dateUtc` selects which day's
// board for a Daily game (defaults to today, UTC); it is ignored for Static.
// Used to check per-game completion (a score on this key) for campaign progress
// and to key the leaderboard board.
export function gameChallengeKey(id: string, rotation: Rotation = 'static', dateUtc?: string): string {
  return rotation === 'daily' ? `def:${id}:${dateUtc ?? todayUtc()}` : `def:${id}`
}

// The game id behind a `def:<id>` (or Daily `def:<id>:<yyyy-mm-dd>`) leaderboard
// challenge, or null for any other challenge namespace (e.g. a legacy
// `<difficulty>:<seed>` board). The inverse of [gameChallengeKey], mirroring the
// server's `parse_definition_challenge`; lets a score row's challenge be resolved
// back to the game it was set on.
export function gameIdFromChallenge(challenge: string): string | null {
  if (!challenge.startsWith('def:')) return null
  const id = challenge.slice('def:'.length).split(':')[0]
  return id === '' ? null : id
}

// How a collection is played once opened: "arcade" = free choice (pick any
// member game), "campaign" = an ordered progression through the members. The
// lowercase wire values mirror the server's `data_model::PlayMode`.
export const PLAY_MODES = ['arcade', 'campaign'] as const
export type PlayMode = (typeof PLAY_MODES)[number]

// User-facing label + one-line description for each play mode, shown by the
// collection editor's Play mode control.
const PLAY_MODE_LABELS: Record<PlayMode, string> = {
  arcade: 'Arcade',
  campaign: 'Campaign',
}

const PLAY_MODE_DESCRIPTIONS: Record<PlayMode, string> = {
  arcade: 'Players pick any game in the collection to play.',
  campaign: 'Players work through the games in order, unlocking the next as they go.',
}

export function playModeLabel(mode: PlayMode): string {
  return PLAY_MODE_LABELS[mode]
}

export function playModeDescription(mode: PlayMode): string {
  return PLAY_MODE_DESCRIPTIONS[mode]
}

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
