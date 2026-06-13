// Scoreboard vocabulary + helpers shared across the score API client and the
// leaderboard UI. The metric / direction string unions are the single
// as-const source (the server accepts the same query-string values); the
// pure DTO types (`ScoreEntry`, `ScoreBoardResponse`) live in `types/api.ts`
// and re-export the unions declared here.

// The metric a leaderboard ranks by — sent as the `metric` query param.
export const SCORE_METRICS = ['time', 'score'] as const
export type ScoreMetric = (typeof SCORE_METRICS)[number]

// The primary metric's sort direction — sent as the `direction` query param.
export const SORT_DIRECTIONS = ['asc', 'desc'] as const
export type SortDirection = (typeof SORT_DIRECTIONS)[number]

// Canonical form of a curated-challenge subject: "<difficulty>:<seed>". This
// is the single source for the convention on the TypeScript side. The vanilla
// game host (`public/game/index.html`) can't import this module, so it forms
// the same string inline against this definition.
export function buildChallenge(difficulty: string, seed: number): string {
  return `${difficulty}:${seed}`
}
