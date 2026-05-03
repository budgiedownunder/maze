import { useState, useEffect, useRef, useCallback } from 'react'
import type { MazeGameWasm } from 'maze_wasm'
import {
  createMazeGame, moveMazeGamePlayer, freeMazeGame,
  MazeGameDirection, MazeGamePlayerMoveResult,
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
): [MazeGameHookState, (dir: MazeGameDirection) => void] {
  const [loadResult, setLoadResult] = useState<LoadResult | null>(null)
  const gameRef = useRef<MazeGameWasm | null>(null)
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
      if (gameRef.current) { freeMazeGame(gameRef.current); gameRef.current = null }
    }
  }, [definitionJson])

  const move = useCallback((dir: MazeGameDirection) => {
    if (!gameRef.current) return
    if (gameRef.current.is_complete()) return
    const now = Date.now()
    if (dir !== lastMoveDirectionRef.current) lastMoveTickRef.current = 0
    if (now - lastMoveTickRef.current < MOVE_INTERVAL_MS) return
    lastMoveTickRef.current = now
    lastMoveDirectionRef.current = dir
    const result = moveMazeGamePlayer(gameRef.current, dir)
    if (result !== MazeGamePlayerMoveResult.Blocked) {
      setLoadResult(prev => prev ? { ...prev, version: prev.version + 1 } : prev)
    }
  }, [])

  return [{ game, version, loading, error }, move]
}
