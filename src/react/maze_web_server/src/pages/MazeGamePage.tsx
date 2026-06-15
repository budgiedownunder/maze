import { useState, useEffect, useRef, useMemo } from 'react'
import { useParams } from 'react-router-dom'
import { getMaze } from '../api/client'
import { useToken } from '../context/AuthContext'
import { useMazeGame, MazeGameDirection } from '../hooks/useMazeGame'
import { getBag, getHp, getMaxHp, getGameGrid, getGameCellOverrides, MazeGameLoseReason } from '../wasm/mazeWasm'
import { AppHeader } from '../components/AppHeader'
import { MazeGrid } from '../components/MazeGrid'
import { GameResultPopup } from '../components/GameResultPopup'
import { PausePopup } from '../components/PausePopup'
import type { Maze } from '../types/api'

const KEY_MAP: Record<string, MazeGameDirection> = {
  ArrowUp: MazeGameDirection.Up,    w: MazeGameDirection.Up,    W: MazeGameDirection.Up,
  ArrowDown: MazeGameDirection.Down, s: MazeGameDirection.Down,  S: MazeGameDirection.Down,
  ArrowLeft: MazeGameDirection.Left, a: MazeGameDirection.Left,  A: MazeGameDirection.Left,
  ArrowRight: MazeGameDirection.Right, d: MazeGameDirection.Right, D: MazeGameDirection.Right,
}

