import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { MazeDefinition, GenerateOptions } from '../../src/wasm/mazeWasm'

// Mock the maze_wasm package — WASM is not supported in jsdom.
// We test that mazeWasm.ts calls the WASM API with the correct arguments,
// handles return values, and frees resources in all cases.
//
// vi.hoisted() ensures ALL mock helpers are initialised before vi.mock()
// hoists its factory to the top of the file.

const {
  MockMazeWasm,
  mockMazeFree, mockToJson, mockFromJson, mockGenerate, mockSolve,
  mockSolutionFree, mockGetPathPoints, mockInit, mockMazeInstance,
  MockMazeGameWasm,
  mockGameFree, mockGameFromJson, mockMovePlayer, mockPlayerRow, mockPlayerCol,
  mockPlayerDirection, mockIsComplete, mockVisitedCells, mockGameInstance,
} = vi.hoisted(() => {
  const mockMazeFree = vi.fn()
  const mockToJson = vi.fn()
  const mockFromJson = vi.fn()
  const mockGenerate = vi.fn()
  const mockSolve = vi.fn()
  const mockSolutionFree = vi.fn()
  const mockGetPathPoints = vi.fn()
  const mockInit = vi.fn().mockResolvedValue(undefined)
  const mockMazeInstance = {
    free: mockMazeFree,
    to_json: mockToJson,
    from_json: mockFromJson,
    generate: mockGenerate,
    solve: mockSolve,
  }
  const MockMazeWasm = vi.fn().mockImplementation(function() { return mockMazeInstance })

  const mockGameFree = vi.fn()
  const mockGameFromJson = vi.fn()
  const mockMovePlayer = vi.fn()
  const mockPlayerRow = vi.fn()
  const mockPlayerCol = vi.fn()
  const mockPlayerDirection = vi.fn()
  const mockIsComplete = vi.fn()
  const mockVisitedCells = vi.fn()
  const mockGameInstance = {
    free: mockGameFree,
    move_player: mockMovePlayer,
    player_row: mockPlayerRow,
    player_col: mockPlayerCol,
    player_direction: mockPlayerDirection,
    is_complete: mockIsComplete,
    visited_cells: mockVisitedCells,
  }
  const MockMazeGameWasm = { from_json: mockGameFromJson }

  return {
    MockMazeWasm, mockMazeFree, mockToJson, mockFromJson, mockGenerate, mockSolve,
    mockSolutionFree, mockGetPathPoints, mockInit, mockMazeInstance,
    MockMazeGameWasm, mockGameFree, mockGameFromJson, mockMovePlayer, mockPlayerRow,
    mockPlayerCol, mockPlayerDirection, mockIsComplete, mockVisitedCells, mockGameInstance,
  }
})

vi.mock('maze_wasm', () => ({
  default: mockInit,
  MazeWasm: MockMazeWasm,
  GenerationAlgorithmWasm: { RecursiveBacktracking: 0 },
  MazeGameWasm: MockMazeGameWasm,
  DirectionWasm: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
}))

// Import after the mock is registered. The WASM singleton initialises once per
// module lifetime; clearAllMocks() resets call counts without invalidating it.
import { generateMaze, solveMaze, createMazeGame, moveMazeGamePlayer, freeMazeGame, MazeGameDirection, MazeGamePlayerMoveResult } from '../../src/wasm/mazeWasm'

const sampleDefinition: MazeDefinition = {
  grid: [
    ['S', ' ', ' '],
    [' ', 'W', ' '],
    [' ', ' ', 'F'],
  ],
}

const sampleOptions: GenerateOptions = {
  rowCount: 5,
  colCount: 7,
  startRow: 1,    // 1-based
  startCol: 1,
  finishRow: 5,
  finishCol: 7,
  minSpineLength: 6,
}

beforeEach(() => {
  vi.clearAllMocks()
  // Restore constructor and default behaviours after clearAllMocks wipes implementations
  MockMazeWasm.mockImplementation(function() { return mockMazeInstance })
  mockInit.mockResolvedValue(undefined)
  mockToJson.mockReturnValue(JSON.stringify({ id: '', name: '', definition: sampleDefinition }))
  mockSolve.mockReturnValue({ get_path_points: mockGetPathPoints, free: mockSolutionFree })
  mockGetPathPoints.mockReturnValue([])
  // Game defaults
  mockGameFromJson.mockReturnValue(mockGameInstance)
  mockMovePlayer.mockReturnValue(MazeGamePlayerMoveResult.Moved)
  mockPlayerRow.mockReturnValue(0)
  mockPlayerCol.mockReturnValue(0)
  mockPlayerDirection.mockReturnValue(MazeGameDirection.None)
  mockIsComplete.mockReturnValue(false)
  mockVisitedCells.mockReturnValue([])
})

