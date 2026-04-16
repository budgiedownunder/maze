import { useState, useEffect, useRef, useCallback, type RefObject } from 'react'

export interface SpeedLevel {
  label: string
  ms: number
}

export const SPEED_LEVELS: SpeedLevel[] = [
  { label: 'Slow',   ms: 750 },
  { label: 'Normal', ms: 500 },
  { label: 'Fast',   ms: 200 },
  { label: 'Turbo',  ms: 20  },
]

const DEFAULT_SPEED_INDEX = 1 // Normal
const STORAGE_KEY = 'walkSpeed'

function loadSpeedIndex(): number {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored !== null) {
    const parsed = parseInt(stored, 10)
    if (!isNaN(parsed) && parsed >= 0 && parsed < SPEED_LEVELS.length) return parsed
  }
  return DEFAULT_SPEED_INDEX
}

export interface UseWalkSpeedResult {
  speedIndex: number
  stepMs: number
  speedLabel: string
  setSpeedIndex: (index: number) => void
  speedRef: RefObject<number>
}

export function useWalkSpeed(): UseWalkSpeedResult {
  const [speedIndex, setSpeedIndexState] = useState<number>(loadSpeedIndex)
  const speedRef = useRef<number>(SPEED_LEVELS[speedIndex].ms)

  // Keep speedRef in sync so mid-walk speed changes take effect on the next step
  useEffect(() => {
    speedRef.current = SPEED_LEVELS[speedIndex].ms
    localStorage.setItem(STORAGE_KEY, String(speedIndex))
  }, [speedIndex])

  const setSpeedIndex = useCallback((index: number) => {
    const clamped = Math.max(0, Math.min(SPEED_LEVELS.length - 1, index))
    setSpeedIndexState(clamped)
  }, [])

  return {
    speedIndex,
    stepMs: SPEED_LEVELS[speedIndex].ms,
    speedLabel: SPEED_LEVELS[speedIndex].label,
    setSpeedIndex,
    speedRef,
  }
}
