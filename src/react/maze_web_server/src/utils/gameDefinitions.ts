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
