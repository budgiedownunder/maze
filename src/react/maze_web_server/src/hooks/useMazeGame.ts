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

export function useMazeGame(
  definitionJson: string | null
): [MazeGameHookState, (dir: MazeGameDirection) => void] {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [version, setVersion] = useState(0)
  const gameRef = useRef<MazeGameWasm | null>(null)

  useEffect(() => {
    if (!definitionJson) return
    setLoading(true)
    setError(null)
    setVersion(0)
    let cancelled = false
    createMazeGame(definitionJson).then(game => {
      if (cancelled) { freeMazeGame(game); return }
      gameRef.current = game
      setLoading(false)
    }).catch((err: Error) => {
      if (!cancelled) { setError(err.message); setLoading(false) }
    })
    return () => {
      cancelled = true
      if (gameRef.current) { freeMazeGame(gameRef.current); gameRef.current = null }
    }
  }, [definitionJson])

  const move = useCallback((dir: MazeGameDirection) => {
    if (!gameRef.current) return
    const result = moveMazeGamePlayer(gameRef.current, dir)
    if (result !== MazeGamePlayerMoveResult.Blocked) setVersion(v => v + 1)
  }, [])

  return [{ game: gameRef.current, version, loading, error }, move]
}
