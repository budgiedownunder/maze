import { useState, useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { getMaze } from '../api/client'
import { useToken } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useMazeGame, MazeGameDirection } from '../hooks/useMazeGame'
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

  const definitionJson = maze ? JSON.stringify(maze.definition) : null
  const [{ game, version, loading, error }, move] = useMazeGame(definitionJson)

  const isComplete = game?.is_complete() ?? false
  const [showResult, setShowResult] = useState(false)
  useEffect(() => {
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
      const dir = KEY_MAP[e.key]
      if (dir !== undefined) { e.preventDefault(); move(dir) }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [move, game])

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
            />

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
            </div>

            <div className="maze-shortcuts-hint">
              [&#x2191;/W]&nbsp;Up&nbsp;&nbsp;&nbsp;
              [&#x2193;/S]&nbsp;Down&nbsp;&nbsp;&nbsp;
              [&#x2190;/A]&nbsp;Left&nbsp;&nbsp;&nbsp;
              [&#x2192;/D]&nbsp;Right
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
