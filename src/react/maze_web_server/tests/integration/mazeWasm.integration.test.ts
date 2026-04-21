// @vitest-environment node
//
// Integration tests for the maze WASM API — run against the real maze_wasm binary.
// These complement the unit tests in tests/unit/mazeWasm.test.ts, which verify
// argument passing and resource cleanup via mocks. These tests verify actual
// behaviour: maze generation, solving, game movement, blocking, completion, and
// visited-cell tracking.
//
// Node cannot use the default fetch-based WASM loader, so we pass the binary bytes
// directly to init(). The ensureInit() singleton in mazeWasm.ts will see wasm already
// initialised and skip its own init() call.

import { describe, it, expect, beforeAll } from 'vitest'
import { readFileSync } from 'fs'
import { fileURLToPath } from 'url'
import { resolve, dirname } from 'path'
import init from 'maze_wasm'
import {
  generateMaze,
  solveMaze,
  createMazeGame,
  moveMazeGamePlayer,
  freeMazeGame,
  MazeGameDirection,
  MazeGamePlayerMoveResult,
  type MazeDefinition,
  type GenerateOptions,
} from '../../src/wasm/mazeWasm'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasmPath = resolve(__dirname, '../../../../rust/maze_wasm/pkg/maze_wasm_bg.wasm')

// ── Fixtures ──────────────────────────────────────────────────────────────────

// Small generate options — 5×7 grid, keeps test fast while exercising real generation.
const GENERATE_OPTIONS: GenerateOptions = {
  rowCount: 5,
  colCount: 7,
  startRow: 1,    // 1-based (UI convention)
  startCol: 1,
  finishRow: 5,
  finishCol: 7,
  minSpineLength: 6,
}

// Trivially solvable 1×3 grid: S at (0,0), open at (0,1), F at (0,2).
const SOLVABLE_DEFINITION: MazeDefinition = { grid: [['S', ' ', 'F']] }

// Unsolvable 1×3 grid: S and F separated by a wall.
const UNSOLVABLE_DEFINITION: MazeDefinition = { grid: [['S', 'W', 'F']] }

// Same solvable grid as a JSON string for game tests.
// Player must move Right twice to complete.
const GAME_GRID_JSON = '{"grid":[["S"," ","F"]]}'

// ── Init ──────────────────────────────────────────────────────────────────────

beforeAll(async () => {
  await init(readFileSync(wasmPath))
})

// ── generateMaze ──────────────────────────────────────────────────────────────

describe('generateMaze (real WASM)', () => {
  it('returns a definition with the requested dimensions', async () => {
    const def = await generateMaze(GENERATE_OPTIONS)
    expect(def.grid.length).toBe(GENERATE_OPTIONS.rowCount)
    expect(def.grid[0].length).toBe(GENERATE_OPTIONS.colCount)
  })

  it('contains exactly one S cell and one F cell', async () => {
    const def = await generateMaze(GENERATE_OPTIONS)
    const flat = def.grid.flat()
    expect(flat.filter(c => c === 'S').length).toBe(1)
    expect(flat.filter(c => c === 'F').length).toBe(1)
  })

  it('contains only valid cell characters', async () => {
    const def = await generateMaze(GENERATE_OPTIONS)
    const valid = new Set(['S', 'F', ' ', 'W'])
    for (const cell of def.grid.flat()) {
      expect(valid.has(cell)).toBe(true)
    }
  })

  it('places S at the requested start position (1-based → 0-based)', async () => {
    const def = await generateMaze(GENERATE_OPTIONS)
    expect(def.grid[GENERATE_OPTIONS.startRow - 1][GENERATE_OPTIONS.startCol - 1]).toBe('S')
  })

  it('places F at the requested finish position (1-based → 0-based)', async () => {
    const def = await generateMaze(GENERATE_OPTIONS)
    expect(def.grid[GENERATE_OPTIONS.finishRow - 1][GENERATE_OPTIONS.finishCol - 1]).toBe('F')
  })

  it('throws on invalid options (rowCount < 3)', async () => {
    await expect(generateMaze({ ...GENERATE_OPTIONS, rowCount: 2 })).rejects.toThrow()
  })
})

// ── solveMaze ─────────────────────────────────────────────────────────────────