export function MazeGamePage() {
  const { id } = useParams<{ id: string }>()
  const token = useToken()

  const [maze, setMaze] = useState<Maze | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)

  useEffect(() => {
    if (!id || !token) return
    getMaze(token, id).then(setMaze).catch((e: Error) => setLoadError(e.message))
  }, [id, token])

  const gameCellSize = window.matchMedia('(pointer: coarse)').matches ? 60 : 32

  const definitionJson = maze ? JSON.stringify(maze.definition) : null
  const [{ game, version, loading, error, damageFlashKey, paused }, move, restart, togglePause] = useMazeGame(definitionJson)

  // Display grid + variant overrides read straight from the live game object — the
  // single source of truth. grid() returns a pure-char grid (overridden cells reported
  // as their base char), and cell_overrides() returns the variant per overridden cell;
  // MazeGrid pairs them to pick the right sprite. Reading from the game (rather than a
  // separate codec pass over the definition) avoids a second WASM maze overlapping the
  // live tick loop. (Live enemy rigs ride the game object too, so they render mid-move.)
  const displayGrid = useMemo(
    () => (game ? getGameGrid(game) : null),
    [game],
  )
  const cellOverrides = useMemo(
    () => (game ? new Map(getGameCellOverrides(game).map(o => [`${o.row},${o.col}`, o.entity])) : undefined),
    [game],
  )

  // Bag contents — recomputed whenever the game advances (version bump). Keys
  // are auto-collected on walk-over, so the bag grows as the player moves.
  const bag = useMemo(
    () => (game ? getBag(game) : []),
    [game, version], // eslint-disable-line react-hooks/exhaustive-deps
  )

  // HP HUD state — re-read each version bump.
  const hp = useMemo(
    () => (game ? getHp(game) : 0),
    [game, version], // eslint-disable-line react-hooks/exhaustive-deps
  )
  const maxHp = useMemo(
    () => (game ? getMaxHp(game) : 0),
    [game, version], // eslint-disable-line react-hooks/exhaustive-deps
  )

  const isComplete = game?.is_complete() ?? false
  const isLost = game?.is_lost() ?? false
  const loseReason = game?.lose_reason() as string | null | undefined
  const resultMessage = isLost
    ? (loseReason === MazeGameLoseReason.Killed ? 'You died!' : "You're stranded!!")
    : 'You win!'
  const [showResult, setShowResult] = useState(false)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (isComplete || isLost) setShowResult(true)
  }, [isComplete, isLost])

  const repeatTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const repeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  function stopRepeat() {
    if (repeatTimeoutRef.current !== null) { clearTimeout(repeatTimeoutRef.current); repeatTimeoutRef.current = null }
    if (repeatIntervalRef.current !== null) { clearInterval(repeatIntervalRef.current); repeatIntervalRef.current = null }
  }

  function startRepeat(dir: MazeGameDirection) {
    stopRepeat()
    move(dir)
    repeatTimeoutRef.current = setTimeout(() => {
      repeatIntervalRef.current = setInterval(() => move(dir), 120)
    }, 300)
  }

  useEffect(() => () => stopRepeat(), [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (game?.is_complete() || game?.is_lost()) return
      // Space / Esc toggle pause. preventDefault stops Space from scrolling the
      // page or activating a focused popup button, and stops Esc from closing
      // the modal natively (the toggle owns resume).
      if (e.key === ' ' || e.key === 'Escape') { e.preventDefault(); togglePause(); return }
      const dir = KEY_MAP[e.key]
      // move() is itself a no-op while paused, so no explicit guard needed here.
      if (dir !== undefined) { e.preventDefault(); move(dir) }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [move, togglePause, game])

  return (
    <div className="maze-game-page">
      <AppHeader title={maze?.name ?? ''} />

      <div className="maze-game-content">
        {loadError && <p className="error-msg" role="alert">{loadError}</p>}
        {error && <p className="error-msg" role="alert">{error}</p>}

        {!loadError && !error && (!maze || !game || loading || !displayGrid) && (
          <p className="loading-msg" role="status" aria-label="Loading">Loading…</p>
        )}

        {maze && game && displayGrid && !loading && !loadError && !error && (
          <>
            {damageFlashKey > 0 && (
              <div
                key={damageFlashKey}
                className="maze-damage-flash"
                aria-hidden="true"
              />
            )}

            <MazeGrid
              grid={displayGrid}
              solution={null}
              activeCell={null}
              anchorCell={null}
              game={game}
              version={version}
              cellSize={gameCellSize}
              cellOverrides={cellOverrides}
              gameSettings={maze?.game_settings}
            />

            <div className="maze-game-status">
              {maxHp > 0 && (
                <div className="maze-hp-hud" aria-label="Health">
                  <span className="maze-hp-hud-label">LIFE</span>
                  {Array.from({ length: maxHp }, (_, i) => (
                    <img
                      key={i}
                      src="/images/maze/health.svg"
                      alt={i < hp ? 'Health' : 'Lost health'}
                      className={i < hp ? 'maze-hp-hud-heart' : 'maze-hp-hud-heart maze-hp-hud-heart--empty'}
                    />
                  ))}
                </div>
              )}

              <div className="maze-bag" aria-label="Bag">
                <span>BAG</span>
                {bag.length === 0
                  ? <span className="maze-bag-empty">empty</span>
                  : bag.map((_, i) => <img key={i} src="/images/maze/key.svg" alt="Key" />)}
              </div>
            </div>

            <div className="game-dpad" aria-label="D-pad">
              <button type="button" aria-label="Move up"    onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Up) }}    onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete || isLost} style={{ gridArea: 'up' }}>
                <img src="/images/maze/dpad_up.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move left"  onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Left) }}  onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete || isLost} style={{ gridArea: 'left' }}>
                <img src="/images/maze/dpad_left.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move down"  onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Down) }}  onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete || isLost} style={{ gridArea: 'down' }}>
                <img src="/images/maze/dpad_down.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move right" onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Right) }} onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete || isLost} style={{ gridArea: 'right' }}>
                <img src="/images/maze/dpad_right.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label={paused ? 'Resume' : 'Pause'} onClick={() => togglePause()} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete || isLost} style={{ gridArea: 'pause' }}>
                <img src="/images/maze/dpad_pause.png" alt="" draggable={false} />
              </button>
            </div>

            <div className="maze-shortcuts-hint">
              [&#x2191;/W]&nbsp;Up&nbsp;&nbsp;&nbsp;
              [&#x2193;/S]&nbsp;Down&nbsp;&nbsp;&nbsp;
              [&#x2190;/A]&nbsp;Left&nbsp;&nbsp;&nbsp;
              [&#x2192;/D]&nbsp;Right&nbsp;&nbsp;&nbsp;
              [Space/Esc]&nbsp;Pause
            </div>

            {paused && (
              <PausePopup
                onResume={() => togglePause()}
                onRestart={() => restart()}
              />
            )}

            {showResult && (
              <GameResultPopup
                message={resultMessage}
                tone={isLost ? 'fail' : 'success'}
                onClose={() => setShowResult(false)}
                onPlayAgain={() => { setShowResult(false); restart() }}
              />
            )}
          </>
        )}
      </div>
    </div>
  )
}
