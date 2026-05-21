import { useState, useEffect, useRef, useCallback } from 'react'
import type { MazeGameWasm } from 'maze_wasm'
import {
  createMazeGame, moveMazeGamePlayer, freeMazeGame,
  pickupItem, tickGame, getDoors,
  MazeGameDirection, MazeGamePlayerMoveResult, MazeDoorState,
} from '../wasm/mazeWasm'

export { MazeGameDirection, MazeGamePlayerMoveResult }
export type { MazeGameWasm }

export interface MazeGameHookState {
  game: MazeGameWasm | null
  version: number
  loading: boolean
  error: string | null
}

type LoadResult = {
  key: string
  game: MazeGameWasm | null
  error: string | null
  version: number
}

export function useMazeGame(
  definitionJson: string | null
): [MazeGameHookState, (dir: MazeGameDirection) => void, () => void] {
  const [loadResult, setLoadResult] = useState<LoadResult | null>(null)
  const gameRef = useRef<MazeGameWasm | null>(null)
  const rafRef = useRef<number | null>(null)
  const lastMoveTickRef = useRef<number>(0)
  const lastMoveDirectionRef = useRef<MazeGameDirection | null>(null)
  const MOVE_INTERVAL_MS = 120

  // Render-time derivation: only honor loadResult while it matches the current
  // definitionJson. When definitionJson changes, the prior result is stale
  // until the effect produces a new one — render as loading in the meantime.
  // Computing this here (instead of resetting state synchronously inside the
  // effect) is what keeps the effect free of set-state-in-effect violations.
  const matches = loadResult !== null && loadResult.key === definitionJson
  const game = matches ? loadResult!.game : null
  const error = matches ? loadResult!.error : null
  const version = matches ? loadResult!.version : 0
  const loading = definitionJson !== null && !matches

  const bumpVersion = useCallback(() => {
    setLoadResult(prev => prev ? { ...prev, version: prev.version + 1 } : prev)
  }, [])

  const stopTickLoop = useCallback(() => {
    if (rafRef.current !== null) { cancelAnimationFrame(rafRef.current); rafRef.current = null }
  }, [])

  // Drives time-based state (door opening) via requestAnimationFrame. Started when a
  // move begins unlocking a door; runs only for the ~1s open then stops — the game is
  // otherwise event-driven. Re-renders only when a door actually finishes opening.
  const startTickLoop = useCallback(() => {
    if (rafRef.current !== null) return
    let lastTs = 0
    const frame = (ts: number) => {
      const g = gameRef.current
      if (!g) { rafRef.current = null; return }
      const dt = lastTs === 0 ? 16 : Math.min(100, ts - lastTs)
      lastTs = ts
      const events = tickGame(g, dt)
      if (events.length > 0) bumpVersion()
      if (getDoors(g).some(d => d.state === MazeDoorState.Opening)) {
        rafRef.current = requestAnimationFrame(frame)
      } else {
        rafRef.current = null
      }
    }
    rafRef.current = requestAnimationFrame(frame)
  }, [bumpVersion])

  useEffect(() => {
    if (!definitionJson) return
    let cancelled = false
    const key = definitionJson
    createMazeGame(definitionJson).then(newGame => {
      if (cancelled) { freeMazeGame(newGame); return }
      gameRef.current = newGame
      setLoadResult({ key, game: newGame, error: null, version: 0 })
    }).catch((err: Error) => {
      if (!cancelled) setLoadResult({ key, game: null, error: err.message, version: 0 })
    })
    return () => {
      cancelled = true
      stopTickLoop()
      if (gameRef.current) { freeMazeGame(gameRef.current); gameRef.current = null }
    }
  }, [definitionJson, stopTickLoop])

  const move = useCallback((dir: MazeGameDirection) => {
    if (!gameRef.current) return
    if (gameRef.current.is_complete()) return
    const now = Date.now()
    if (dir !== lastMoveDirectionRef.current) lastMoveTickRef.current = 0
    if (now - lastMoveTickRef.current < MOVE_INTERVAL_MS) return
    lastMoveTickRef.current = now
    lastMoveDirectionRef.current = dir
    const result = moveMazeGamePlayer(gameRef.current, dir)
    if (
      result === MazeGamePlayerMoveResult.Moved ||
      result === MazeGamePlayerMoveResult.Complete ||
      result === MazeGamePlayerMoveResult.StartedUnlocking
    ) {
      bumpVersion()
    }
    if (result === MazeGamePlayerMoveResult.StartedUnlocking) {
      startTickLoop()
    }
  }, [bumpVersion, startTickLoop])

  const pickup = useCallback(() => {
    if (!gameRef.current) return
    if (pickupItem(gameRef.current)) bumpVersion()
  }, [bumpVersion])

  return [{ game, version, loading, error }, move, pickup]
}
