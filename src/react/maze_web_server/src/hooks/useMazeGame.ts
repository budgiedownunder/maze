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
  const [game, setGame] = useState<MazeGameWasm | null>(null)
  const gameRef = useRef<MazeGameWasm | null>(null)
  const lastMoveTickRef = useRef<number>(0)
  const lastMoveDirectionRef = useRef<MazeGameDirection | null>(null)
  const MOVE_INTERVAL_MS = 120

  useEffect(() => {
    if (!definitionJson) return
    setLoading(true)
    setError(null)
    setVersion(0)
    setGame(null)
    let cancelled = false
    createMazeGame(definitionJson).then(newGame => {
      if (cancelled) { freeMazeGame(newGame); return }
      gameRef.current = newGame
      setGame(newGame)
      setLoading(false)
    }).catch((err: Error) => {
      if (!cancelled) { setError(err.message); setLoading(false) }
    })
    return () => {
      cancelled = true
      if (gameRef.current) { freeMazeGame(gameRef.current); gameRef.current = null }
      setGame(null)
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
    if (result !== MazeGamePlayerMoveResult.Blocked) setVersion(v => v + 1)
  }, [])

  return [{ game, version, loading, error }, move]
}
