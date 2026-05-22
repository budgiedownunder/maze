export function isValidEmail(email: string): boolean {
  return /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)
}

// Maximum number of doors a generated maze may be seeded with. Mirrors
// `MAX_AUTO_DOORS` in `src/rust/maze/src/generator.rs` (kept at 8 so keys+doors
// stay within the key-aware solver's verification bound). The generator clamps
// to this regardless; the Generate dialog uses it to bound the Doors input.
export const MAX_DOOR_COUNT = 8

// Returns true when the rows × cols product would exceed the server-reported
// store cap. A null cap means the configured store imposes no cap, in which
// case this always returns false.
export function exceedsMazeCellCap(rows: number, cols: number, cap: number | null): boolean {
  if (cap === null) return false
  return rows * cols > cap
}

export function validateSignupForm(fields: {
  email: string
  password: string
  confirmPassword: string
}): string | null {
  if (!fields.email.trim() || !fields.password || !fields.confirmPassword) {
    return 'All fields are required'
  }
  if (!isValidEmail(fields.email)) {
    return 'Please enter a valid email address'
  }
  if (fields.password !== fields.confirmPassword) {
    return 'Passwords do not match'
  }
  if (fields.password.length < 8) {
    return 'Password must be at least 8 characters'
  }
  if (!/[A-Z]/.test(fields.password)) return 'Password must contain an uppercase letter'
  if (!/[a-z]/.test(fields.password)) return 'Password must contain a lowercase letter'
  if (!/[0-9]/.test(fields.password)) return 'Password must contain a digit'
  if (!/[^A-Za-z0-9]/.test(fields.password)) return 'Password must contain a special character'
  return null
}
