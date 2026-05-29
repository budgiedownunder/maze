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
}

export interface SaveMazeRequest {
  name: string
  definition: MazeDefinition
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
