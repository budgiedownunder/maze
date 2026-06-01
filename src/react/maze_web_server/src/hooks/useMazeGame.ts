import { useState, useEffect, useRef, useCallback } from 'react'
import type { MazeGameWasm } from 'maze_wasm'
import {
  createMazeGame, moveMazeGamePlayer, freeMazeGame,
  tickGame, getTimeUntilNextEvent,
  MazeGameDirection, MazeGamePlayerMoveResult, MazeGameEventType,
  type MazeGameEvent,
} from '../wasm/mazeWasm'

export { MazeGameDirection, MazeGamePlayerMoveResult }
export type { MazeGameWasm }

export interface MazeGameHookState {
  game: MazeGameWasm | null
  version: number
  loading: boolean
  error: string | null
  // Increments once per `playerDamaged` event; consumers use this as a React
  // `key` on the damage-flash overlay so the CSS animation restarts on every
  // hit even when consecutive hits would otherwise dedupe.
  damageFlashKey: number
}

type LoadResult = {
  key: string
  nonce: number
  game: MazeGameWasm | null
  error: string | null
  version: number
  damageFlashKey: number
}

export function useMazeGame(
  definitionJson: string | null
): [MazeGameHookState, (dir: MazeGameDirection) => void, () => void] {
  const [loadResult, setLoadResult] = useState<LoadResult | null>(null)
  // Bumped by restart() to force the effect to free and recreate the game from
  // the same definition. Part of the match key so the old (freed) game is never
  // read during the recreate window.
  const [restartNonce, setRestartNonce] = useState(0)
  const gameRef = useRef<MazeGameWasm | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const lastTickAtRef = useRef<number>(0)
  const lastMoveTickRef = useRef<number>(0)
  const lastMoveDirectionRef = useRef<MazeGameDirection | null>(null)
  const MOVE_INTERVAL_MS = 120

  // Render-time derivation: only honor loadResult while it matches the current
  // definitionJson AND restart generation. When either changes, the prior result
  // is stale until the effect produces a new one — render as loading in the
  // meantime. Computing this here (instead of resetting state synchronously
  // inside the effect) is what keeps the effect free of set-state-in-effect
  // violations.
  const matches = loadResult !== null && loadResult.key === definitionJson && loadResult.nonce === restartNonce
  const game = matches ? loadResult!.game : null
  const error = matches ? loadResult!.error : null
  const version = matches ? loadResult!.version : 0
  const damageFlashKey = matches ? loadResult!.damageFlashKey : 0
  const loading = definitionJson !== null && !matches

  const stopTickLoop = useCallback(() => {
    if (timerRef.current !== null) { clearTimeout(timerRef.current); timerRef.current = null }
  }, [])

  // Updates derived state from a batch of events processed in this tick.
  // Bumps version so memoised reads of game.hp() / game.enemies() / etc.
  // re-derive on the next render. Increments damageFlashKey once per
  // playerDamaged event so the page-level CSS overlay restarts its
  // animation on every hit (including back-to-back hits).
  const applyEvents = useCallback((events: MazeGameEvent[]) => {
    if (events.length === 0) return
    let damageHits = 0
    for (const ev of events) {
      if (ev.type === MazeGameEventType.PlayerDamaged) damageHits++
    }
    setLoadResult(prev => prev
      ? { ...prev, version: prev.version + 1, damageFlashKey: prev.damageFlashKey + damageHits }
      : prev)
  }, [])

  // Self-scheduling chain: each fired tick re-queries the time-to-next-event
  // and arms the next setTimeout. When the getter returns null, no timer is
  // armed — the loop stays idle until the next call to scheduleWake()
  // (typically triggered by a player move) restarts it.
  const scheduleWake = useCallback(() => {
    if (timerRef.current !== null) { clearTimeout(timerRef.current); timerRef.current = null }
    const g = gameRef.current
    if (!g) return
    if (g.is_complete() || g.is_lost()) return
    const wakeIn = getTimeUntilNextEvent(g)
    if (wakeIn === null) return
    const elapsedSinceTick = performance.now() - lastTickAtRef.current
    const delay = Math.max(0, wakeIn - elapsedSinceTick)
    timerRef.current = setTimeout(() => {
      timerRef.current = null
      const g2 = gameRef.current
      if (!g2) return
      const now = performance.now()
      const dtMs = now - lastTickAtRef.current
      lastTickAtRef.current = now
      const events = tickGame(g2, dtMs)
      applyEvents(events)
      scheduleWake()
    }, delay)
  }, [applyEvents])

  useEffect(() => {
    if (!definitionJson) return
    let cancelled = false
    const key = definitionJson
    const nonce = restartNonce
    createMazeGame(definitionJson).then(newGame => {
      if (cancelled) { freeMazeGame(newGame); return }
      gameRef.current = newGame
      lastTickAtRef.current = performance.now()
      setLoadResult({ key, nonce, game: newGame, error: null, version: 0, damageFlashKey: 0 })
      // A fresh game with enemies or opening doors already has a non-null
      // time-to-next-event — schedule the first wake.
      scheduleWake()
    }).catch((err: Error) => {
      if (!cancelled) setLoadResult({ key, nonce, game: null, error: err.message, version: 0, damageFlashKey: 0 })
    })
    return () => {
      cancelled = true
      stopTickLoop()
      if (gameRef.current) { freeMazeGame(gameRef.current); gameRef.current = null }
    }
  }, [definitionJson, restartNonce, scheduleWake, stopTickLoop])

  const move = useCallback((dir: MazeGameDirection) => {
    if (!gameRef.current) return
    if (gameRef.current.is_complete() || gameRef.current.is_lost()) return
    const now = Date.now()
    if (dir !== lastMoveDirectionRef.current) lastMoveTickRef.current = 0
    if (now - lastMoveTickRef.current < MOVE_INTERVAL_MS) return
    lastMoveTickRef.current = now
    lastMoveDirectionRef.current = dir
    const result = moveMazeGamePlayer(gameRef.current, dir)
    if (
      result === MazeGamePlayerMoveResult.Moved ||
      result === MazeGamePlayerMoveResult.Complete ||
      result === MazeGamePlayerMoveResult.StartedUnlocking ||
      result === MazeGamePlayerMoveResult.Stranded ||
      result === MazeGamePlayerMoveResult.Killed
    ) {
      setLoadResult(prev => prev ? { ...prev, version: prev.version + 1 } : prev)
    }
    // A successful move may have queued KeyCollected / PlayerDamaged /
    // PlayerHealed / PlayerNotHealed events (wakeIn becomes 0), started a door
    // opening (wakeIn becomes DOOR_OPEN_MS), or otherwise changed the schedule —
    // re-evaluate so the next tick fires at the correct time.
    scheduleWake()
  }, [scheduleWake])

  // Restart the current maze from the beginning. Bumping the nonce re-runs the
  // load effect, which frees the existing game and recreates it from the same
  // definition; the nonce in the match key keeps the freed game from rendering
  // during the recreate window.
  const restart = useCallback(() => {
    setRestartNonce(n => n + 1)
  }, [])

  return [{ game, version, loading, error, damageFlashKey }, move, restart]
}
