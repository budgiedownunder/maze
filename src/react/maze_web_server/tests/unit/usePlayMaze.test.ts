import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { usePlayMaze, GameType } from '../../src/hooks/usePlayMaze'

const { mockSolveMaze } = vi.hoisted(() => ({
  mockSolveMaze: vi.fn(),
}))

vi.mock('../../src/wasm/mazeWasm', () => ({
  solveMaze: mockSolveMaze,
}))

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return { ...actual, useNavigate: () => mockNavigate }
})

const testMaze = {
  id: 'maze-test-1',
  name: 'Test Maze',
  definition: { grid: [['S', 'F']] },
}

afterEach(() => {
  vi.clearAllMocks()
  vi.unstubAllGlobals()
})

describe('usePlayMaze', () => {
  it("play(GameType.TwoD) on solvable maze calls navigate and does not set href", async () => {
    mockSolveMaze.mockResolvedValue([])
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    const { result } = renderHook(() => usePlayMaze())
    await act(async () => {
      await result.current.play(testMaze, GameType.TwoD)
    })

    expect(mockNavigate).toHaveBeenCalledWith('/play/' + encodeURIComponent(testMaze.id))
    expect(locationStub.href).toBe('')
  })

  it("play('3d') on solvable maze sets window.location.href and does not call navigate", async () => {
    mockSolveMaze.mockResolvedValue([])
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    const { result } = renderHook(() => usePlayMaze())
    await act(async () => {
      await result.current.play(testMaze, GameType.ThreeD)
    })

    expect(locationStub.href).toBe('/game/?id=' + encodeURIComponent(testMaze.id))
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('unsolvable maze sets capitalised error and does not navigate', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution found'))
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    const { result } = renderHook(() => usePlayMaze())
    await act(async () => {
      await result.current.play(testMaze, GameType.TwoD)
    })

    expect(result.current.error).toBe('No solution found')
    expect(mockNavigate).not.toHaveBeenCalled()
    expect(locationStub.href).toBe('')
  })

  it('clearError resets error to null', async () => {
    mockSolveMaze.mockRejectedValue(new Error('no solution'))
    const locationStub = { href: '' }
    vi.stubGlobal('location', locationStub)

    const { result } = renderHook(() => usePlayMaze())
    await act(async () => {
      await result.current.play(testMaze, GameType.TwoD)
    })
    expect(result.current.error).not.toBeNull()

    act(() => {
      result.current.clearError()
    })
    expect(result.current.error).toBeNull()
  })
})
