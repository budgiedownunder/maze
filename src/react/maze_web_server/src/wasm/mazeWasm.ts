import init, { MazeWasm, GenerationAlgorithmWasm, MazeGameWasm, DirectionWasm } from 'maze_wasm'

export interface MazeDefinition {
  grid: string[][]
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

let initialized = false

async function ensureInit(): Promise<void> {
  if (!initialized) {
    await init()
    initialized = true
  }
}

/**
 * Generates a new maze using WASM. Options use 1-based row/col (UI convention);
 * conversion to 0-based is done internally before calling the WASM API.
 */
function toError(ex: unknown): Error {
  if (ex instanceof Error) return ex
  return new Error(typeof ex === 'string' ? ex : 'Unknown error.')
}

export async function generateMaze(options: GenerateOptions): Promise<MazeDefinition> {
  await ensureInit()
  const maze = new MazeWasm()
  try {
    try {
      maze.generate(
        options.rowCount,
        options.colCount,
        GenerationAlgorithmWasm.RecursiveBacktracking,
        options.startRow - 1,   // convert 1-based → 0-based
        options.startCol - 1,
        options.finishRow - 1,
        options.finishCol - 1,
        options.minSpineLength,
        100,        // max_retries
        undefined,  // branch_from_finish (use WASM default)
        undefined,  // seed (random)
        options.doorCount,
        options.spareDoors,
        options.spareKeys,
        options.enemyCount,
        options.healthCount,
      )
    } catch (ex) { throw toError(ex) }
    const parsed = JSON.parse(maze.to_json()) as { definition: MazeDefinition }
    return parsed.definition
  } finally {
    maze.free()
  }
}

/**
 * Solves a maze definition using WASM.
 * Returns the solution path as an array of {row, col} points (0-based).
 * Throws if the maze cannot be solved.
 */
export async function solveMaze(definition: MazeDefinition): Promise<Array<{ row: number; col: number }>> {
  await ensureInit()
  const maze = new MazeWasm()
  try {
    try {
      maze.from_json(JSON.stringify({ id: '', name: '', definition }))
      const solution = maze.solve()
      try {
        return solution.get_path_points() as Array<{ row: number; col: number }>
      } finally {
        solution.free()
      }
    } catch (ex) { throw toError(ex) }
  } finally {
    maze.free()
  }
}

// ── Game API ──────────────────────────────────────────────────────────────────

// Integer values match Rust DirectionWasm / C# Direction exactly.
export const MazeGameDirection = {
  None:  0,
  Up:    1,
  Down:  2,
  Left:  3,
  Right: 4,
} as const
export type MazeGameDirection = typeof MazeGameDirection[keyof typeof MazeGameDirection]

// Integer values match Rust MoveResultWasm / C# MoveResult exactly.
export const MazeGamePlayerMoveResult = {
  None:                0,
  Moved:               1,
  Blocked:             2,
  Complete:            3,
  BlockedByLockedDoor: 4,
  StartedUnlocking:    5,
  Stranded:            6,
} as const
export type MazeGamePlayerMoveResult = typeof MazeGamePlayerMoveResult[keyof typeof MazeGamePlayerMoveResult]

// String values match the strings emitted by `MazeGameWasm::lose_reason()` in
// wasm_bindgen.rs. Consumers reference these constants, never the literals.
export const MazeGameLoseReason = {
  Stranded: 'stranded',
} as const
export type MazeGameLoseReason = typeof MazeGameLoseReason[keyof typeof MazeGameLoseReason]

// String values match the objects emitted by wasm_bindgen.rs (and the Rust DoorState /
// GameEvent / BagItem variants). Consumers reference these constants, never the literals.
export const MazeDoorState = {
  Locked:  'locked',
  Opening: 'opening',
  Open:    'open',
} as const
export type MazeDoorState = typeof MazeDoorState[keyof typeof MazeDoorState]

export const MazeGameEventType = {
  DoorOpened: 'doorOpened',
} as const
export type MazeGameEventType = typeof MazeGameEventType[keyof typeof MazeGameEventType]

export const MazeBagItemType = {
  Key: 'key',
} as const
export type MazeBagItemType = typeof MazeBagItemType[keyof typeof MazeBagItemType]

// Object shapes returned by the MazeGameWasm accessors.
export interface MazeDoor { row: number; col: number; state: MazeDoorState }
export interface MazeKeyCell { row: number; col: number; id: number }
export type MazeBagItem = { type: typeof MazeBagItemType.Key; id: number }
export type MazeGameEvent = { type: typeof MazeGameEventType.DoorOpened; row: number; col: number }

export type { MazeGameWasm }

/** Creates a new MazeGameWasm from a definition JSON string {"grid":[...]}. Caller must call freeMazeGame() on unmount. */
export async function createMazeGame(definitionJson: string): Promise<MazeGameWasm> {
  await ensureInit()
  try {
    return MazeGameWasm.from_json(definitionJson)
  } catch (ex) { throw toError(ex) }
}

/**
 * Moves the player one step. Returns MazeGamePlayerMoveResult.
 * Blocked means the game object is unchanged. Moved or Complete means it has advanced.
 */
export function moveMazeGamePlayer(game: MazeGameWasm, dir: MazeGameDirection): MazeGamePlayerMoveResult {
  // MazeGameDirection and DirectionWasm share identical integer values — cast is zero-cost.
  return game.move_player(dir as unknown as DirectionWasm) as unknown as MazeGamePlayerMoveResult
}

/** Picks up the item at the player's current cell, or null if the cell holds none. */
export function pickupItem(game: MazeGameWasm): MazeBagItem | null {
  return game.pickup() as unknown as MazeBagItem | null
}

/** Advances time-based state by dtMs milliseconds; returns the events that occurred. */
export function tickGame(game: MazeGameWasm, dtMs: number): MazeGameEvent[] {
  return game.tick(dtMs) as unknown as MazeGameEvent[]
}

/** Returns the door cells and their current state. */
export function getDoors(game: MazeGameWasm): MazeDoor[] {
  return game.doors() as unknown as MazeDoor[]
}

/** Returns the cells still holding an uncollected key. */
export function getKeys(game: MazeGameWasm): MazeKeyCell[] {
  return game.keys() as unknown as MazeKeyCell[]
}

/** Returns the player's bag contents, in pickup order. */
export function getBag(game: MazeGameWasm): MazeBagItem[] {
  return game.bag() as unknown as MazeBagItem[]
}

/** Whether the game has ended in a loss (player stranded — too few keys for remaining real path doors). */
export function isMazeGameLost(game: MazeGameWasm): boolean {
  return game.is_lost()
}

/** Lose reason ('stranded'), or null while the game is still in play or already won. */
export function mazeGameLoseReason(game: MazeGameWasm): MazeGameLoseReason | null {
  return game.lose_reason() as unknown as MazeGameLoseReason | null
}

/** Frees the WASM game object. Call on unmount or when definitionJson changes. */
export function freeMazeGame(game: MazeGameWasm): void {
  game.free()
}
