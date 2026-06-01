import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'

// vi.hoisted() ensures all mock helpers are initialised before vi.mock() hoisting.
const {
  mockCreateMazeGame, mockMoveMazeGamePlayer, mockFreeMazeGame,
  mockTickGame, mockGetTimeUntilNextEvent, mockGameInstance,
} = vi.hoisted(() => {
    const mockGameInstance = {
      player_row: vi.fn().mockReturnValue(0),
      player_col: vi.fn().mockReturnValue(0),
      player_direction: vi.fn().mockReturnValue(0),
      is_complete: vi.fn().mockReturnValue(false),
      is_lost: vi.fn().mockReturnValue(false),
      lose_reason: vi.fn().mockReturnValue(null),
      visited_cells: vi.fn().mockReturnValue([]),
      free: vi.fn(),
    }
    return {
      mockCreateMazeGame: vi.fn().mockResolvedValue(mockGameInstance),
      mockMoveMazeGamePlayer: vi.fn().mockReturnValue(1), // Moved
      mockFreeMazeGame: vi.fn(),
      mockTickGame: vi.fn().mockReturnValue([]),
      mockGetTimeUntilNextEvent: vi.fn().mockReturnValue(null),
      mockGameInstance,
    }
  })

vi.mock('../../src/wasm/mazeWasm', () => ({
  createMazeGame: mockCreateMazeGame,
  moveMazeGamePlayer: mockMoveMazeGamePlayer,
  freeMazeGame: mockFreeMazeGame,
  tickGame: mockTickGame,
  getTimeUntilNextEvent: mockGetTimeUntilNextEvent,
  MazeGameDirection: { None: 0, Up: 1, Down: 2, Left: 3, Right: 4 },
  MazeGamePlayerMoveResult: { None: 0, Moved: 1, Blocked: 2, Complete: 3, BlockedByLockedDoor: 4, StartedUnlocking: 5, Stranded: 6, Killed: 7 },
  MazeGameEventType: { DoorOpened: 'doorOpened', EnemyMoved: 'enemyMoved', PlayerDamaged: 'playerDamaged', PlayerHealed: 'playerHealed', PlayerNotHealed: 'playerNotHealed' },
}))

import { useMazeGame, MazeGameDirection, MazeGamePlayerMoveResult } from '../../src/hooks/useMazeGame'

const DEFINITION_JSON = '{"grid":[["S"," ","F"]]}'

beforeEach(() => {
  vi.clearAllMocks()
  mockCreateMazeGame.mockResolvedValue(mockGameInstance)
  mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Moved)
  mockGetTimeUntilNextEvent.mockReturnValue(null)
  mockTickGame.mockReturnValue([])
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

  it('move Killed — version increments', async () => {
    mockMoveMazeGamePlayer.mockReturnValue(MazeGamePlayerMoveResult.Killed)
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    act(() => { result.current[1](MazeGameDirection.Right) })
    expect(result.current[0].version).toBe(1)
  })

  it('damageFlashKey starts at 0 on a fresh game', async () => {
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    expect(result.current[0].damageFlashKey).toBe(0)
  })

  it('damageFlashKey increments once per playerDamaged event from a tick', async () => {
    vi.useFakeTimers()
    // Two playerDamaged events arrive in a single tick (e.g. two enemies on
    // the same cell). damageFlashKey should reflect both hits so consecutive
    // hits still restart the CSS animation.
    mockGetTimeUntilNextEvent.mockReturnValueOnce(0).mockReturnValue(null)
    mockTickGame.mockReturnValueOnce([
      { type: 'playerDamaged', hpAfter: 2 },
      { type: 'playerDamaged', hpAfter: 1 },
    ])
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    await act(async () => { await vi.runOnlyPendingTimersAsync() })
    expect(result.current[0].damageFlashKey).toBe(2)
    vi.useRealTimers()
  })

  it('non-damage events do not bump damageFlashKey', async () => {
    vi.useFakeTimers()
    mockGetTimeUntilNextEvent.mockReturnValueOnce(0).mockReturnValue(null)
    mockTickGame.mockReturnValueOnce([
      { type: 'enemyMoved', id: 0, row: 0, col: 1 },
      { type: 'playerHealed', hpAfter: 3, row: 0, col: 2 },
    ])
    const { result } = renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    await act(async () => { await vi.runOnlyPendingTimersAsync() })
    expect(result.current[0].damageFlashKey).toBe(0)
    // But version still bumps (events were emitted).
    expect(result.current[0].version).toBeGreaterThan(0)
    vi.useRealTimers()
  })

  it('no timer is armed when getTimeUntilNextEvent returns null (idle game)', async () => {
    vi.useFakeTimers()
    mockGetTimeUntilNextEvent.mockReturnValue(null)
    renderHook(() => useMazeGame(DEFINITION_JSON))
    await act(async () => {})
    // tick should not have been called yet
    expect(mockTickGame).not.toHaveBeenCalled()
    // Even after time passes, idle game stays idle.
    await act(async () => { await vi.advanceTimersByTimeAsync(5000) })
    expect(mockTickGame).not.toHaveBeenCalled()
    vi.useRealTimers()
  })
})
