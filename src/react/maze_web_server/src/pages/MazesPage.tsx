import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'
import { useToken } from '../context/AuthContext'
import { getMazes } from '../api/client'
import type { Maze } from '../types/api'

export function MazesPage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
  const token = useToken()
  const navigate = useNavigate()

  const [mazes, setMazes] = useState<Maze[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [refreshCount, setRefreshCount] = useState(0)

  useEffect(() => {
    if (!token) return
    setIsLoading(true)
    setError(null)
    document.body.classList.add('is-busy')
    getMazes(token, true)
      .then(setMazes)
      .catch(err => setError((err as Error).message || 'Failed to load mazes'))
      .finally(() => {
        setIsLoading(false)
        document.body.classList.remove('is-busy')
      })
  }, [token, refreshCount])

  return (
    <div>
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">Mazes</span>
        <div className="header-actions">
          <button
            className="btn-icon"
            onClick={() => setRefreshCount(c => c + 1)}
            aria-label="Refresh"
            title="Refresh"
          >
            <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
          </button>
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
      <main className="maze-list-page">
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && (
          <p className="error-msg" role="alert">{error}</p>
        )}
        {!isLoading && !error && mazes.length === 0 && (
          <p>No mazes yet.</p>
        )}
        {!isLoading && !error && mazes.length > 0 && (
          <ul className="maze-list">
            {mazes.map(maze => {
              const rows = maze.definition.grid.length
              const cols = maze.definition.grid[0]?.length ?? 0
              return (
                <li
                  key={maze.id}
                  className="maze-list-item"
                  onClick={() => navigate(`/mazes/${maze.id}`)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={e => {
                    if (e.key === 'Enter' || e.key === ' ') navigate(`/mazes/${maze.id}`)
                  }}
                >
                  <img src="/images/maze/maze.png" className="maze-item-icon" alt="" aria-hidden="true" />
                  <div className="maze-item-text">
                    <span className="maze-item-name">{maze.name}</span>
                    <span className="maze-item-subtitle">{rows} {rows === 1 ? 'row' : 'rows'} × {cols} {cols === 1 ? 'column' : 'columns'}</span>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </main>
    </div>
  )
}
