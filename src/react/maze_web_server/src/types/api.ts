export interface UserProfile {
  id: string
  username: string
  full_name: string
  email: string
  is_admin: boolean
}

export interface LoginResponse {
  token: string
  issued_at: string
  expiry: string
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