describe('generateMaze', () => {
  it('calls WASM generate with correct 0-based coordinates', async () => {
    await generateMaze(sampleOptions)

    expect(mockGenerate).toHaveBeenCalledWith(
      5,    // rowCount
      7,    // colCount
      0,    // GenerationAlgorithmWasm.RecursiveBacktracking
      0,    // startRow - 1  (1-based → 0-based)
      0,    // startCol - 1
      4,    // finishRow - 1
      6,    // finishCol - 1
      6,    // minSpineLength
      100,  // max_retries
      undefined,
      undefined
    )
  })

  it('returns the definition from the WASM to_json output', async () => {
    const generatedDefinition: MazeDefinition = { grid: [['S', ' '], [' ', 'F']] }
    mockToJson.mockReturnValue(JSON.stringify({ id: '', name: '', definition: generatedDefinition }))

    const result = await generateMaze(sampleOptions)

    expect(result).toEqual(generatedDefinition)
  })

  it('frees the WASM maze instance on success', async () => {
    await generateMaze(sampleOptions)

    expect(mockMazeFree).toHaveBeenCalledOnce()
  })

  it('frees the WASM maze instance even when generate throws', async () => {
    mockGenerate.mockImplementation(() => { throw new Error('generation failed') })

    await expect(generateMaze(sampleOptions)).rejects.toThrow('generation failed')

    expect(mockMazeFree).toHaveBeenCalledOnce()
  })

  it('propagates the WASM error message to the caller', async () => {
    mockGenerate.mockImplementation(() => { throw new Error('generation failed') })

    await expect(generateMaze(sampleOptions)).rejects.toThrow('generation failed')
  })
})

describe('solveMaze', () => {
  it('calls from_json with the correct JSON payload', async () => {
    await solveMaze(sampleDefinition)

    expect(mockFromJson).toHaveBeenCalledWith(
      JSON.stringify({ id: '', name: '', definition: sampleDefinition })
    )
  })

  it('returns the solution path points', async () => {
    const expectedPath = [{ row: 0, col: 0 }, { row: 1, col: 0 }, { row: 2, col: 2 }]
    mockGetPathPoints.mockReturnValue(expectedPath)

    const result = await solveMaze(sampleDefinition)

    expect(result).toEqual(expectedPath)
  })

  it('frees both maze and solution instances on success', async () => {
    await solveMaze(sampleDefinition)

    expect(mockMazeFree).toHaveBeenCalledOnce()
    expect(mockSolutionFree).toHaveBeenCalledOnce()
  })

  it('frees the maze instance even when solve throws', async () => {
    mockSolve.mockImplementation(() => { throw new Error('unsolvable') })

    await expect(solveMaze(sampleDefinition)).rejects.toThrow('unsolvable')

    expect(mockMazeFree).toHaveBeenCalledOnce()
  })

  it('propagates the WASM error message to the caller', async () => {
    mockSolve.mockImplementation(() => { throw new Error('maze has no solution') })

    await expect(solveMaze(sampleDefinition)).rejects.toThrow('maze has no solution')
  })
})

describe('createMazeGame', () => {
  it('calls MazeGameWasm.from_json with the definition JSON', async () => {
    const json = '{"grid":[["S","F"]]}'
    await createMazeGame(json)
    expect(mockGameFromJson).toHaveBeenCalledWith(json)
  })

  it('returns the MazeGameWasm instance', async () => {
    const game = await createMazeGame('{"grid":[["S","F"]]}')
    expect(game).toBe(mockGameInstance)
  })

  it('throws a friendly error when from_json fails', async () => {
    mockGameFromJson.mockImplementation(() => { throw new Error('invalid json') })
    await expect(createMazeGame('bad')).rejects.toThrow('invalid json')
  })
})

describe('moveMazeGamePlayer', () => {
  it('calls move_player with the correct direction value', () => {
    moveMazeGamePlayer(mockGameInstance as never, MazeGameDirection.Up)
    expect(mockMovePlayer).toHaveBeenCalledWith(MazeGameDirection.Up)
  })

  it('returns Moved when the player advances', () => {
    mockMovePlayer.mockReturnValue(MazeGamePlayerMoveResult.Moved)
    expect(moveMazeGamePlayer(mockGameInstance as never, MazeGameDirection.Right))
      .toBe(MazeGamePlayerMoveResult.Moved)
  })

  it('returns Blocked when the move is invalid', () => {
    mockMovePlayer.mockReturnValue(MazeGamePlayerMoveResult.Blocked)
    expect(moveMazeGamePlayer(mockGameInstance as never, MazeGameDirection.Left))
      .toBe(MazeGamePlayerMoveResult.Blocked)
  })

  it('returns Complete when the player reaches the finish', () => {
    mockMovePlayer.mockReturnValue(MazeGamePlayerMoveResult.Complete)
    expect(moveMazeGamePlayer(mockGameInstance as never, MazeGameDirection.Down))
      .toBe(MazeGamePlayerMoveResult.Complete)
  })
})

describe('freeMazeGame', () => {
  it('calls free on the game instance', () => {
    freeMazeGame(mockGameInstance as never)
    expect(mockGameFree).toHaveBeenCalledOnce()
  })
})
