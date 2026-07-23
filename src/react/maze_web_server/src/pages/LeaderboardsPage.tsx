import { useEffect, useMemo, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { SubjectSelector, type MazeOption, type SubjectSelection } from '../components/SubjectSelector'
import { Leaderboard, type BoardSubject } from '../components/Leaderboard'
import { DailyBoardDatePicker } from '../components/DailyBoardDatePicker'
import { ConfirmModal } from '../components/ConfirmModal'
import { AlertModal } from '../components/AlertModal'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { usePlayMaze, GameType } from '../hooks/usePlayMaze'
import { useToken, useAuth } from '../context/AuthContext'
import { getScoreHistory, getMazes, getGameDefinition, resetLeaderboard } from '../api/client'
import { gameChallengeKey, gameIdFromChallenge, todayUtc } from '../utils/gameDefinitions'
import { launchPlay3dWithSettings, launchDefinition } from '../utils/play3dLaunch'
import { normalizeMazeGameSettings } from '../utils/mazeGameSettings'
import type { Maze, ScoreEntry } from '../types/api'
import leaderboardsIcon from '../assets/leaderboards.svg'

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

// The board to show first: the subject of the player's most recent run — their
// maze, or the stored 3D game behind a `def:<id>` challenge (resolved through
// the access-checked play-fetch, so a since-deleted or no-longer-visible game
// falls through) — else their first maze, else nothing selected.
async function defaultSelection(
  token: string,
  mostRecent: ScoreEntry | undefined,
  mazes: MazeOption[],
): Promise<SubjectSelection> {
  if (mostRecent?.maze_id) {
    const id = resolveMazeId(mostRecent.maze_id, mazes)
    if (id) return { gameType: 'my-mazes', mazeId: id }
  }
  const gameId = mostRecent?.challenge ? gameIdFromChallenge(mostRecent.challenge) : null
  if (gameId) {
    try {
      const def = await getGameDefinition(token, gameId)
      return { gameType: 'play3d', game: { id: def.id, name: def.name, ownerId: def.ownerId, rotation: def.rotation } }
    } catch {
      // Gone or no longer accessible — fall through to a maze.
    }
  }
  if (mazes.length > 0) return { gameType: 'my-mazes', mazeId: mazes[0].mazeId }
  return { gameType: 'play3d', game: null }
}

export function LeaderboardsPage() {
  const token = useToken()
  const { profile } = useAuth()

  const [mazes, setMazes] = useState<MazeOption[]>([])
  // Full maze records (with game_settings) kept so the Play button can launch a
  // personal maze in 3D with its saved settings.
  const [allMazes, setAllMazes] = useState<Maze[]>([])
  const [selection, setSelection] = useState<SubjectSelection | null>(null)
  // Which day's board to show for a Daily game (`yyyy-mm-dd`, UTC) — ignored for
  // Static games. Reset to today whenever the picked game changes.
  const [selectedDate, setSelectedDate] = useState<string>(todayUtc())
  const [isLoadingSubjects, setIsLoadingSubjects] = useState(true)
  const [subjectsError, setSubjectsError] = useState<string | null>(null)

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
  useBusyCursor(isLoadingSubjects || isBoardLoading || isCheckingPlay)

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
        const options: MazeOption[] = loaded
          .map(m => ({ mazeId: m.id, name: m.name }))
          .sort((a, b) => a.name.localeCompare(b.name))
        const initial = await defaultSelection(token, history.scores[0], options)
        if (cancelled) return
        setAllMazes(loaded)
        setMazes(options)
        setSelection(initial)
      } catch (err) {
        if (!cancelled) setSubjectsError((err as Error).message || 'Failed to load your scores')
      } finally {
        if (!cancelled) setIsLoadingSubjects(false)
      }
    })()
    return () => { cancelled = true }
  }, [token])

  // The picked 3D game (null for a maze subject or before a game is chosen).
  const pickedGame = selection?.gameType === 'play3d' ? selection.game : null
  const isDailyGame = pickedGame?.rotation === 'daily'

  // A newly-picked game starts on today's board; the date control then browses
  // past days (Daily only).
  useEffect(() => {
    setSelectedDate(todayUtc())
  }, [pickedGame?.id])

  // The board subject follows the selection directly — a maze board keys on the
  // maze id, a 3D game board on the game's challenge (the picker already resolved
  // the game, so nothing needs fetching): `def:<id>` for a Static game,
  // `def:<id>:<date>` for the selected day of a Daily one. Memoised so the board
  // isn't handed a fresh subject object every render.
  const boardSubject = useMemo<BoardSubject | null>(() => {
    if (selection == null) return null
    if (selection.gameType === 'my-mazes') {
      return selection.mazeId ? { mazeId: selection.mazeId } : null
    }
    return selection.game
      ? { challenge: gameChallengeKey(selection.game.id, selection.game.rotation, selectedDate) }
      : null
  }, [selection, selectedDate])

  // Launch the selected subject in 3D: a personal maze with its saved settings,
  // or the stored game (the host page fetches its config by id). No prompt.
  function handlePlay() {
    if (selection == null) return
    if (selection.gameType === 'play3d') {
      if (selection.game) launchDefinition(selection.game.id)
      return
    }
    const maze = allMazes.find(m => m.id === selection.mazeId)
    if (!maze) return
    // `playMaze` runs the solvability check (rejecting an empty / cleared maze)
    // and, on success, fires `onLaunch3d` above to launch with the saved settings.
    void playMaze(maze, GameType.ThreeD)
  }

  // The Reset button shows only when the board has rows AND the caller may clear
  // it: a 3D game's board belongs to that game, so its owner may reset it — as
  // may an admin, who curates the featured set; a personal maze board → its owner
  // (the page lists only the caller's own mazes). The server enforces this
  // regardless; the gate just hides a button the caller can't use.
  const canReset =
    boardSubject != null &&
    boardRowCount > 0 &&
    (selection?.gameType === 'play3d'
      ? !!profile?.is_admin || selection.game?.ownerId === profile?.id
      : true)

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
  // (the maze id is empty), or when no 3D game has been picked yet.
  const canPlay =
    selection != null &&
    (selection.gameType === 'play3d' ? selection.game != null : selection.mazeId !== '')

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
            <div className="leaderboard-filters" role="group" aria-label="Leaderboard filters">
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
              {isDailyGame && pickedGame && token && (
                <DailyBoardDatePicker
                  token={token}
                  gameId={pickedGame.id}
                  value={selectedDate}
                  onChange={setSelectedDate}
                />
              )}
            </div>
            {boardSubject && token && (
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
            {!boardSubject && (
              <p className="leaderboard-empty">
                {selection?.gameType === 'play3d'
                  ? 'Choose a game to see its leaderboard.'
                  : 'No winning scores yet'}
              </p>
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
