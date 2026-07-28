// Scoreboard vocabulary + helpers shared across the score API client and the
// leaderboard UI. The metric / direction string unions are the single
// as-const source (the server accepts the same query-string values); the
// pure DTO types (`ScoreEntry`, `ScoreboardResponse`) live in `types/api.ts`
// and re-export the unions declared here.

// The metric a leaderboard ranks by — sent as the `metric` query param.
export const SCORE_METRICS = ['time', 'score'] as const
export type ScoreMetric = (typeof SCORE_METRICS)[number]

// The primary metric's sort direction — sent as the `direction` query param.
export const SORT_DIRECTIONS = ['asc', 'desc'] as const
export type SortDirection = (typeof SORT_DIRECTIONS)[number]

// Formats an elapsed-run duration as "m:ss.mmm" (e.g. 42137 → "0:42.137"),
// matching the Bevy win-overlay format.
export function formatElapsedMs(ms: number): string {
  const total = Math.max(0, Math.floor(ms))
  const minutes = Math.floor(total / 60000)
  const seconds = Math.floor((total % 60000) / 1000)
  const millis = total % 1000
  return `${minutes}:${String(seconds).padStart(2, '0')}.${String(millis).padStart(3, '0')}`
}
