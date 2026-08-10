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

// Maximum number of treasure cells a generated maze may be seeded with. Mirrors
// `MAX_TREASURE_COUNT` in `src/rust/maze/src/generator.rs`. The generator clamps
// to this regardless; the Generate dialog uses it to bound the Treasure input.
export const MAX_TREASURE_COUNT = 12

// Maximum combined number of 'K' + 'D' cells in any maze (saved or generated).
// Mirrors `maze::MAX_TOTAL_FEATURES` on the Rust side. The key-aware solver
// tracks each as a bit in a u32 mask, so its search is exponential in their
// sum and it refuses above this cap; the Generate dialog and the editor save
// flow refuse over-cap requests up front so the server (which validates the
// same rule) never has to reject them.
export const MAX_TOTAL_FEATURES = 16

// Per-field caps for the two "spare" (decoy) feature counts. Spare doors are
// clamped to the same door cap as real doors on the Rust side
// (`place_spare_keys_and_doors` → `MAX_AUTO_DOORS`), so they share
// `MAX_DOOR_COUNT`. Spare keys carry no door constraint — they are bounded only
// by the combined `MAX_TOTAL_FEATURES` budget — so a spare key may in principle
// use the whole budget.
export const MAX_SPARE_DOOR_COUNT = MAX_DOOR_COUNT
export const MAX_SPARE_KEY_COUNT = MAX_TOTAL_FEATURES

// Largest number of rows or columns a 3D game definition may be authored with.
// A judgement about play rather than a limit the renderer imposes: a 40x40 floor
// renders but is tedious to walk. Applies to game definitions only — an authored
// 2D maze is bounded by the store's cell cap alone.
//
// Set above the largest shipped preset rather than at it, so a curated game is
// not the ceiling an authored one is measured against.
//
// Deliberately has no Rust twin. `GameDefinition.config` is opaque to the server,
// and clamping rows/cols at generation time would change what a given seed
// produces, silently altering a shared game's layout and invalidating its
// leaderboard — unlike `MAX_LEVEL_COUNT`, where clamping merely plays fewer
// levels. Existing over-cap definitions therefore keep their size until edited.
export const MAX_GAME_MAZE_DIMENSION = 30

// The advanced HP pair, as raw input strings.
//
// A blank starting value is valid — it means "start at full health" — but a
// blank *cap* is not: it stores as 0, and the game computes its starting HP as
// `clamp(1, maxHp)`, which panics outright when the cap is below 1. So the cap
// is checked first and on its own terms, rather than being ignored as
// unparseable.
export function validateHpFields(startingHp: string, maxHp: string): string | null {
  const max = parseInt(maxHp, 10)
  if (!Number.isInteger(max) || max < 1) return 'Max HP must be a whole number of 1 or more.'
  if (startingHp.trim() === '') return null
  const starting = parseInt(startingHp, 10)
  if (!Number.isInteger(starting) || starting < 1) return 'Starting HP must be a whole number of 1 or more.'
  if (starting > max) return 'Starting HP cannot exceed Max HP.'
  return null
}

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

