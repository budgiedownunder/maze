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
        undefined   // seed (random)
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
  None:     0,
  Moved:    1,
  Blocked:  2,
  Complete: 3,
} as const
export type MazeGamePlayerMoveResult = typeof MazeGamePlayerMoveResult[keyof typeof MazeGamePlayerMoveResult]

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

/** Frees the WASM game object. Call on unmount or when definitionJson changes. */
export function freeMazeGame(game: MazeGameWasm): void {
  game.free()
}
