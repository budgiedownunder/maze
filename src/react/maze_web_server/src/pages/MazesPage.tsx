import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { ConfirmModal } from '../components/ConfirmModal'
import { PromptModal } from '../components/PromptModal'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useTheme } from '../context/ThemeContext'
import { useToken } from '../context/AuthContext'
import { getMazes, deleteMaze, updateMaze, createMaze } from '../api/client'
import type { Maze } from '../types/api'

export function MazesPage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
  const token = useToken()
  const navigate = useNavigate()

  const [mazes, setMazes] = useState<Maze[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [refreshCount, setRefreshCount] = useState(0)

  const [mazeToDelete, setMazeToDelete] = useState<Maze | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)

  const [mazeToRename, setMazeToRename] = useState<Maze | null>(null)
  const [renameError, setRenameError] = useState<string | null>(null)
  const [isRenaming, setIsRenaming] = useState(false)

  const [mazeToDuplicate, setMazeToDuplicate] = useState<Maze | null>(null)
  const [duplicateError, setDuplicateError] = useState<string | null>(null)
  const [isDuplicating, setIsDuplicating] = useState(false)

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

  async function handleConfirmDelete() {
    if (!mazeToDelete) return
    setIsDeleting(true)
    document.body.classList.add('is-busy')
    try {
      await deleteMaze(token!, mazeToDelete.id)
      setMazeToDelete(null)
      setDeleteError(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setDeleteError((ex as { message?: string }).message ?? 'Failed to delete maze.')
    } finally {
      setIsDeleting(false)
      document.body.classList.remove('is-busy')
    }
  }

  async function handleConfirmRename(name: string) {
    if (!mazeToRename) return
    setIsRenaming(true)
    document.body.classList.add('is-busy')
    try {
      await updateMaze(token!, mazeToRename.id, { name, definition: mazeToRename.definition })
      setMazeToRename(null)
      setRenameError(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setRenameError((ex as { message?: string }).message ?? 'Failed to rename maze.')
    } finally {
      setIsRenaming(false)
      document.body.classList.remove('is-busy')
    }
  }

  function validateDuplicateName(name: string): string | null {
    return mazes.some(m => m.name.toLowerCase() === name.toLowerCase())
      ? 'A maze with that name already exists.'
      : null
  }

  function validateRenameName(name: string): string | null {
    return mazes.some(m => m.id !== mazeToRename?.id && m.name.toLowerCase() === name.toLowerCase())
      ? 'A maze with that name already exists.'
      : null
  }

  async function handleConfirmDuplicate(name: string) {
    if (!mazeToDuplicate) return
    setIsDuplicating(true)
    document.body.classList.add('is-busy')
    try {
      await createMaze(token!, { name, definition: mazeToDuplicate.definition })
      setMazeToDuplicate(null)
      setDuplicateError(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setDuplicateError((ex as { message?: string }).message ?? 'Failed to duplicate maze.')
    } finally {
      setIsDuplicating(false)
      document.body.classList.remove('is-busy')
    }
  }

  return (
    <div className="mazes-page">
      {mazeToDelete && (
        <ConfirmModal
          title="Delete Maze"
          message={`Delete "${mazeToDelete.name}"? This cannot be undone.`}
          confirmLabel="Delete"
          isDangerous
          isLoading={isDeleting}
          error={deleteError}
          onConfirm={handleConfirmDelete}
          onCancel={() => { setMazeToDelete(null); setDeleteError(null) }}
        />
      )}
      {mazeToDuplicate && (
        <PromptModal
          title="Duplicate Maze"
          label="Name"
          initialValue={`Copy of ${mazeToDuplicate.name}`}
          confirmLabel="Duplicate"
          validate={validateDuplicateName}
          isLoading={isDuplicating}
          error={duplicateError}
          onConfirm={handleConfirmDuplicate}
          onCancel={() => { setMazeToDuplicate(null); setDuplicateError(null) }}
        />
      )}
      {mazeToRename && (
        <PromptModal
          title="Rename Maze"
          label="New name"
          initialValue={mazeToRename.name}
          confirmLabel="Rename"
          validate={validateRenameName}
          isLoading={isRenaming}
          error={renameError}
          onConfirm={handleConfirmRename}
          onCancel={() => { setMazeToRename(null); setRenameError(null) }}
        />
      )}
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">Mazes</span>
        <div className="header-actions">
          <button
            className="btn-icon"
            onClick={() => navigate('/mazes/new')}
            aria-label="New maze"
            title="New maze"
          >
            <img src="/images/icons/icon_new.png" alt="New maze" style={{ width: '1.1rem', height: '1.1rem' }} />
          </button>
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
                  onClick={() => navigate(`/mazes/${encodeURIComponent(maze.id)}`)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={e => {
                    if (e.key === 'Enter' || e.key === ' ') navigate(`/mazes/${encodeURIComponent(maze.id)}`)
                  }}
                >
                  <img src="/images/maze/maze.png" className="maze-item-icon" alt="" aria-hidden="true" />
                  <div className="maze-item-text">
                    <span className="maze-item-name">{maze.name}</span>
                    <span className="maze-item-subtitle">{rows} {rows === 1 ? 'row' : 'rows'} × {cols} {cols === 1 ? 'column' : 'columns'}</span>
                  </div>
                  <div className="maze-item-actions">
                    <button
                      type="button"
                      className="maze-item-action btn-secondary"
                      onClick={e => { e.stopPropagation(); setMazeToRename(maze); setRenameError(null) }}
                      aria-label={`Rename ${maze.name}`}
                    >
                      <img src="/images/icons/icon_rename.png" alt="" aria-hidden="true" />
                      <span className="maze-item-action-label">Rename</span>
                    </button>
                    <button
                      type="button"
                      className="maze-item-action btn-secondary"
                      onClick={e => { e.stopPropagation(); setMazeToDuplicate(maze); setDuplicateError(null) }}
                      aria-label={`Duplicate ${maze.name}`}
                    >
                      <img src="/images/icons/icon_duplicate.png" alt="" aria-hidden="true" />
                      <span className="maze-item-action-label">Duplicate</span>
                    </button>
                    <button
                      type="button"
                      className="maze-item-action btn-danger-outline"
                      onClick={e => { e.stopPropagation(); setMazeToDelete(maze); setDeleteError(null) }}
                      aria-label={`Delete ${maze.name}`}
                    >
                      <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" />
                      <span className="maze-item-action-label">Delete</span>
                    </button>
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
