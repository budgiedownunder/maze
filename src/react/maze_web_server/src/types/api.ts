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
}

export interface GenerateOptions {
  rowCount: number
  colCount: number
  startRow: number     // 1-based (UI convention)
  startCol: number     // 1-based
  finishRow: number    // 1-based
  finishCol: number    // 1-based
  minSpineLength: number
}
