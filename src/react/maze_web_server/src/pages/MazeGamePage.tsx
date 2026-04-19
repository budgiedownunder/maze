import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { getMaze } from '../api/client'
import { useToken } from '../context/AuthContext'
import { useMazeGame, MazeGameDirection } from '../hooks/useMazeGame'
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

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (game?.is_complete()) return
      const dir = KEY_MAP[e.key]
      if (dir !== undefined) { e.preventDefault(); move(dir) }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [move, game])

  if (loadError) return <p className="error-msg" role="alert">{loadError}</p>
  if (!maze || loading) return <p aria-label="Loading">Loading…</p>
  if (error) return <p className="error-msg" role="alert">{error}</p>

  return (
    <div className="maze-game-page">
      <div className="maze-game-header">
        <span className="maze-game-title">{maze.name}</span>
      </div>

      <MazeGrid
        grid={maze.definition.grid}
        solution={null}
        activeCell={null}
        anchorCell={null}
        game={game}
        version={version}
      />

      <div className="game-dpad" aria-label="D-pad">
        <button type="button" aria-label="Move up"    onClick={() => move(MazeGameDirection.Up)}    disabled={isComplete} style={{ gridArea: 'up' }}>
          <img src="/images/maze/dpad_up.png" alt="" />
        </button>
        <button type="button" aria-label="Move left"  onClick={() => move(MazeGameDirection.Left)}  disabled={isComplete} style={{ gridArea: 'left' }}>
          <img src="/images/maze/dpad_left.png" alt="" />
        </button>
        <button type="button" aria-label="Move down"  onClick={() => move(MazeGameDirection.Down)}  disabled={isComplete} style={{ gridArea: 'down' }}>
          <img src="/images/maze/dpad_down.png" alt="" />
        </button>
        <button type="button" aria-label="Move right" onClick={() => move(MazeGameDirection.Right)} disabled={isComplete} style={{ gridArea: 'right' }}>
          <img src="/images/maze/dpad_right.png" alt="" />
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
          message="Congratulations! You completed the maze!"
          onClose={() => setShowResult(false)}
        />
      )}
    </div>
  )
}
