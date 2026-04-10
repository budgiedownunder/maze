import { useEffect, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { MazeGrid } from '../components/MazeGrid'
import { useToken } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useMazeEditor } from '../hooks/useMazeEditor'
import { getMaze } from '../api/client'

const BLANK_GRID = Array.from({ length: 7 }, () => Array<string>(7).fill(' '))

export function MazePage() {
  const { id } = useParams<{ id?: string }>()
  const token = useToken()
  const { theme, toggleTheme } = useTheme()
  const menuVariant = useMenuVariant()
  const gridRef = useRef<HTMLDivElement>(null)

  const isNew = id === undefined

  const { grid, mazeName, activeCell, anchorCell, solution, initFromDefinition } = useMazeEditor()

  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notFound, setNotFound] = useState(false)

  useEffect(() => {
    if (isNew) {
      initFromDefinition(null, '', { grid: BLANK_GRID })
      return
    }
    if (!token) return
    setIsLoading(true)
    setError(null)
    setNotFound(false)
    getMaze(token, id!)
      .then(maze => {
        initFromDefinition(maze.id, maze.name, maze.definition)
      })
      .catch(err => {
        const status = (err as { status?: number }).status
        if (status === 404) {
          setNotFound(true)
        } else {
          setError((err as Error).message || 'Failed to load maze')
        }
      })
      .finally(() => setIsLoading(false))
  }, [token, id, isNew, initFromDefinition])

  const headerTitle = isNew
    ? '(unsaved)'
    : isLoading ? '…' : mazeName || '…'

  return (
    <div className="maze-page">
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">{headerTitle}</span>
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

      <main className="maze-page-content">
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && notFound && <p>Maze not found.</p>}
        {!isLoading && error && (
          <p className="error-msg" role="alert">{error}</p>
        )}
        {!isLoading && !notFound && !error && grid.length > 0 && (
          <MazeGrid
            ref={gridRef}
            grid={grid}
            solution={solution}
            activeCell={activeCell}
            anchorCell={anchorCell}
          />
        )}
        {/* empty toolbar — buttons added in Step 7 */}
        <div className="maze-toolbar" />
      </main>
    </div>
  )
}
