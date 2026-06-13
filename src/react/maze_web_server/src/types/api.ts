import type { CanonicalMazeDefinition } from './cellEntities'
import type { MazeGameSettings } from '../utils/mazeGameSettings'
// Re-export score metrics so consumers can pull the DTO types and
// the query vocabulary from one place.
export type { ScoreMetric, SortDirection } from '../utils/scores'

export interface UserEmail {
  email: string
  is_primary: boolean
  verified: boolean
  verified_at: string | null
}

export interface UserProfile {
  id: string
  username: string
  full_name: string
  email: string
  emails: UserEmail[]
  is_admin: boolean
  has_password: boolean
}

export interface UserEmailsResponse {
  emails: UserEmail[]
}

export interface AddUserEmailRequest {
  email: string
}

export interface LoginResponse {
  login_token_id: string
  login_token_expires_at: string
  is_first_sign_in: boolean
}

export interface RenewResponse {
  login_token_id: string
  login_token_expires_at: string
}

export interface UpdateProfileRequest {
  username: string
  full_name: string
}

export interface ChangePasswordRequest {
  current_password?: string
  new_password: string
}

export interface MazeDefinition {
  grid: string[][]
}

export interface Maze {
  id: string
  name: string
  definition: MazeDefinition
  game_settings?: MazeGameSettings
}

export interface SaveMazeRequest {
  name: string
  // The saved definition is the canonical char-or-array form (overridden cells carry
  // an entity array), so it accepts overrides as well as a plain-char grid.
  definition: CanonicalMazeDefinition
  // Persisted per-maze 3D game settings; omitted when the maze has none.
  game_settings?: MazeGameSettings
}

export interface OAuthProviderPublic {
  name: string         // canonical: "google" | "github" | ...
  display_name: string // user-facing label rendered on the button
}

export interface AppFeatures {
  allow_signup: boolean
  oauth_providers: OAuthProviderPublic[]
  email_enabled: boolean
  max_maze_cells: number | null
}

export interface GenerateOptions {
  rowCount: number
  colCount: number
  startRow: number     // 1-based (UI convention)
  startCol: number     // 1-based
  finishRow: number    // 1-based
  finishCol: number    // 1-based
  minSpineLength: number
  doorCount: number    // number of real path doors (each with one key) to auto-place; 0 = none
  spareDoors: number   // number of decoy doors planted on off-spine branches; 0 = none
  spareKeys: number    // number of spare keys planted on off-spine branches; 0 = none
  enemyCount: number   // number of enemy cells to auto-place at random passable cells; 0 = none
  healthCount: number  // number of health-pickup cells to auto-place at random passable cells; 0 = none
}

// A recorded run, as returned by the score endpoints. Mirrors the server's
// `ScoreResponse` (snake_case keys; exactly one of `maze_id` / `challenge` is
// set). `recorded_at` is an RFC 3339 timestamp string.
export interface ScoreEntry {
  id: string
  user_id: string
  maze_id: string | null
  challenge: string | null
  score: number
  elapsed_ms: number
  recorded_at: string
}

// A page of a leaderboard or personal history. Mirrors the server's
// `ScoreBoardResponse`: `limit` is the effective (server-capped) page size and
// `has_more` says whether a further page exists.
export interface ScoreBoardResponse {
  scores: ScoreEntry[]
  limit: number
  offset: number
  has_more: boolean
}
