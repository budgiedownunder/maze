import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { flushSync } from 'react-dom'
import { useParams, useNavigate, useBlocker } from 'react-router-dom'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { MazeGrid } from '../components/MazeGrid'
import { CellOverridePanel } from '../components/CellOverridePanel'
import { ConfirmModal } from '../components/ConfirmModal'
import { PromptModal } from '../components/PromptModal'
import { GenerateMazeModal } from '../components/GenerateMazeModal'
import { Play3dCustomLaunchModal } from '../components/Play3dCustomLaunchModal'
import { AlertModal } from '../components/AlertModal'
import { generateMaze, solveMaze, splitDefinition, buildDefinitionWithOverrides } from '../wasm/mazeWasm'
import type { GenerateOptions } from '../types/api'
import type { FeatureChar } from '../types/cellEntities'
import { useAppFeatures } from '../context/AppFeaturesContext'
import { useToken } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useMazeEditor } from '../hooks/useMazeEditor'
import { useWalkAnimation } from '../hooks/useWalkAnimation'
import { useWalkSpeed } from '../hooks/useWalkSpeed'
import { usePlayMaze, GameType } from '../hooks/usePlayMaze'
import { WalkSpeedControl } from '../components/WalkSpeedControl'
import { getMaze, createMaze, updateMaze } from '../api/client'
import { launchPlay3dWithSettings } from '../utils/play3dLaunch'
import { countKeysAndDoors, exceedsKeyDoorCap, MAX_TOTAL_FEATURES } from '../utils/validation'

const BLANK_GRID = Array.from({ length: 5 }, () => Array<string>(5).fill(' '))

// Narrows a grid cell char to a feature type that can carry a per-cell override.
function isFeatureChar(ch: string | undefined): ch is FeatureChar {
  return ch === 'E' || ch === 'H' || ch === 'K' || ch === 'D'
}

