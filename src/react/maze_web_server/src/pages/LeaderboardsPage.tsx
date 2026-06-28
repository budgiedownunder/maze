import { useEffect, useRef, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { SubjectSelector, type MazeOption, type SubjectSelection } from '../components/SubjectSelector'
import { Leaderboard, type BoardSubject } from '../components/Leaderboard'
import { ConfirmModal } from '../components/ConfirmModal'
import { AlertModal } from '../components/AlertModal'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { usePlayMaze, GameType } from '../hooks/usePlayMaze'
import { useToken, useAuth } from '../context/AuthContext'
import { getScoreHistory, getMazes, getPlay3dConfig, resetLeaderboard } from '../api/client'
import { buildChallenge } from '../utils/scores'
import { launchPlay3dWithSettings, launchPlay3dCurated } from '../utils/play3dLaunch'
import { normalizeMazeGameSettings } from '../utils/mazeGameSettings'
import type { Maze, ScoreEntry } from '../types/api'
import leaderboardsIcon from '../assets/leaderboards.svg'

function parseDifficulty(challenge: string): string {
  return challenge.split(':')[0]
}

function basename(id: string): string {
  return id.split(/[\\/]/).pop() ?? id
}

// Resolve a history maze_id to a maze in the list. FileStore ids are full file
// paths, so a score row's stored maze_id may not byte-match the current id from
// getMazes — match by exact id first, then by filename.
function resolveMazeId(historyId: string, mazes: MazeOption[]): string | undefined {
  const exact = mazes.find(m => m.mazeId === historyId)
  if (exact) return exact.mazeId
  const bn = basename(historyId)
  return mazes.find(m => basename(m.mazeId) === bn)?.mazeId
}

// The board to show first: the subject of the player's most recent run (when it
// maps to a current maze / difficulty), else the first maze, else the Easy
// global board — so the page is never inert.
function defaultSelection(mostRecent: ScoreEntry | undefined, mazes: MazeOption[]): SubjectSelection {
  if (mostRecent?.maze_id) {
    const id = resolveMazeId(mostRecent.maze_id, mazes)
    if (id) return { gameType: 'my-mazes', mazeId: id }
  }
  if (mostRecent?.challenge) return { gameType: 'play3d', difficulty: parseDifficulty(mostRecent.challenge) }
  if (mazes.length > 0) return { gameType: 'my-mazes', mazeId: mazes[0].mazeId }
  return { gameType: 'play3d', difficulty: 'easy' }
}

export function LeaderboardsPage() {
  const token = useToken()
  const { profile } = useAuth()

  const [mazes, setMazes] = useState<MazeOption[]>([])
  // Full maze records (with game_settings) kept so the Play button can launch a
  // personal maze in 3D with its saved settings.
  const [allMazes, setAllMazes] = useState<Maze[]>([])
  const [selection, setSelection] = useState<SubjectSelection | null>(null)
  const [isLoadingSubjects, setIsLoadingSubjects] = useState(true)
  const [subjectsError, setSubjectsError] = useState<string | null>(null)

  const [boardSubject, setBoardSubject] = useState<BoardSubject | null>(null)
  const [isResolving, setIsResolving] = useState(false)
  const [resolveError, setResolveError] = useState<string | null>(null)
  const [isBoardLoading, setIsBoardLoading] = useState(false)
  // Whether the caller has a run on the current board → Play vs "Play Again".
  const [hasPlayed, setHasPlayed] = useState(false)
  // Bumped by the Refresh button (and after a reset) to force the board to re-fetch.
  const [refreshNonce, setRefreshNonce] = useState(0)
  // Number of rows on the loaded board — the Reset button shows only when > 0.
  const [boardRowCount, setBoardRowCount] = useState(0)
  // Reset flow: a confirm modal gates the destructive clear.
  const [isConfirmingReset, setIsConfirmingReset] = useState(false)
  const [isResetting, setIsResetting] = useState(false)
  const [resetError, setResetError] = useState<string | null>(null)
  // Play a personal maze through the shared solvability check
  const { play: playMaze, isChecking: isCheckingPlay, error: playCheckError, clearError: clearPlayCheckError } =
    usePlayMaze({
      onLaunch3d: maze => launchPlay3dWithSettings(maze.id, normalizeMazeGameSettings(maze.game_settings ?? {})),
    })
  // Busy cursor while any of the page's loads are in flight; cleared on
  // completion or failure.
  useBusyCursor(isLoadingSubjects || isResolving || isBoardLoading || isCheckingPlay)
  // difficulty → fixed seed; the seeds don't change, so resolve each once.
  const seedCache = useRef<Map<string, number>>(new Map())

  // List all the player's mazes (the Mazes dropdown shows every maze, scored or
  // not) + pick the initial subject from their most-recent run.
  useEffect(() => {
    if (!token) return
    let cancelled = false
    setIsLoadingSubjects(true)
    setSubjectsError(null)
    ;(async () => {
      try {
        // `true` → mazes carry game_settings, needed by the Play button.
        const loaded = await getMazes(token, true)
        const history = await getScoreHistory(token, { limit: 1 })
        if (cancelled) return
        const options: MazeOption[] = loaded
          .map(m => ({ mazeId: m.id, name: m.name }))
          .sort((a, b) => a.name.localeCompare(b.name))
        setAllMazes(loaded)
        setMazes(options)
        setSelection(defaultSelection(history.scores[0], options))
      } catch (err) {
        if (!cancelled) setSubjectsError((err as Error).message || 'Failed to load your scores')
      } finally {
        if (!cancelled) setIsLoadingSubjects(false)
      }
    })()
    return () => { cancelled = true }
  }, [token])

  // Resolve the selection into a board subject. My-Mazes is direct; Play-3D
  // resolves the difficulty's fixed seed (cached) into a challenge key.
  useEffect(() => {
    if (selection == null) { setBoardSubject(null); return }
    setResolveError(null)
    if (selection.gameType === 'my-mazes') {
      setBoardSubject(selection.mazeId ? { mazeId: selection.mazeId } : null)
      return
    }
    const difficulty = selection.difficulty
    const cachedSeed = seedCache.current.get(difficulty)
    if (cachedSeed != null) {
      setBoardSubject({ challenge: buildChallenge(difficulty, cachedSeed) })
      return
    }
    let cancelled = false
    setIsResolving(true)
    getPlay3dConfig(difficulty)
      .then(config => {
        if (cancelled) return
        seedCache.current.set(difficulty, config.seed)
        setBoardSubject({ challenge: buildChallenge(difficulty, config.seed) })
      })
      .catch(err => {
        if (cancelled) return
        setResolveError((err as Error).message || 'Failed to load difficulty')
        setBoardSubject(null)
      })
      .finally(() => { if (!cancelled) setIsResolving(false) })
    return () => { cancelled = true }
  }, [selection])

  // Launch the selected subject in 3D: a personal maze with its saved settings,
  // or a curated difficulty (server resolves the preset). No prompt.
  function handlePlay() {
    if (selection == null) return
    if (selection.gameType === 'play3d') {
      launchPlay3dCurated(selection.difficulty)
      return
    }
    const maze = allMazes.find(m => m.id === selection.mazeId)
    if (!maze) return
    // `playMaze` runs the solvability check (rejecting an empty / cleared maze)
    // and, on success, fires `onLaunch3d` above to launch with the saved settings.
    void playMaze(maze, GameType.ThreeD)
  }

  // The Reset button shows only when the board has rows AND the caller may clear
  // it: a Play-3D (challenge) board is global → admins only; a personal maze board
  // → its owner (the page lists only the caller's own mazes). The server enforces
  // this regardless; the gate just hides a button the caller can't use.
  const canReset =
    boardSubject != null &&
    boardRowCount > 0 &&
    ('challenge' in boardSubject ? !!profile?.is_admin : true)

  async function handleConfirmReset() {
    if (boardSubject == null || token == null) return
    setIsResetting(true)
    setResetError(null)
    try {
      await resetLeaderboard(token, boardSubject)
      setIsConfirmingReset(false)
      setRefreshNonce(n => n + 1) // re-fetch the now-empty board
    } catch (err) {
      setResetError((err as Error).message || 'Failed to reset leaderboard')
    } finally {
      setIsResetting(false)
    }
  }

  const showPlayer = selection?.gameType === 'play3d'
  // Nothing to launch when the Mazes type is selected but the player has none
  // (the maze id is empty); a Play-3D difficulty is always playable.
  const canPlay = selection != null && (selection.gameType === 'play3d' || selection.mazeId !== '')

  return (
    <div className="leaderboards-page">
      <AppHeader title="Leaderboards" titleIcon={leaderboardsIcon}>
        {canReset && (
          <button
            type="button"
            className="theme-toggle leaderboard-reset"
            onClick={() => { setResetError(null); setIsConfirmingReset(true) }}
            aria-label="Reset leaderboard"
            title="Reset leaderboard"
          >
            <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" width={18} height={18} />
          </button>
        )}
        <button
          className="theme-toggle"
          onClick={() => setRefreshNonce(n => n + 1)}
          aria-label="Refresh"
          title="Refresh"
        >
          ↻
        </button>
      </AppHeader>
      <main className="leaderboards-main">
        {isLoadingSubjects && <p aria-label="Loading">Loading…</p>}
        {!isLoadingSubjects && subjectsError && (
          <p className="error-msg" role="alert">{subjectsError}</p>
        )}
        {!isLoadingSubjects && !subjectsError && (
          <>
            <span className="subject-label" aria-hidden="true">Game</span>
            <SubjectSelector
              mazes={mazes}
              value={selection}
              onChange={setSelection}
            >
              <button type="button" className="btn-primary leaderboard-play" onClick={handlePlay} disabled={!canPlay}>
                {hasPlayed ? '↻ Play Again' : '▶ Play'}
              </button>
            </SubjectSelector>
            {resolveError && <p className="error-msg" role="alert">{resolveError}</p>}
            {!resolveError && isResolving && <p aria-label="Loading">Loading…</p>}
            {!resolveError && !isResolving && boardSubject && token && (
              <Leaderboard
                token={token}
                subject={boardSubject}
                currentUserId={profile?.id}
                showPlayer={!!showPlayer}
                reloadNonce={refreshNonce}
                onLoadingChange={setIsBoardLoading}
                onHasPlayedChange={setHasPlayed}
                onRowCountChange={setBoardRowCount}
              />
            )}
            {!resolveError && !isResolving && !boardSubject && (
              <p className="leaderboard-empty">No winning scores yet</p>
            )}
          </>
        )}
      </main>
      {isConfirmingReset && (
        <ConfirmModal
          title="Reset leaderboard"
          message="This permanently deletes every score on this leaderboard. This cannot be undone."
          confirmLabel="Reset"
          isDangerous
          isLoading={isResetting}
          error={resetError}
          onConfirm={handleConfirmReset}
          onCancel={() => setIsConfirmingReset(false)}
        />
      )}
      {playCheckError && (
        <AlertModal title="Cannot Play Maze" message={playCheckError} onClose={clearPlayCheckError} />
      )}
    </div>
  )
}
