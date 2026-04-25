import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { solveMaze } from '../wasm/mazeWasm'
import type { Maze } from '../types/api'

export const GameType = {
  TwoD: 0,
  ThreeD: 1,
} as const
export type GameType = (typeof GameType)[keyof typeof GameType]

export function usePlayMaze() {
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