describe('solveMaze (real WASM)', () => {
  it('returns a non-empty path for a solvable maze', async () => {
    const path = await solveMaze(SOLVABLE_DEFINITION)
    expect(path.length).toBeGreaterThan(0)
  })

  it('path starts at the S cell (0-based)', async () => {
    const path = await solveMaze(SOLVABLE_DEFINITION)
    expect(path[0]).toEqual({ row: 0, col: 0 })
  })

  it('path ends at the F cell (0-based)', async () => {
    const path = await solveMaze(SOLVABLE_DEFINITION)
    expect(path[path.length - 1]).toEqual({ row: 0, col: 2 })
  })

  it('returns a path of the correct length for a known grid', async () => {
    // S → open → F requires 3 points: (0,0), (0,1), (0,2)
    const path = await solveMaze(SOLVABLE_DEFINITION)
    expect(path.length).toBe(3)
  })

  it('throws for an unsolvable maze', async () => {
    await expect(solveMaze(UNSOLVABLE_DEFINITION)).rejects.toThrow()
  })
})

// ── createMazeGame ────────────────────────────────────────────────────────────

describe('createMazeGame (real WASM)', () => {
  it('returns a game instance with player at the start cell', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      expect(game.player_row()).toBe(0)
      expect(game.player_col()).toBe(0)
      expect(game.is_complete()).toBe(false)
    } finally {
      freeMazeGame(game)
    }
  })

  it('throws on invalid JSON', async () => {
    await expect(createMazeGame('not json')).rejects.toThrow()
  })

  it('throws on a grid with no S cell', async () => {
    await expect(createMazeGame('{"grid":[[" "," ","F"]]}')).rejects.toThrow()
  })
})

// ── moveMazeGamePlayer ────────────────────────────────────────────────────────

describe('moveMazeGamePlayer (real WASM)', () => {
  it('returns Moved when moving into an open cell and advances the player', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      const result = moveMazeGamePlayer(game, MazeGameDirection.Right)
      expect(result).toBe(MazeGamePlayerMoveResult.Moved)
      expect(game.player_row()).toBe(0)
      expect(game.player_col()).toBe(1)
    } finally {
      freeMazeGame(game)
    }
  })

  it('returns Blocked when moving into a wall or out of bounds and leaves position unchanged', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      const result = moveMazeGamePlayer(game, MazeGameDirection.Up)
      expect(result).toBe(MazeGamePlayerMoveResult.Blocked)
      expect(game.player_row()).toBe(0)
      expect(game.player_col()).toBe(0)
    } finally {
      freeMazeGame(game)
    }
  })

  it('returns Complete when player reaches the finish cell', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      moveMazeGamePlayer(game, MazeGameDirection.Right)
      const result = moveMazeGamePlayer(game, MazeGameDirection.Right)
      expect(result).toBe(MazeGamePlayerMoveResult.Complete)
      expect(game.is_complete()).toBe(true)
      expect(game.player_col()).toBe(2)
    } finally {
      freeMazeGame(game)
    }
  })

  it('visited_cells grows by one after a Moved result', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      const before = (game.visited_cells() as Array<{ row: number; col: number }>).length
      moveMazeGamePlayer(game, MazeGameDirection.Right)
      const after = (game.visited_cells() as Array<{ row: number; col: number }>).length
      expect(after).toBe(before + 1)
    } finally {
      freeMazeGame(game)
    }
  })

  it('visited_cells grows by one after a Complete result', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      moveMazeGamePlayer(game, MazeGameDirection.Right)
      const before = (game.visited_cells() as Array<{ row: number; col: number }>).length
      moveMazeGamePlayer(game, MazeGameDirection.Right)
      const after = (game.visited_cells() as Array<{ row: number; col: number }>).length
      expect(after).toBe(before + 1)
    } finally {
      freeMazeGame(game)
    }
  })

  it('visited_cells does not grow after a Blocked result', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    try {
      const before = (game.visited_cells() as Array<{ row: number; col: number }>).length
      moveMazeGamePlayer(game, MazeGameDirection.Up)   // blocked — no movement
      const after = (game.visited_cells() as Array<{ row: number; col: number }>).length
      expect(after).toBe(before)
    } finally {
      freeMazeGame(game)
    }
  })
})

// ── freeMazeGame ──────────────────────────────────────────────────────────────

describe('freeMazeGame (real WASM)', () => {
  it('does not throw when freeing a valid game instance', async () => {
    const game = await createMazeGame(GAME_GRID_JSON)
    expect(() => freeMazeGame(game)).not.toThrow()
  })
})
