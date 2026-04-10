import init, { MazeWasm, GenerationAlgorithmWasm } from 'maze_wasm'

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
export async function generateMaze(options: GenerateOptions): Promise<MazeDefinition> {
  await ensureInit()
  const maze = new MazeWasm()
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
    maze.from_json(JSON.stringify({ id: '', name: '', definition }))
    const solution = maze.solve()
    try {
      return solution.get_path_points() as Array<{ row: number; col: number }>
    } finally {
      solution.free()
    }
  } finally {
    maze.free()
  }
}
