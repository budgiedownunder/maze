// Vocabulary shared across the game-definition / collection API client and UI.
// The visibility / rotation string unions are the single as-const source; the
// server accepts and emits the same lowercase wire values (see data_model's
// `Visibility` / `Rotation`).

// Access tier of a game definition or collection, in ascending-openness order:
// "private" = owner only, "shared" = explicit grantees, "public" = any signed-in
// user, "curated" = admin-featured.
export const VISIBILITIES = ['private', 'shared', 'public', 'curated'] as const
export type Visibility = (typeof VISIBILITIES)[number]

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
