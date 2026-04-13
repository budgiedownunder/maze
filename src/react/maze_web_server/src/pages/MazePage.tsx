import { useCallback, useEffect, useRef, useState } from 'react'
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

  const {
    grid, mazeName, activeCell, anchorCell, solution, isRangeMode, selectionStatus,
    initFromDefinition,
    selectAll, activateCell, activateRow, activateCol,
    moveActive, moveActiveHome, moveActiveEnd,
    enableRangeMode, disableRangeMode,
    setWall, setStart, setFinish, clearCell,
  } = useMazeEditor()

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

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault()
        moveActive(-1, 0, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowDown':
        e.preventDefault()
        moveActive(1, 0, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowLeft':
        e.preventDefault()
        moveActive(0, -1, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowRight':
        e.preventDefault()
        moveActive(0, 1, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'Home':
        e.preventDefault()
        moveActiveHome(e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'End':
        e.preventDefault()
        moveActiveEnd(e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'w':
      case 'W':
        if (!selectionStatus.isAllWalls && !selectionStatus.hasSolution) setWall()
        break
      case 's':
      case 'S':
        if (selectionStatus.isSingleCell && !selectionStatus.isStart && !selectionStatus.hasSolution) setStart()
        break
      case 'f':
      case 'F':
        if (selectionStatus.isSingleCell && !selectionStatus.isFinish && !selectionStatus.hasSolution) setFinish()
        break
      case 'Delete':
      case 'Backspace':
        if (!selectionStatus.isEmpty && !selectionStatus.hasSolution) clearCell()
        break
    }
  }, [moveActive, moveActiveHome, moveActiveEnd, setWall, setStart, setFinish, clearCell, selectionStatus])

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
        {activeCell !== null && (
          <div className="maze-toolbar" aria-label="Maze editor toolbar">
            <button
              className="maze-toolbar-btn"
              title="Set Wall"
              aria-label="Set Wall"
              disabled={selectionStatus.isAllWalls || selectionStatus.hasSolution}
              onClick={() => { setWall(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/wall_button.png" alt="Set Wall" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Start"
              aria-label="Set Start"
              disabled={!selectionStatus.isSingleCell || selectionStatus.isStart || selectionStatus.hasSolution}
              onClick={() => { setStart(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/start_button.png" alt="Set Start" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Finish"
              aria-label="Set Finish"
              disabled={!selectionStatus.isSingleCell || selectionStatus.isFinish || selectionStatus.hasSolution}
              onClick={() => { setFinish(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/finish_button.png" alt="Set Finish" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Clear"
              aria-label="Clear"
              disabled={selectionStatus.isEmpty || selectionStatus.hasSolution}
              onClick={() => { clearCell(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/clear_button.png" alt="Clear" />
            </button>
            {!isRangeMode && (
              <button
                className="maze-toolbar-btn maze-range-mode-btn"
                title="Select range"
                aria-label="Select range"
                onClick={() => { enableRangeMode(); gridRef.current?.focus() }}
              >
                <img src="/images/maze/select_range_button.png" alt="Select range" />
              </button>
            )}
            {isRangeMode && (
              <button
                className="maze-toolbar-btn maze-range-mode-btn maze-range-mode-btn--done"
                title="Done"
                aria-label="Done"
                onClick={() => { disableRangeMode(); gridRef.current?.focus() }}
              >
                Done
              </button>
            )}
          </div>
        )}

        {!isLoading && !notFound && !error && grid.length > 0 && (
          <MazeGrid
            ref={gridRef}
            grid={grid}
            solution={solution}
            activeCell={activeCell}
            anchorCell={anchorCell}
            onCellClick={(row, col, shift) => activateCell(row, col, shift)}
            onRowHeaderClick={(row, shift) => activateRow(row, shift)}
            onColHeaderClick={(col, shift) => activateCol(col, shift)}
            onCornerClick={() => selectAll()}
            onKeyDown={handleKeyDown}
          />
        )}
      </main>
    </div>
  )
}
