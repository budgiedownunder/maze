import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'

// vi.hoisted() ensures all mock helpers are initialised before vi.mock() hoisting.
const {
  mockCreateMazeGame, mockMoveMazeGamePlayer, mockFreeMazeGame,
  mockPickupItem, mockTickGame, mockGetDoors, mockGameInstance,
} = vi.hoisted(() => {
    const mockGameInstance = {
      player_row: vi.fn().mockReturnValue(0),
      player_col: vi.fn().mockReturnValue(0),
      player_direction: vi.fn().mockReturnValue(0),
      is_complete: vi.fn().mockReturnValue(false),
      visited_cells: vi.fn().mockReturnValue([]),
      free: vi.fn(),
    }
    return {
      mockCreateMazeGame: vi.fn().mockResolvedValue(mockGameInstance),
      mockMoveMazeGamePlayer: vi.fn().mockReturnValue(1), // Moved
      mockFreeMazeGame: vi.fn(),
      mockPickupItem: vi.fn().mockReturnValue(null),
      mockTickGame: vi.fn().mockReturnValue([]),
      mockGetDoors: vi.fn().mockReturnValue([]),
      mockGameInstance,
    }
  })

vi.mock('../../src/wasm/mazeWasm', () => ({
  createMazeGame: mockCreateMazeGame,
  moveMazeGamePlayer: mockMoveMazeGamePlayer,
  freeMazeGame: mockFreeMazeGame,
  pickupItem: mockPickupItem,
  tickGame: mockTickGame,
  getDoors: mockGetDoors,
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
  MazeGamePlayerMoveResult: { None: 0, Moved: 1, Blocked: 2, Complete: 3, BlockedByLockedDoor: 4, StartedUnlocking: 5 },
  MazeDoorState: { Locked: 'locked', Opening: 'opening', Open: 'open' },
}))

import { useMazeGame, MazeGameDirection, MazeGamePlayerMoveResult } from '../../src/hooks/useMazeGame'

const DEFINITION_JSON = '{"grid":[["S"," ","F"]]}'

beforeEach(() => {
  vi.clearAllMocks()
  mockCreateMazeGame.mockResolvedValue(mockGameInstance)
  mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Moved)
})

describe('useMazeGame', () => {
  it('null definitionJson — no loading, no error, game stays null', () => {
    const { result } = renderHook(() => useMazeGame(null))
    const [state] = result.current
    expect(state.loading).toBe(false)
    expect(state.error).toBeNull()
    expect(state.game).toBeNull()
    expect(mockCreateMazeGame).not.toHaveBeenCalled()
  })

  it('loading is true while createMazeGame is pending', async () => {
    let resolveGame!: (g: typeof mockGameInstance) => void
    mockCreateMazeGame.mockReturnValue(new Promise(res => { resolveGame = res }))

    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))

    // Effect fires synchronously in renderHook setup; loading should be true before resolution
    expect(result.current[0].loading).toBe(true)
    expect(result.current[0].game).toBeNull()

    // Resolve and flush
    await act(async () => { resolveGame(mockGameInstance) })
    expect(result.current[0].loading).toBe(false)
    expect(result.current[0].game).toBe(mockGameInstance)
  })

  it('resolves: loading false, game set', async () => {
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    const [state] = result.current
    expect(state.loading).toBe(false)
    expect(state.game).toBe(mockGameInstance)
    expect(state.error).toBeNull()
  })

  it('move Moved — version increments', async () => {
    mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Moved)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    expect(result.current[0].version).toBe(0)
    act(() => { result.current[1](MazeGameDirection.Right) })
    expect(result.current[0].version).toBe(1)
  })

  it('move Complete — version increments', async () => {
    mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Complete)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[1](MazeGameDirection.Right) })
    expect(result.current[0].version).toBe(1)
  })

  it('move Blocked — version unchanged', async () => {
    mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Blocked)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[1](MazeGameDirection.Up) })
    expect(result.current[0].version).toBe(0)
  })

  it('move BlockedByLockedDoor — version unchanged', async () => {
    mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.BlockedByLockedDoor)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[1](MazeGameDirection.Right) })
    expect(result.current[0].version).toBe(0)
  })

  it('pickup picks up an item — version increments', async () => {
    mockPickupItem.mockReturnValue({ type: 'key', id: 0 })
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[2]() })
    expect(result.current[0].version).toBe(1)
  })

  it('pickup with nothing to collect — version unchanged', async () => {
    mockPickupItem.mockReturnValue(null)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[2]() })
    expect(result.current[0].version).toBe(0)
  })

  it('unmount calls freeMazeGame', async () => {
    const { result, unmount } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    expect(result.current[0].game).toBe(mockGameInstance)
    unmount()
    expect(mockFreeMazeGame).toHaveBeenCalledWith(mockGameInstance)
  })

  it('error: createMazeGame rejects — error set, loading false', async () => {
    mockCreateMazeGame.mockRejectedValue(new Error('invalid definition'))
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    expect(result.current[0].loading).toBe(false)
    expect(result.current[0].error).toBe('invalid definition')
    expect(result.current[0].game).toBeNull()
  })
})
