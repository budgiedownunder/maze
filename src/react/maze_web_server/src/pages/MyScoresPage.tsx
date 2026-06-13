import { useEffect, useRef, useState } from 'react'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { SubjectSelector, type PlayedMaze, type SubjectSelection } from '../components/SubjectSelector'
import { ScoreBoard, type BoardSubject } from '../components/ScoreBoard'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { useTheme } from '../context/ThemeContext'
import { useToken, useAuth } from '../context/AuthContext'
import { getScoreHistory, getMazes, getPlay3dConfig } from '../api/client'
import { buildChallenge } from '../utils/scores'
import type { ScoreEntry } from '../types/api'

// Cap on history pages scanned to discover the player's played mazes — bounds
// the work for a very active player; the server caps each page at 100.
const MAX_DISCOVERY_PAGES = 25
const DISCOVERY_PAGE = 100

function parseDifficulty(challenge: string): string {
  return challenge.split(':')[0]
}

// The board to show first: the subject of the player's most recent run, else a
// usable default (the Easy global board) so the page is never inert.
function defaultSelection(mostRecent: ScoreEntry | undefined, played: PlayedMaze[]): SubjectSelection {
  if (mostRecent?.maze_id) return { gameType: 'my-mazes', mazeId: mostRecent.maze_id }
  if (mostRecent?.challenge) return { gameType: 'play3d', difficulty: parseDifficulty(mostRecent.challenge) }
  if (played.length > 0) return { gameType: 'my-mazes', mazeId: played[0].mazeId }
  return { gameType: 'play3d', difficulty: 'easy' }
}

export function MyScoresPage() {
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()
  const token = useToken()
  const { profile } = useAuth()

  const [playedMazes, setPlayedMazes] = useState<PlayedMaze[]>([])
  const [selection, setSelection] = useState<SubjectSelection | null>(null)
  const [isLoadingSubjects, setIsLoadingSubjects] = useState(true)
  const [subjectsError, setSubjectsError] = useState<string | null>(null)

  const [boardSubject, setBoardSubject] = useState<BoardSubject | null>(null)
  const [isResolving, setIsResolving] = useState(false)
  const [resolveError, setResolveError] = useState<string | null>(null)
  const [isBoardLoading, setIsBoardLoading] = useState(false)
  // Busy cursor while any of the page's loads are in flight; cleared on
  // completion or failure.
  useBusyCursor(isLoadingSubjects || isResolving || isBoardLoading)
  // difficulty → fixed seed; the seeds don't change, so resolve each once.
  const seedCache = useRef<Map<string, number>>(new Map())

  // Discover the player's played mazes + pick the initial subject.
  useEffect(() => {
    if (!token) return
    let cancelled = false
    setIsLoadingSubjects(true)
    setSubjectsError(null)
    ;(async () => {
      try {
        const mazeIds = new Set<string>()
        let mostRecent: ScoreEntry | undefined
        let offset = 0
        for (let page = 0; page < MAX_DISCOVERY_PAGES; page++) {
          const resp = await getScoreHistory(token, { limit: DISCOVERY_PAGE, offset })
          if (page === 0) mostRecent = resp.scores[0]
          for (const row of resp.scores) if (row.maze_id) mazeIds.add(row.maze_id)
          if (!resp.has_more) break
          offset += DISCOVERY_PAGE
        }
        const mazes = await getMazes(token, false)
        if (cancelled) return
        const nameById = new Map(mazes.map(m => [m.id, m.name]))
        const played: PlayedMaze[] = [...mazeIds]
          .map(id => ({ mazeId: id, name: nameById.get(id) ?? id }))
          .sort((a, b) => a.name.localeCompare(b.name))
        setPlayedMazes(played)
        setSelection(defaultSelection(mostRecent, played))
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

  const showPlayer = selection?.gameType === 'play3d'

  return (
    <div className="my-scores-page">
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <span className="app-header-title">My Scores</span>
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
      <main className="my-scores-main">
        {isLoadingSubjects && <p aria-label="Loading">Loading…</p>}
        {!isLoadingSubjects && subjectsError && (
          <p className="error-msg" role="alert">{subjectsError}</p>
        )}
        {!isLoadingSubjects && !subjectsError && (
          <>
            <SubjectSelector
              playedMazes={playedMazes}
              value={selection}
              onChange={setSelection}
            />
            {resolveError && <p className="error-msg" role="alert">{resolveError}</p>}
            {!resolveError && isResolving && <p aria-label="Loading">Loading…</p>}
            {!resolveError && !isResolving && boardSubject && token && (
              <ScoreBoard
                token={token}
                subject={boardSubject}
                currentUserId={profile?.id}
                showPlayer={!!showPlayer}
                onLoadingChange={setIsBoardLoading}
              />
            )}
            {!resolveError && !isResolving && !boardSubject && (
              <p className="score-empty">No win scores yet</p>
            )}
          </>
        )}
      </main>
    </div>
  )
}
