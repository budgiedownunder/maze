import { useState, useRef, useCallback, useEffect } from 'react'
import type { CellPoint } from './useMazeEditor'

export const WALK_STEP_DURATION_MS = 500

export interface WalkState {
  path: Array<CellPoint>
  currentIndex: number
  isComplete: boolean
}

interface UseWalkAnimationResult {
  walkState: WalkState | null
  isWalking: boolean
  startWalk: (path: Array<CellPoint>) => void
  cancelWalk: () => void
}

export function useWalkAnimation(): UseWalkAnimationResult {
  const [walkState, setWalkState] = useState<WalkState | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const cancelWalk = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
    setWalkState(null)
  }, [])

  // Clean up timer on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current !== null) clearTimeout(timerRef.current)
    }
  }, [])

  const startWalk = useCallback((path: Array<CellPoint>) => {
    if (path.length === 0) return

    // Cancel any in-progress walk
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }

    const advance = (index: number) => {
      const isComplete = index === path.length - 1

      setWalkState({ path, currentIndex: index, isComplete })

      if (!isComplete) {
        timerRef.current = setTimeout(() => {
          advance(index + 1)
        }, WALK_STEP_DURATION_MS)
      }
      // When complete, hold the celebrate frame until cancelWalk() is called
    }

    advance(0)
  }, [])

  return {
    walkState,
    isWalking: walkState !== null,
    startWalk,
    cancelWalk,
  }
}
