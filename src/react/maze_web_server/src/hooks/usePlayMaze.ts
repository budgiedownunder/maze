import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { solveMaze } from '../wasm/mazeWasm'
import type { Maze } from '../types/api'

export const GameType = {
  TwoD: 0,
  ThreeD: 1,
} as const
export type GameType = (typeof GameType)[keyof typeof GameType]

interface UsePlayMazeOptions {
  /// Called after a successful solvability check for a `GameType.ThreeD`
  /// launch instead of navigating directly. The page receiving the
  /// callback typically opens the MazeGameSettingsModal with this
  /// maze, then on the modal's Play action does the actual navigation
  /// to `/game/?id=…`. When this callback is absent the hook falls back
  /// to the legacy direct-navigation behaviour.
  onLaunch3d?: (maze: Maze) => void
}

export function usePlayMaze(opts: UsePlayMazeOptions = {}) {
  const navigate = useNavigate()
  const [isChecking, setIsChecking] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function play(maze: Maze, gameType: GameType) {
    setError(null)
    setIsChecking(true)
    document.body.classList.add('is-busy')
    try {
      await solveMaze(maze.definition)
      if (gameType === GameType.TwoD) {
        navigate('/play/' + encodeURIComponent(maze.id))
      } else if (opts.onLaunch3d) {
        opts.onLaunch3d(maze)
      } else {
        window.location.href = '/game/?id=' + encodeURIComponent(maze.id)
      }
    } catch (ex: unknown) {
      const msg = (ex as { message?: string }).message ?? 'Unknown error.'
      setError(msg.charAt(0).toUpperCase() + msg.slice(1))
    } finally {
      setIsChecking(false)
      document.body.classList.remove('is-busy')
    }
  }

  return { play, isChecking, error, clearError: () => setError(null) }
}
