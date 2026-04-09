export interface UserProfile {
  id: string
  username: string
  full_name: string
  email: string
  is_admin: boolean
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
  email: string
}

export interface ChangePasswordRequest {
  current_password: string
  new_password: string
}