export function MazePage() {
  const { id } = useParams<{ id?: string }>()
  const token = useToken()
  const navigate = useNavigate()
  const { theme, toggleTheme } = useTheme()
  const menuVariant = useMenuVariant()
  const gridRef = useRef<HTMLDivElement>(null)

  const isNew = id === undefined

  // Touch-only devices have no Shift key, so any existing multi-cell selection is
  // treated as sticky: tapping a cell or header extends it rather than replacing it.
  // On pointer/keyboard devices, Shift (or range mode) is required to extend.
  const isTouchOnly = useMemo(
    () => typeof window.matchMedia === 'function' && !window.matchMedia('(hover: hover) and (pointer: fine)').matches,
    [],
  )

  const {
    grid, mazeName, mazeId, isDirty,
    activeCell, anchorCell, solution, isRangeMode, selectionStatus,
    initFromDefinition, markSaved,
    selectAll, activateCell, activateRow, activateCol,
    moveActive, moveActiveHome, moveActiveEnd,
    enableRangeMode, disableRangeMode,
    setWall, setStart, setFinish, setKey, setDoor, setEnemy, setHealth, clearCell,
    insertRowsBefore, deleteRows, insertColsBefore, deleteCols,
    canInsertRows, canInsertColumns,
    applyGenerated, applySolution, clearSolution,
    overrides, getOverride, setOverride, clearOverride, getOverridesList,
  } = useMazeEditor()
  const { max_maze_cells } = useAppFeatures()

  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notFound, setNotFound] = useState(false)

  // Save state
  const [isSaving, setIsSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [showSaveNameModal, setShowSaveNameModal] = useState(false)

  // Refresh state
  const [showRefreshConfirm, setShowRefreshConfirm] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [refreshError, setRefreshError] = useState<string | null>(null)

  // Generate state
  const [showGenerateModal, setShowGenerateModal] = useState(false)
  const [isGenerating, setIsGenerating] = useState(false)
  const [generateError, setGenerateError] = useState<string | null>(null)
  const [lastMinSpineLength, setLastMinSpineLength] = useState(1)

  // Reset remembered min spine length when the maze changes
  useEffect(() => { setLastMinSpineLength(1) }, [mazeId])

  // Solve state
  const [isSolving, setIsSolving] = useState(false)
  const [solveError, setSolveError] = useState<string | null>(null)

  // Play state
  const [maze3dLaunch, setMaze3dLaunch] = useState<{ id: string; name: string } | null>(null)
  const { play: playMaze, isChecking: isCheckingPlay, error: playCheckError, clearError: clearPlayCheckError } =
    usePlayMaze({ onLaunch3d: m => setMaze3dLaunch({ id: m.id, name: m.name }) })
  const [showPlayDirtyConfirm, setShowPlayDirtyConfirm] = useState(false)
  const [pendingPlayGameType, setPendingPlayGameType] = useState<GameType>(GameType.TwoD)

  // Walk speed (persisted to localStorage)
  const { speedRef, speedIndex, setSpeedIndex } = useWalkSpeed()

  // Walk animation state
  const { walkState, isWalking, startWalk, cancelWalk } = useWalkAnimation(speedRef)
  const isWalkInProgress = walkState !== null && !walkState.isComplete

  const isBusy = isSaving || isRefreshing || isGenerating || isSolving || isWalkInProgress || isCheckingPlay
  const hasUnsavedWork = isDirty || (isNew && mazeId === null)
  const canSave = hasUnsavedWork
  const canRefresh = isDirty && mazeId !== null

  // Cells carrying a per-cell override, for the grid badge.
  const overrideCells = useMemo(() => new Set(overrides.keys()), [overrides])

  // The single feature cell (E/H/K/D) whose override panel is shown, or null. Gated to
  // a single selected feature cell with no solution displayed and the editor idle.
  const overridePanelCell = useMemo(() => {
    if (!activeCell || anchorCell !== null) return null
    if (selectionStatus.hasSolution || isBusy) return null
    const ch = grid[activeCell.row]?.[activeCell.col]
    if (!isFeatureChar(ch)) return null
    return { row: activeCell.row, col: activeCell.col, cellType: ch }
  }, [activeCell, anchorCell, selectionStatus.hasSolution, isBusy, grid])

  useEffect(() => {
    if (isBusy) document.body.classList.add('is-busy')
    else document.body.classList.remove('is-busy')
    return () => document.body.classList.remove('is-busy')
  }, [isBusy])

  const blocker = useBlocker(hasUnsavedWork)
  const [showBlockerSaveModal, setShowBlockerSaveModal] = useState(false)

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
      .then(async maze => {
        // Split the canonical definition (char-or-array grid) into a pure-char grid
        // plus per-cell overrides via the WASM codec — JS never parses the wire form.
        const { grid, overrides } = await splitDefinition(JSON.stringify(maze))
        initFromDefinition(maze.id, maze.name, { grid }, overrides)
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

  function tooManyFeaturesMessage(): string {
    const { keys, doors } = countKeysAndDoors(grid)
    return (
      `This maze has ${keys} keys + ${doors} doors = ${keys + doors}, ` +
      `over the limit of ${MAX_TOTAL_FEATURES}. ` +
      `Remove some key or door cells before saving.`
    )
  }

  // Builds the canonical definition (pure-char grid merged with per-cell overrides)
  // for persistence. The char-or-array wire form is produced by the WASM codec, not
  // assembled in JS. Used by every save path so overrides always round-trip.
  function buildDefinition() {
    return buildDefinitionWithOverrides(grid, getOverridesList())
  }

  async function handleSaveNew(name: string) {
    if (!token) return
    if (exceedsKeyDoorCap(grid)) {
      setSaveError(tooManyFeaturesMessage())
      return
    }
    setIsSaving(true)
    setSaveError(null)
    try {
      const saved = await createMaze(token, { name, definition: await buildDefinition() })
      flushSync(() => {
        setShowSaveNameModal(false)
        markSaved(saved.id, saved.name)
      })
      navigate(`/mazes/${encodeURIComponent(saved.id)}`, { replace: true })
    } catch (ex: unknown) {
      setSaveError((ex as { message?: string }).message ?? 'Failed to save.')
    } finally {
      setIsSaving(false)
    }
  }

  async function handleSaveExisting() {
    if (!token || !mazeId) return
    if (exceedsKeyDoorCap(grid)) {
      setSaveError(tooManyFeaturesMessage())
      return
    }
    setIsSaving(true)
    setSaveError(null)
    try {
      await updateMaze(token, mazeId, { name: mazeName, definition: await buildDefinition() })
      markSaved(mazeId, mazeName)
    } catch (ex: unknown) {
      setSaveError((ex as { message?: string }).message ?? 'Failed to save.')
    } finally {
      setIsSaving(false)
    }
  }

  function handleSaveClick() {
    if (mazeId === null) {
      setSaveError(null)
      setShowSaveNameModal(true)
    } else {
      handleSaveExisting()
    }
  }

  function handleBlockerSave() {
    setSaveError(null)
    if (mazeId === null) {
      setShowBlockerSaveModal(true)
    } else {
      handleSaveExistingAndProceed()
    }
  }

  async function handleSaveExistingAndProceed() {
    if (!token || !mazeId) return
    setIsSaving(true)
    setSaveError(null)
    try {
      await updateMaze(token, mazeId, { name: mazeName, definition: await buildDefinition() })
      markSaved(mazeId, mazeName)
      blocker.proceed?.()
    } catch (ex: unknown) {
      setSaveError((ex as { message?: string }).message ?? 'Failed to save.')
    } finally {
      setIsSaving(false)
    }
  }

  async function handleBlockerSaveNew(name: string) {
    if (!token) return
    setIsSaving(true)
    setSaveError(null)
    try {
      const saved = await createMaze(token, { name, definition: await buildDefinition() })
      flushSync(() => {
        setShowBlockerSaveModal(false)
        markSaved(saved.id, saved.name)
      })
      blocker.proceed?.()
    } catch (ex: unknown) {
      setSaveError((ex as { message?: string }).message ?? 'Failed to save.')
    } finally {
      setIsSaving(false)
    }
  }

  async function handleGenerate(options: GenerateOptions) {
    setIsGenerating(true)
    setGenerateError(null)
    try {
      await new Promise<void>(r => requestAnimationFrame(() => r()))
      const definition = await generateMaze(options)
      setLastMinSpineLength(options.minSpineLength)
      setShowGenerateModal(false)
      applyGenerated(definition)
    } catch (ex: unknown) {
      setGenerateError((ex as { message?: string }).message ?? 'Generation failed.')
    } finally {
      setIsGenerating(false)
    }
  }

  async function handleSolve() {
    setIsSolving(true)
    setSolveError(null)
    try {
      await new Promise<void>(r => requestAnimationFrame(() => r()))
      const path = await solveMaze({ grid })
      applySolution(path)
    } catch (ex: unknown) {
      const msg = (ex as { message?: string }).message ?? 'Unknown error.'
      setSolveError(msg.charAt(0).toUpperCase() + msg.slice(1))
    } finally {
      setIsSolving(false)
    }
  }

  async function handleWalkSolution() {
    setIsSolving(true)
    setSolveError(null)
    try {
      await new Promise<void>(r => requestAnimationFrame(() => r()))
      const path = await solveMaze({ grid })
      setIsSolving(false)
      startWalk(path)
    } catch (ex: unknown) {
      const msg = (ex as { message?: string }).message ?? 'Unknown error.'
      setSolveError(msg.charAt(0).toUpperCase() + msg.slice(1))
      setIsSolving(false)
    }
  }

  function handleClearSolution() {
    cancelWalk()
    clearSolution()
  }

  function handlePlayClick(gameType: GameType) {
    if (mazeId && !isDirty) {
      void playMaze({ id: mazeId, name: mazeName, definition: { grid } }, gameType)
    } else if (mazeId && isDirty) {
      setPendingPlayGameType(gameType)
      setShowPlayDirtyConfirm(true)
    } else {
      // New/unsaved — open save modal; user clicks Play again after saving
      setShowSaveNameModal(true)
    }
  }

  async function handlePlayDirtyConfirm() {
    if (!token || !mazeId) return
    setShowPlayDirtyConfirm(false)
    setIsSaving(true)
    setSaveError(null)
    let savedId: string | null = null
    try {
      await updateMaze(token, mazeId, { name: mazeName, definition: await buildDefinition() })
      const id = mazeId
      flushSync(() => { markSaved(id, mazeName) })
      savedId = id
    } catch (ex: unknown) {
      setSaveError((ex as { message?: string }).message ?? 'Failed to save.')
    } finally {
      setIsSaving(false)
    }
    if (!savedId) return
    void playMaze({ id: savedId, name: mazeName, definition: { grid } }, pendingPlayGameType)
  }

  async function handleConfirmRefresh() {
    if (!token || !mazeId) return
    setShowRefreshConfirm(false)
    setIsRefreshing(true)
    setRefreshError(null)
    try {
      const maze = await getMaze(token, mazeId)
      const { grid: refreshedGrid, overrides } = await splitDefinition(JSON.stringify(maze))
      initFromDefinition(maze.id, maze.name, { grid: refreshedGrid }, overrides)
    } catch (ex: unknown) {
      setRefreshError((ex as { message?: string }).message ?? 'Failed to refresh.')
    } finally {
      setIsRefreshing(false)
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- signature dictated by MazeGrid's onCellDoubleClick prop type
  const handleCellDoubleClick = useCallback((_row: number, _col: number) => {
    if (!isTouchOnly) return
    if (isRangeMode) {
      disableRangeMode()
    } else {
      enableRangeMode()
    }
    gridRef.current?.focus()
  }, [isTouchOnly, isRangeMode, enableRangeMode, disableRangeMode])

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault()
        if (!isBusy) moveActive(-1, 0, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowDown':
        e.preventDefault()
        if (!isBusy) moveActive(1, 0, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowLeft':
        e.preventDefault()
        if (!isBusy) moveActive(0, -1, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'ArrowRight':
        e.preventDefault()
        if (!isBusy) moveActive(0, 1, e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'Home':
        e.preventDefault()
        if (!isBusy) moveActiveHome(e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'End':
        e.preventDefault()
        if (!isBusy) moveActiveEnd(e.shiftKey, e.ctrlKey || e.metaKey)
        break
      case 'w':
      case 'W':
        if (!isBusy && !selectionStatus.isAllWalls && !selectionStatus.hasSolution) setWall()
        break
      case 's':
      case 'S':
        if (!isBusy && selectionStatus.isSingleCell && !selectionStatus.isStart && !selectionStatus.hasSolution) setStart()
        break
      case 'f':
      case 'F':
        if (!isBusy && selectionStatus.isSingleCell && !selectionStatus.isFinish && !selectionStatus.hasSolution) setFinish()
        break
      case 'k':
      case 'K':
        if (!isBusy && !selectionStatus.hasSolution) setKey()
        break
      case 'd':
      case 'D':
        if (!isBusy && !selectionStatus.hasSolution) setDoor()
        break
      case 'e':
      case 'E':
        if (!isBusy && !selectionStatus.hasSolution) setEnemy()
        break
      case 'h':
      case 'H':
        if (!isBusy && !selectionStatus.hasSolution) setHealth()
        break
      case 'Delete':
      case 'Backspace':
        if (!isBusy && !selectionStatus.isEmpty && !selectionStatus.hasSolution) clearCell()
        break
    }
  }, [isBusy, moveActive, moveActiveHome, moveActiveEnd, setWall, setStart, setFinish, setKey, setDoor, setEnemy, setHealth, clearCell, selectionStatus])

  const headerTitle = isNew
    ? '(unsaved)'
    : isLoading ? '…' : mazeName || '…'

  return (
    <div className="maze-page">
      {showSaveNameModal && (
        <PromptModal
          title="Save Maze"
          label="Name"
          initialValue=""
          confirmLabel="Save"
          isLoading={isSaving}
          error={saveError}
          onConfirm={handleSaveNew}
          onCancel={() => { setShowSaveNameModal(false); setSaveError(null) }}
        />
      )}
      {showRefreshConfirm && (
        <ConfirmModal
          title="Discard changes?"
          message="Reloading will discard your unsaved changes."
          confirmLabel="Reload"
          isDangerous
          isLoading={isRefreshing}
          error={refreshError}
          onConfirm={handleConfirmRefresh}
          onCancel={() => { setShowRefreshConfirm(false); setRefreshError(null) }}
        />
      )}
      {blocker.state === 'blocked' && !showBlockerSaveModal && (
        <ConfirmModal
          title="Unsaved Changes"
          message="Do you want to save your changes?"
          confirmLabel="Save"
          isLoading={isSaving}
          error={saveError}
          secondaryAction={{ label: 'Discard', onClick: () => blocker.proceed(), isDangerous: true }}
          onConfirm={handleBlockerSave}
          onCancel={() => { blocker.reset(); setSaveError(null) }}
        />
      )}
      {blocker.state === 'blocked' && showBlockerSaveModal && (
        <PromptModal
          title="Save Maze"
          label="Name"
          initialValue=""
          confirmLabel="Save"
          isLoading={isSaving}
          error={saveError}
          onConfirm={handleBlockerSaveNew}
          onCancel={() => { setShowBlockerSaveModal(false); setSaveError(null); blocker.reset() }}
        />
      )}

      {showGenerateModal && (
        <GenerateMazeModal
          grid={grid}
          initialMinSpineLength={lastMinSpineLength}
          isLoading={isGenerating}
          error={generateError}
          onGenerate={handleGenerate}
          onCancel={() => { setShowGenerateModal(false); setGenerateError(null) }}
        />
      )}
      {solveError && (
        <AlertModal
          title="Unable to solve maze"
          message={solveError}
          onClose={() => setSolveError(null)}
        />
      )}
      {showPlayDirtyConfirm && (
        <ConfirmModal
          title="Unsaved Changes"
          message="You have unsaved changes. Save and play?"
          confirmLabel="Save & Play"
          isLoading={isSaving || isCheckingPlay}
          error={saveError}
          onConfirm={handlePlayDirtyConfirm}
          onCancel={() => { setShowPlayDirtyConfirm(false); setSaveError(null) }}
        />
      )}
      {playCheckError && (
        <AlertModal
          title="Cannot Play Maze"
          message={playCheckError}
          onClose={clearPlayCheckError}
        />
      )}
      {maze3dLaunch && (
        <Play3dCustomLaunchModal
          mazeName={maze3dLaunch.name}
          onCancel={() => setMaze3dLaunch(null)}
          onPlay={settings => launchPlay3dWithSettings(maze3dLaunch.id, settings)}
        />
      )}

      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">{headerTitle}</span>
        <div className="header-actions">
          {!isNew && (
            <button
              className="btn-icon"
              aria-label="Refresh"
              title="Refresh"
              disabled={!canRefresh || isBusy}
              onClick={() => setShowRefreshConfirm(true)}
            >
              <img src="/images/maze/refresh.png" alt="" aria-hidden="true" style={{ width: '1.1rem', height: '1.1rem' }} />
            </button>
          )}
          <button
            className="btn-icon"
            aria-label="Save"
            title="Save"
            disabled={!canSave || isBusy}
            onClick={handleSaveClick}
          >
            <img src="/images/icons/icon_save.png" alt="" aria-hidden="true" style={{ width: '1.1rem', height: '1.1rem' }} />
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

      <main className="maze-page-content">
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && notFound && <p>Maze not found.</p>}
        {!isLoading && error && (
          <p className="error-msg" role="alert">{error}</p>
        )}
        {saveError && !showSaveNameModal && (
          <p className="error-msg" role="alert">{saveError}</p>
        )}
        {refreshError && (
          <p className="error-msg" role="alert">{refreshError}</p>
        )}
        {!isLoading && !notFound && !error && grid.length > 0 && (
          <div className="maze-toolbar" aria-label="Maze editor toolbar">
            {/* noSelection disables all editing buttons when no cell is selected */}
            <button
              className="maze-toolbar-btn"
              title="Set Wall [W]"
              aria-label="Set Wall"
              disabled={activeCell === null || selectionStatus.isAllWalls || selectionStatus.hasSolution || isWalking}
              onClick={() => { setWall(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/wall_button.png" alt="Set Wall" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Start [S]"
              aria-label="Set Start"
              disabled={activeCell === null || !selectionStatus.isSingleCell || selectionStatus.isStart || selectionStatus.hasSolution || isWalking}
              onClick={() => { setStart(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/start_button.png" alt="Set Start" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Finish [F]"
              aria-label="Set Finish"
              disabled={activeCell === null || !selectionStatus.isSingleCell || selectionStatus.isFinish || selectionStatus.hasSolution || isWalking}
              onClick={() => { setFinish(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/finish_button.png" alt="Set Finish" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Key [K]"
              aria-label="Set Key"
              disabled={activeCell === null || selectionStatus.hasSolution || isWalking}
              onClick={() => { setKey(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/key_button.svg" alt="Set Key" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Door [D]"
              aria-label="Set Door"
              disabled={activeCell === null || selectionStatus.hasSolution || isWalking}
              onClick={() => { setDoor(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/door_button.svg" alt="Set Door" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Enemy [E]"
              aria-label="Set Enemy"
              disabled={activeCell === null || selectionStatus.hasSolution || isWalking}
              onClick={() => { setEnemy(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/enemy_button.svg" alt="Set Enemy" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Set Health [H]"
              aria-label="Set Health"
              disabled={activeCell === null || selectionStatus.hasSolution || isWalking}
              onClick={() => { setHealth(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/health_button.svg" alt="Set Health" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Clear Cells [DEL]"
              aria-label="Clear"
              disabled={activeCell === null || selectionStatus.isEmpty || selectionStatus.hasSolution || isWalking}
              onClick={() => { clearCell(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/clear_button.png" alt="Clear" />
            </button>
            <button
              className="maze-toolbar-btn"
              title={!canInsertRows && max_maze_cells !== null
                ? `Insert Rows Before — disabled: would exceed the ${max_maze_cells}-cell limit`
                : 'Insert Rows Before'}
              aria-label={!canInsertRows && max_maze_cells !== null
                ? `Insert Rows Before (disabled: would exceed the ${max_maze_cells}-cell limit)`
                : 'Insert Rows Before'}
              disabled={activeCell === null || !selectionStatus.allColumnsSelected || selectionStatus.hasSolution || isWalking || !canInsertRows}
              onClick={() => { insertRowsBefore(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/insert_rows_button.png" alt="Insert Row Before" />
            </button>
            <button
              className="maze-toolbar-btn"
              title={!canInsertColumns && max_maze_cells !== null
                ? `Insert Columns Before — disabled: would exceed the ${max_maze_cells}-cell limit`
                : 'Insert Columns Before'}
              aria-label={!canInsertColumns && max_maze_cells !== null
                ? `Insert Columns Before (disabled: would exceed the ${max_maze_cells}-cell limit)`
                : 'Insert Columns Before'}
              disabled={activeCell === null || !selectionStatus.allRowsSelected || selectionStatus.hasSolution || isWalking || !canInsertColumns}
              onClick={() => { insertColsBefore(); gridRef.current?.focus() }}
            >
              <img src="/images/maze/insert_columns_button.png" alt="Insert Column Before" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Delete"
              aria-label="Delete"
              disabled={
                activeCell === null ||
                (!selectionStatus.allColumnsSelected && !selectionStatus.allRowsSelected) ||
                (selectionStatus.allColumnsSelected && selectionStatus.allRowsSelected) ||
                selectionStatus.hasSolution ||
                isWalking
              }
              onClick={() => {
                if (selectionStatus.allColumnsSelected) deleteRows(); else deleteCols()
                gridRef.current?.focus()
              }}
            >
              <img src="/images/maze/delete_button.png" alt="Delete" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Generate"
              aria-label="Generate"
              disabled={selectionStatus.hasSolution || isBusy || isWalking}
              onClick={() => { setGenerateError(null); setShowGenerateModal(true) }}
            >
              <img src="/images/maze/generate_button.png" alt="Generate" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Solve"
              aria-label="Solve"
              disabled={selectionStatus.hasSolution || isBusy || isWalking}
              onClick={handleSolve}
            >
              <img src="/images/maze/solve_button.png" alt="Solve" />
            </button>
            <button
              className="maze-toolbar-btn"
              title="Walk Solution"
              aria-label="Walk Solution"
              disabled={selectionStatus.hasSolution || isBusy || isWalking}
              onClick={handleWalkSolution}
            >
              <img src="/images/maze/walk_solution_button.png" alt="Walk Solution" />
            </button>
            {(selectionStatus.hasSolution || isWalking) && (
              <button
                className="maze-toolbar-btn"
                title="Clear Solution"
                aria-label="Clear Solution"
                disabled={isSaving || isRefreshing || isGenerating || isSolving}
                onClick={handleClearSolution}
              >
                <img src="/images/maze/clear_solution_button.png" alt="Clear Solution" />
              </button>
            )}
            {isWalking && (
              <WalkSpeedControl
                speedIndex={speedIndex}
                onSpeedChange={setSpeedIndex}
              />
            )}
            <button
              type="button"
              className="maze-toolbar-btn"
              title="Play"
              aria-label="Play"
              disabled={isBusy}
              onClick={() => handlePlayClick(GameType.TwoD)}
            >
              <img src="/images/maze/play_button.png" alt="Play" />
            </button>
            <button
              type="button"
              className="maze-toolbar-btn"
              title="Play in 3D"
              aria-label="Play in 3D"
              disabled={isBusy}
              onClick={() => handlePlayClick(GameType.ThreeD)}
            >
              <img src="/images/maze/play_3d_button.png" alt="Play in 3D" />
            </button>
            {!isRangeMode && anchorCell === null && (
              <button
                className="maze-toolbar-btn maze-range-mode-btn"
                title="Select range"
                aria-label="Select range"
                disabled={activeCell === null}
                onClick={() => { enableRangeMode(); gridRef.current?.focus() }}
              >
                <img src="/images/maze/select_range_button.png" alt="Select range" />
              </button>
            )}
            {(isRangeMode || anchorCell !== null) && (
              <button
                className="maze-toolbar-btn maze-range-mode-btn"
                title="Done"
                aria-label="Done"
                onClick={() => { disableRangeMode(); gridRef.current?.focus() }}
              >
                <img src="/images/maze/done_select_range_button.png" alt="Done" />
              </button>
            )}
          </div>
        )}

        {!isLoading && !notFound && !error && grid.length > 0 && (
          <div className="maze-editor-body">
            <MazeGrid
              ref={gridRef}
              grid={grid}
              solution={solution}
              walkState={walkState}
              activeCell={activeCell}
              anchorCell={anchorCell}
              isRangeMode={isRangeMode}
              overrideCells={overrideCells}
              onCellClick={isBusy ? undefined : (row, col, shift) => activateCell(row, col, shift || (isTouchOnly && anchorCell !== null))}
              onCellDoubleClick={isBusy ? undefined : handleCellDoubleClick}
              onRowHeaderClick={isBusy ? undefined : (row, shift) => activateRow(row, shift || (isTouchOnly && anchorCell !== null))}
              onColHeaderClick={isBusy ? undefined : (col, shift) => activateCol(col, shift || (isTouchOnly && anchorCell !== null))}
              onCornerClick={isBusy ? undefined : () => selectAll()}
              onKeyDown={handleKeyDown}
            />
            {overridePanelCell && (
              <CellOverridePanel
                key={`${overridePanelCell.row},${overridePanelCell.col}`}
                cellType={overridePanelCell.cellType}
                row={overridePanelCell.row}
                col={overridePanelCell.col}
                override={getOverride(overridePanelCell.row, overridePanelCell.col)}
                onApply={entity => setOverride(overridePanelCell.row, overridePanelCell.col, entity)}
                onClear={() => clearOverride(overridePanelCell.row, overridePanelCell.col)}
              />
            )}
          </div>
        )}

        {activeCell !== null && (
          <div className="maze-shortcuts-hint">
            [W]&nbsp;Wall&nbsp;&nbsp;&nbsp;
            [S]&nbsp;Start&nbsp;&nbsp;&nbsp;
            [F]&nbsp;Finish&nbsp;&nbsp;&nbsp;
            [K]&nbsp;Key&nbsp;&nbsp;&nbsp;
            [D]&nbsp;Door&nbsp;&nbsp;&nbsp;
            [E]&nbsp;Enemy&nbsp;&nbsp;&nbsp;
            [H]&nbsp;Health&nbsp;&nbsp;&nbsp;
            [DEL]&nbsp;Clear&nbsp;&nbsp;&nbsp;
            [&#x2190;&#x2191;&#x2192;&#x2193;]&nbsp;Move&nbsp;&nbsp;&nbsp;
            [Shift]&nbsp;Range
          </div>
        )}
      </main>
    </div>
  )
}
