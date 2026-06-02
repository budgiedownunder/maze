export function isValidEmail(email: string): boolean {
  return /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email)
}

// Maximum number of doors a generated maze may be seeded with. Mirrors
// `MAX_AUTO_DOORS` in `src/rust/maze/src/generator.rs` (kept at 8 so keys+doors
// stay within the key-aware solver's verification bound). The generator clamps
// to this regardless; the Generate dialog uses it to bound the Doors input.
export const MAX_DOOR_COUNT = 8

// Maximum number of enemy cells a generated maze may be seeded with. Mirrors
// `MAX_ENEMY_COUNT` in `src/rust/maze/src/generator.rs`. The generator clamps
// to this regardless; the Generate dialog uses it to bound the Enemies input.
export const MAX_ENEMY_COUNT = 8

// Maximum number of health-pickup cells a generated maze may be seeded with.
// Mirrors `MAX_HEALTH_COUNT` in `src/rust/maze/src/generator.rs`. The generator
// clamps to this regardless; the Generate dialog uses it to bound the Health input.
export const MAX_HEALTH_COUNT = 8

// Maximum combined number of 'K' + 'D' cells in any maze (saved or generated).
// Mirrors `maze::MAX_TOTAL_FEATURES` on the Rust side. The key-aware solver
// tracks each as a bit in a u32 mask, so its search is exponential in their
// sum and it refuses above this cap; the Generate dialog and the editor save
// flow refuse over-cap requests up front so the server (which validates the
// same rule) never has to reject them.
export const MAX_TOTAL_FEATURES = 16

// Returns true when the rows × cols product would exceed the server-reported
// store cap. A null cap means the configured store imposes no cap, in which
// case this always returns false.
export function exceedsMazeCellCap(rows: number, cols: number, cap: number | null): boolean {
  if (cap === null) return false
  return rows * cols > cap
}

// Counts the 'K' (key) and 'D' (door) cells in a maze grid. Used by both the
// Generate dialog (against the planned door_count + spare_doors + spare_keys
// inputs, with 2 * door_count for the one-K-per-real-door pairing) and the
// editor save flow (against the actual saved grid).
export function countKeysAndDoors(grid: string[][]): { keys: number; doors: number } {
  let keys = 0
  let doors = 0
  for (const row of grid) {
    for (const cell of row) {
      if (cell === 'K') keys++
      else if (cell === 'D') doors++
    }
  }
  return { keys, doors }
}

// Returns true when the saved grid would exceed the K + D cap.
export function exceedsKeyDoorCap(grid: string[][]): boolean {
  const { keys, doors } = countKeysAndDoors(grid)
  return keys + doors > MAX_TOTAL_FEATURES
}

// Returns true when a Generate request's planned features (each real door
// contributes one K and one D, hence 2*) would exceed the cap.
export function exceedsGenerateFeatureCap(
  doorCount: number,
  spareDoors: number,
  spareKeys: number,
): boolean {
  return 2 * doorCount + spareDoors + spareKeys > MAX_TOTAL_FEATURES
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