// Validates the parametric generation fields shared by the maze Generate dialog
// and the game-definition editor — rows/cols/minSolutionLength plus the
// door/spare/enemy/health/treasure counts. Returns the first error message, or
// null when all valid. Reuses the same caps + feature-budget rule so neither
// path can ask for a maze the generator or solver would reject.
//
// `kind` selects whether the start/finish positions are checked: a maze is
// authored on a concrete grid (`'maze'` → positions required + in-bounds +
// distinct), whereas a game definition is generated from a seed with no grid
// (`'game'` → positions ignored). The position checks run in the same order and
// with the same messages the Generate dialog used inline. `kind` also sets the
// `minSolutionLength` floor: a game accepts 0 (no minimum, as the server does),
// while an authored maze needs a solution of at least one step.
export function validateMazeGenerationFields(
  v: {
    rows: string
    cols: string
    minSolutionLength: string
    doorCount: string
    spareDoors: string
    spareKeys: string
    enemyCount: string
    healthCount: string
    treasureCount: string
    startRow?: string
    startCol?: string
    finishRow?: string
    finishCol?: string
  },
  maxMazeCells: number | null,
  kind: 'maze' | 'game',
): string | null {
  const rows = parseInt(v.rows, 10)
  const cols = parseInt(v.cols, 10)
  const msl = parseInt(v.minSolutionLength, 10)
  const doors = parseInt(v.doorCount, 10)
  const sdoors = parseInt(v.spareDoors, 10)
  const skeys = parseInt(v.spareKeys, 10)
  const enemies = parseInt(v.enemyCount, 10)
  const healths = parseInt(v.healthCount, 10)
  const treasures = parseInt(v.treasureCount, 10)

  if (!Number.isInteger(rows) || rows < 3) return 'Rows must be a whole number of 3 or more.'
  if (!Number.isInteger(cols) || cols < 3) return 'Columns must be a whole number of 3 or more.'
  if (kind === 'game') {
    if (rows > MAX_GAME_MAZE_DIMENSION) return `Rows cannot exceed ${MAX_GAME_MAZE_DIMENSION}.`
    if (cols > MAX_GAME_MAZE_DIMENSION) return `Columns cannot exceed ${MAX_GAME_MAZE_DIMENSION}.`
  }
  if (exceedsMazeCellCap(rows, cols, maxMazeCells)) {
    return `Total cells (rows × columns) cannot exceed ${maxMazeCells}.`
  }
  if (kind === 'maze') {
    const sr = parseInt(v.startRow ?? '', 10)
    const sc = parseInt(v.startCol ?? '', 10)
    const fr = parseInt(v.finishRow ?? '', 10)
    const fc = parseInt(v.finishCol ?? '', 10)
    if (!Number.isInteger(sr) || sr < 1 || sr > rows) return `Start Row must be between 1 and ${rows}.`
    if (!Number.isInteger(sc) || sc < 1 || sc > cols) return `Start Column must be between 1 and ${cols}.`
    if (!Number.isInteger(fr) || fr < 1 || fr > rows) return `Finish Row must be between 1 and ${rows}.`
    if (!Number.isInteger(fc) || fc < 1 || fc > cols) return `Finish Column must be between 1 and ${cols}.`
    if (sr === fr && sc === fc) return 'Start and Finish cells must be different.'
  }
  const minMsl = kind === 'maze' ? 1 : 0
  if (!Number.isInteger(msl) || msl < minMsl) {
    return `Min Start to Finish Distance must be a whole number of ${minMsl} or more.`
  }
  if (!Number.isInteger(doors) || doors < 0 || doors > MAX_DOOR_COUNT) {
    return `Doors must be a whole number between 0 and ${MAX_DOOR_COUNT}.`
  }
  if (!Number.isInteger(sdoors) || sdoors < 0 || sdoors > MAX_SPARE_DOOR_COUNT) {
    return `Spare Doors must be a whole number between 0 and ${MAX_SPARE_DOOR_COUNT}.`
  }
  if (!Number.isInteger(skeys) || skeys < 0 || skeys > MAX_SPARE_KEY_COUNT) {
    return `Spare Keys must be a whole number between 0 and ${MAX_SPARE_KEY_COUNT}.`
  }
  if (!Number.isInteger(enemies) || enemies < 0 || enemies > MAX_ENEMY_COUNT) {
    return `Enemies must be a whole number between 0 and ${MAX_ENEMY_COUNT}.`
  }
  if (!Number.isInteger(healths) || healths < 0 || healths > MAX_HEALTH_COUNT) {
    return `Health must be a whole number between 0 and ${MAX_HEALTH_COUNT}.`
  }
  if (!Number.isInteger(treasures) || treasures < 0 || treasures > MAX_TREASURE_COUNT) {
    return `Treasure must be a whole number between 0 and ${MAX_TREASURE_COUNT}.`
  }
  if (exceedsGenerateFeatureCap(doors, sdoors, skeys)) {
    const total = 2 * doors + sdoors + skeys
    return (
      `Total keys + doors (${total}) exceeds the limit of ${MAX_TOTAL_FEATURES}. ` +
      `Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.`
    )
  }
  return null
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
