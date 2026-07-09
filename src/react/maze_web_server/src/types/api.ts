import type { CanonicalMazeDefinition } from './cellEntities'
import type { MazeGameSettings } from '../utils/mazeGameSettings'
import type { Visibility, Rotation } from '../utils/gameDefinitions'
// Re-export score metrics so consumers can pull the DTO types and
// the query vocabulary from one place.
export type { ScoreMetric, SortDirection } from '../utils/scores'
// Likewise the game-definition access/rotation vocabulary.
export type { Visibility, Rotation } from '../utils/gameDefinitions'

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
  avatar_updated_at?: string | null
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
  treasureCount: number // number of treasure cells to auto-place dead-end-first, type-weighted; 0 = none
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
  username?: string | null
  avatar_updated_at?: string | null
}

// A page of a leaderboard or personal history. Mirrors the server's
// `ScoreboardResponse`: `limit` is the effective (server-capped) page size and
// `has_more` says whether a further page exists.
export interface ScoreboardResponse {
  scores: ScoreEntry[]
  limit: number
  offset: number
  has_more: boolean
}

// Result of resetting a leaderboard (DELETE /scores): the number of score rows
// removed.
export interface ResetScoresResponse {
  deleted: number
}

// The subset of the server's Play3dConfigResponse the client consumes: the
// curated difficulty's fixed maze seed, used to key its leaderboard
// (`challenge = "<difficulty>:<seed>"`).
export interface Play3dConfig {
  difficulty: string
  seed: number
}

// --- Game definitions & collections -----------------------------------------

// A stored 3D game definition — presentation metadata plus an opaque, client-
// owned generation/render `config` (a StartConfig-shaped blob, stored and
// forwarded verbatim). `seed` is server-owned: auto-minted and hidden from the
// editor. `description` / `imageUpdatedAt` are absent when unset.
export interface GameDefinition {
  id: string
  ownerId: string
  name: string
  description?: string
  visibility: Visibility
  seed: number
  rotation: Rotation
  config: Record<string, unknown>
  imageUpdatedAt?: string
  createdAt: string
  updatedAt: string
}

// The play-fetch of a single definition (`GET /game-definitions/{id}`): the
// definition with the effective seed spliced into `config`, plus its leaderboard
// subject key and whether that board is tracked (published definitions only).
export interface GamePlayResponse extends GameDefinition {
  challengeKey: string
  leaderboardTracked: boolean
}

// Create / update body — the caller supplies only editable fields; id, seed,
// ownerId, image and timestamps are server-owned. `visibility` / `rotation`
// default server-side (private / static) when omitted.
export interface GameDefinitionRequest {
  name: string
  description?: string | null
  visibility?: Visibility
  rotation?: Rotation
  config: Record<string, unknown>
}

// A page of the definitions the caller may see (own ∨ shared ∨ public ∨ curated).
export interface GameDefinitionListResponse {
  definitions: GameDefinition[]
  limit: number
  offset: number
  hasMore: boolean
}

// One ordered member of a collection — a reference to a definition by id.
export interface CollectionItem {
  definitionId: string
  sortOrder: number
}

// A collection: an ordered, presentation-only grouping of definitions. It
// carries its own access `visibility`; membership is order-only (`items`).
export interface GameCollection {
  id: string
  ownerId: string
  name: string
  visibility: Visibility
  description?: string
  imageUpdatedAt?: string
  items: CollectionItem[]
  createdAt: string
  updatedAt: string
}

// Create / update body for a collection's own metadata; membership is managed
// via the item endpoints, so it is not part of this body.
export interface GameCollectionRequest {
  name: string
  description?: string | null
  visibility?: Visibility
}

// A page of the collections the caller may see.
export interface GameCollectionListResponse {
  collections: GameCollection[]
  limit: number
  offset: number
  hasMore: boolean
}

// Collection detail (`GET /game-collections/{id}`): the collection's metadata
// plus its member definitions — hydrated, in order, and filtered to what the
// viewer may access (inaccessible members and dangling refs omitted). This is
// why it carries `definitions` rather than the raw `items`.
export interface GameCollectionDetailResponse {
  id: string
  ownerId: string
  name: string
  description?: string
  visibility: Visibility
  imageUpdatedAt?: string
  createdAt: string
  updatedAt: string
  definitions: GameDefinition[]
}

// --- Sharing & user lookup --------------------------------------------------

// The grantee list returned by the definition / collection share endpoints — a
// bare list of user ids (the server's `DefinitionSharesResponse` /
// `CollectionSharesResponse`, which are structurally identical). Resolve ids to
// usernames via the lookup when a grantee needs a display name.
export interface SharesResponse {
  grantees: string[]
}

// A single username-prefix lookup hit for the share people-picker. The server
// returns only id + username here — never email, admin flag, or avatar.
export interface UserLookupEntry {
  id: string
  username: string
}

// A page of username-prefix lookup hits. Mirrors the server's
// `UserLookupResponse`, which — unlike the other paged lists — is NOT
// camelCase-renamed, so the last-page flag is the snake_case `has_more` (as in
// `ScoreboardResponse`).
export interface UserLookupResponse {
  users: UserLookupEntry[]
  limit: number
  offset: number
  has_more: boolean
}
