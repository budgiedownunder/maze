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
  startWalk: (path: Array<CellPoint>, onComplete: () => void) => void
  cancelWalk: () => void
}

export function useWalkAnimation(): UseWalkAnimationResult {
  const [walkState, setWalkState] = useState<WalkState | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const onCompleteRef = useRef<(() => void) | null>(null)

  const cancelWalk = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
    onCompleteRef.current = null
    setWalkState(null)
  }, [])

  // Clean up timer on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current !== null) clearTimeout(timerRef.current)
    }
  }, [])

  const startWalk = useCallback((path: Array<CellPoint>, onComplete: () => void) => {
    if (path.length === 0) return

    // Cancel any in-progress walk
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }

    onCompleteRef.current = onComplete

    const advance = (index: number) => {
      const isComplete = index === path.length - 1

      setWalkState({ path, currentIndex: index, isComplete })

      if (isComplete) {
        // Hold celebrate frame for one extra step, then hand off to caller
        timerRef.current = setTimeout(() => {
          timerRef.current = null
          setWalkState(null)
          onCompleteRef.current?.()
          onCompleteRef.current = null
        }, WALK_STEP_DURATION_MS)
      } else {
        timerRef.current = setTimeout(() => {
          advance(index + 1)
        }, WALK_STEP_DURATION_MS)
      }
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
