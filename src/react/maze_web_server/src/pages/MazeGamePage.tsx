import { useState, useEffect, useRef, useMemo } from 'react'
import { useParams } from 'react-router-dom'
import { getMaze } from '../api/client'
import { useToken } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useMazeGame, MazeGameDirection } from '../hooks/useMazeGame'
import { getBag, getKeys } from '../wasm/mazeWasm'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { MazeGrid } from '../components/MazeGrid'
import { GameResultPopup } from '../components/GameResultPopup'
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
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()

  const [maze, setMaze] = useState<Maze | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)

  useEffect(() => {
    if (!id || !token) return
    getMaze(token, id).then(setMaze).catch((e: Error) => setLoadError(e.message))
  }, [id, token])

  const gameCellSize = window.matchMedia('(pointer: coarse)').matches ? 60 : 32

  const definitionJson = maze ? JSON.stringify(maze.definition) : null
  const [{ game, version, loading, error }, move, pickup] = useMazeGame(definitionJson)

  // Bag contents and whether the player is standing on an uncollected key —
  // recomputed whenever the game advances (version bump).
  const bag = useMemo(
    () => (game ? getBag(game) : []),
    [game, version], // eslint-disable-line react-hooks/exhaustive-deps
  )
  const onKey = useMemo(
    () => (game ? getKeys(game).some(k => k.row === game.player_row() && k.col === game.player_col()) : false),
    [game, version], // eslint-disable-line react-hooks/exhaustive-deps
  )

  const isComplete = game?.is_complete() ?? false
  const [showResult, setShowResult] = useState(false)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (isComplete) setShowResult(true)
  }, [isComplete])

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
      if (game?.is_complete()) return
      if (e.key === 'e' || e.key === 'E') { e.preventDefault(); pickup(); return }
      const dir = KEY_MAP[e.key]
      if (dir !== undefined) { e.preventDefault(); move(dir) }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [move, pickup, game])

  return (
    <div className="maze-game-page">
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">{maze?.name ?? ''}</span>
        <div className="header-actions">
          <button
            className="theme-toggle"
            onClick={toggleTheme}
            aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
            title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {theme === 'dark' ? '☀' : '☾'}
          </button>
        </div>
      </header>

      <div className="maze-game-content">
        {loadError && <p className="error-msg" role="alert">{loadError}</p>}
        {error && <p className="error-msg" role="alert">{error}</p>}

        {!loadError && !error && (!maze || !game || loading) && (
          <p className="loading-msg" role="status" aria-label="Loading">Loading…</p>
        )}

        {maze && game && !loading && !loadError && !error && (
          <>
            <MazeGrid
              grid={maze.definition.grid}
              solution={null}
              activeCell={null}
              anchorCell={null}
              game={game}
              version={version}
              cellSize={gameCellSize}
            />

            <div className="maze-bag" aria-label="Bag">
              <span>Bag:</span>
              {bag.length === 0
                ? <span className="maze-bag-empty">empty</span>
                : bag.map((_, i) => <img key={i} src="/images/maze/key.svg" alt="Key" />)}
            </div>

            <div className="game-dpad" aria-label="D-pad">
              <button type="button" aria-label="Move up"    onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Up) }}    onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete} style={{ gridArea: 'up' }}>
                <img src="/images/maze/dpad_up.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move left"  onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Left) }}  onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete} style={{ gridArea: 'left' }}>
                <img src="/images/maze/dpad_left.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move down"  onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Down) }}  onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete} style={{ gridArea: 'down' }}>
                <img src="/images/maze/dpad_down.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Move right" onPointerDown={e => { e.preventDefault(); startRepeat(MazeGameDirection.Right) }} onPointerUp={stopRepeat} onPointerLeave={stopRepeat} onPointerCancel={stopRepeat} onContextMenu={e => e.preventDefault()} aria-disabled={isComplete} style={{ gridArea: 'right' }}>
                <img src="/images/maze/dpad_right.png" alt="" draggable={false} />
              </button>
              <button type="button" aria-label="Pick up" onClick={() => pickup()} onContextMenu={e => e.preventDefault()} aria-disabled={!onKey} style={{ gridArea: 'pick' }}>
                <img src="/images/maze/key.svg" alt="" draggable={false} />
              </button>
            </div>

            <div className="maze-shortcuts-hint">
              [&#x2191;/W]&nbsp;Up&nbsp;&nbsp;&nbsp;
              [&#x2193;/S]&nbsp;Down&nbsp;&nbsp;&nbsp;
              [&#x2190;/A]&nbsp;Left&nbsp;&nbsp;&nbsp;
              [&#x2192;/D]&nbsp;Right&nbsp;&nbsp;&nbsp;
              [E]&nbsp;Pick&nbsp;up
            </div>

            {showResult && (
              <GameResultPopup
                message="You win!"
                onClose={() => setShowResult(false)}
              />
            )}
          </>
        )}
      </div>
    </div>
  )
}
