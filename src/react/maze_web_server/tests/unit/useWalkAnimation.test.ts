import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useRef } from 'react'
import { useWalkAnimation, WALK_STEP_DURATION_MS } from '../../src/hooks/useWalkAnimation'
import type { CellPoint } from '../../src/hooks/useMazeEditor'

const PATH: Array<CellPoint> = [
  { row: 0, col: 0 },
  { row: 0, col: 1 },
  { row: 0, col: 2 },
  { row: 1, col: 2 },
]

// Helper: render useWalkAnimation wired to a speed ref with the given initial ms
function setupHook(initialMs = WALK_STEP_DURATION_MS) {
  return renderHook(() => {
    const speedRef = useRef<number>(initialMs)
    const animation = useWalkAnimation(speedRef)
    return { ...animation, speedRef }
  })
}

beforeEach(() => {
  vi.useFakeTimers()
})

afterEach(() => {
  vi.useRealTimers()
})

describe('useWalkAnimation — initial state', () => {
  it('starts with walkState null and isWalking false', () => {
    const { result } = setupHook()
    expect(result.current.walkState).toBeNull()
    expect(result.current.isWalking).toBe(false)
  })
})

describe('useWalkAnimation — startWalk', () => {
  it('sets walkState to first cell immediately on startWalk', () => {
    const { result } = setupHook()
    act(() => result.current.startWalk(PATH))
    expect(result.current.walkState).not.toBeNull()
    expect(result.current.walkState?.currentIndex).toBe(0)
    expect(result.current.walkState?.isComplete).toBe(false)
    expect(result.current.isWalking).toBe(true)
  })

  it('advances one step per speedRef.current ms', () => {
    const { result } = setupHook(200)
    act(() => result.current.startWalk(PATH))
    expect(result.current.walkState?.currentIndex).toBe(0)

    act(() => vi.advanceTimersByTime(200))
    expect(result.current.walkState?.currentIndex).toBe(1)

    act(() => vi.advanceTimersByTime(200))
    expect(result.current.walkState?.currentIndex).toBe(2)
  })

  it('reaches isComplete at the last cell', () => {
    const { result } = setupHook(100)
    act(() => result.current.startWalk(PATH))
    // Advance through all but the last step
    act(() => vi.advanceTimersByTime(100 * (PATH.length - 1)))
    expect(result.current.walkState?.currentIndex).toBe(PATH.length - 1)
    expect(result.current.walkState?.isComplete).toBe(true)
    expect(result.current.isWalking).toBe(true) // still "walking" until cancelWalk
  })

  it('holds the celebrate frame — no more advances after isComplete', () => {
    const { result } = setupHook(100)
    act(() => result.current.startWalk(PATH))
    act(() => vi.advanceTimersByTime(100 * PATH.length + 5000))
    expect(result.current.walkState?.currentIndex).toBe(PATH.length - 1)
    expect(result.current.walkState?.isComplete).toBe(true)
  })

  it('ignores startWalk with an empty path', () => {
    const { result } = setupHook()
    act(() => result.current.startWalk([]))
    expect(result.current.walkState).toBeNull()
  })
})

describe('useWalkAnimation — mid-walk speed change', () => {
  it('new speedRef.current is used when the next setTimeout is scheduled', () => {
    const { result } = setupHook(500)
    act(() => result.current.startWalk(PATH))

    // Advance one step at 500ms; a new 500ms timer is now in-flight for index 2
    act(() => vi.advanceTimersByTime(500))
    expect(result.current.walkState?.currentIndex).toBe(1)

    // Change speed to 100ms — the in-flight 500ms timer still fires at full 500ms
    act(() => { result.current.speedRef.current = 100 })

    // In-flight 500ms timer still has 499ms remaining
    act(() => vi.advanceTimersByTime(499))
    expect(result.current.walkState?.currentIndex).toBe(1)

    // In-flight timer fires at 500ms; advance() reads speedRef.current=100 for the next timer
    act(() => vi.advanceTimersByTime(1))
    expect(result.current.walkState?.currentIndex).toBe(2)

    // Next timer now uses the new 100ms speed
    act(() => vi.advanceTimersByTime(99))
    expect(result.current.walkState?.currentIndex).toBe(2)

    act(() => vi.advanceTimersByTime(1))
    expect(result.current.walkState?.currentIndex).toBe(3)
  })
})

describe('useWalkAnimation — cancelWalk', () => {
  it('cancelWalk clears walkState and stops the timer', () => {
    const { result } = setupHook(300)
    act(() => result.current.startWalk(PATH))
    act(() => vi.advanceTimersByTime(300))
    expect(result.current.walkState?.currentIndex).toBe(1)

    act(() => result.current.cancelWalk())
    expect(result.current.walkState).toBeNull()
    expect(result.current.isWalking).toBe(false)

    // Confirm no further advancement after cancel
    act(() => vi.advanceTimersByTime(300 * 10))
    expect(result.current.walkState).toBeNull()
  })
})

describe('useWalkAnimation — unmount cleanup', () => {
  it('clears the timer on unmount without throwing', () => {
    const { result, unmount } = setupHook(300)
    act(() => result.current.startWalk(PATH))
    expect(() => unmount()).not.toThrow()
  })
})
