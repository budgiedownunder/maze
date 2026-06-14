import { useEffect, useRef, useState } from 'react'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { SubjectSelector, type MazeOption, type SubjectSelection } from '../components/SubjectSelector'
import { Leaderboard, type BoardSubject } from '../components/Leaderboard'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { useTheme } from '../context/ThemeContext'
import { useToken, useAuth } from '../context/AuthContext'
import { getScoreHistory, getMazes, getPlay3dConfig } from '../api/client'
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
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
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
  // Busy cursor while any of the page's loads are in flight; cleared on
  // completion or failure.
  useBusyCursor(isLoadingSubjects || isResolving || isBoardLoading)
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
    launchPlay3dWithSettings(selection.mazeId, normalizeMazeGameSettings(maze?.game_settings ?? {}))
  }

  const showPlayer = selection?.gameType === 'play3d'
  // Nothing to launch when the Mazes type is selected but the player has none
  // (the maze id is empty); a Play-3D difficulty is always playable.
  const canPlay = selection != null && (selection.gameType === 'play3d' || selection.mazeId !== '')

  return (
    <div className="leaderboards-page">
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title app-header-title--with-icon">
          <img src={leaderboardsIcon} className="app-header-title-icon" alt="" aria-hidden="true" />
          Leaderboards
        </span>
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
      <main className="leaderboards-main">
        {isLoadingSubjects && <p aria-label="Loading">Loading…</p>}
        {!isLoadingSubjects && subjectsError && (
          <p className="error-msg" role="alert">{subjectsError}</p>
        )}
        {!isLoadingSubjects && !subjectsError && (
          <>
            <SubjectSelector
              mazes={mazes}
              value={selection}
              onChange={setSelection}
            >
              <button type="button" className="btn-primary leaderboard-play" onClick={handlePlay} disabled={!canPlay}>
                {hasPlayed ? '↻ Play Again' : 'Play'}
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
                onLoadingChange={setIsBoardLoading}
                onHasPlayedChange={setHasPlayed}
              />
            )}
            {!resolveError && !isResolving && !boardSubject && (
              <p className="leaderboard-empty">No winning scores yet</p>
            )}
          </>
        )}
      </main>
    </div>
  )
}
