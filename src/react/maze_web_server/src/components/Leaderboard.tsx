import { useCallback, useEffect, useState } from 'react'
import { getLeaderboard } from '../api/client'
import { useLeaderboard } from '../hooks/useLeaderboard'
import { LeaderboardTable } from './LeaderboardTable'
import type { ScoreMetric } from '../types/api'

// A leaderboard subject: exactly one of a stored maze or a curated challenge.
export type BoardSubject = { mazeId: string } | { challenge: string }

interface LeaderboardProps {
  token: string
  subject: BoardSubject
  currentUserId?: string
  // True for global (Play 3D) boards → request + show usernames; false for
  // My-Mazes boards where every row is the caller.
  showPlayer: boolean
  // Bumping this re-fetches the current board (folded into the board key) — used
  // by the host's Refresh button. Unchanged across a normal selection.
  reloadNonce?: number
  // Reports the first-page load state so a host can show a busy cursor.
  onLoadingChange?: (loading: boolean) => void
  // Reports whether the caller has a run on the currently loaded board, so a
  // host can label its Play button "Play Again".
  onHasPlayedChange?: (hasPlayed: boolean) => void
  // Reports the number of rows on the loaded board, so a host can show controls
  // (e.g. a Reset button) only when the board is non-empty.
  onRowCountChange?: (count: number) => void
}

const METRIC_TABS: { metric: ScoreMetric; label: string }[] = [
  { metric: 'time', label: 'Fastest Time' },
  { metric: 'score', label: 'Highest Score' },
]

function subjectKey(subject: BoardSubject): string {
  return 'mazeId' in subject ? `m:${subject.mazeId}` : `c:${subject.challenge}`
}

// Reusable board view: ranking-metric tabs over a single subject's paged
// leaderboard. Switching the metric (or the subject) reloads from the top.
export function Leaderboard({ token, subject, currentUserId, showPlayer, reloadNonce, onLoadingChange, onHasPlayedChange, onRowCountChange }: LeaderboardProps) {
  const [metric, setMetric] = useState<ScoreMetric>('time')
  const key = `${metric}|${subjectKey(subject)}|${reloadNonce ?? 0}`

  const fetchPage = useCallback(
    (limit: number, offset: number) => {
      const query = 'mazeId' in subject ? { mazeId: subject.mazeId } : { challenge: subject.challenge }
      return getLeaderboard(token, { ...query, metric, includeUsernames: showPlayer, limit, offset })
    },
    [token, subject, metric, showPlayer],
  )

  const board = useLeaderboard(key, fetchPage)

  // Surface the first-page load to the host (busy cursor), and clear it on unmount.
  useEffect(() => { onLoadingChange?.(board.isLoading) }, [board.isLoading, onLoadingChange])
  useEffect(() => () => onLoadingChange?.(false), [onLoadingChange])

  // Report whether the caller appears on the loaded board (drives the host's
  // Play / Play Again label).
  const hasPlayed = currentUserId != null && board.rows.some(r => r.user_id === currentUserId)
  useEffect(() => { onHasPlayedChange?.(hasPlayed) }, [hasPlayed, onHasPlayedChange])
  useEffect(() => () => onHasPlayedChange?.(false), [onHasPlayedChange])

  // Report the loaded row count (drives the host's Reset button visibility).
  const rowCount = board.rows.length
  useEffect(() => { onRowCountChange?.(rowCount) }, [rowCount, onRowCountChange])
  useEffect(() => () => onRowCountChange?.(0), [onRowCountChange])

  return (
    <div className="leaderboard-board-view">
      <div className="leaderboard-metric-tabs" role="tablist" aria-label="Ranking metric">
        {METRIC_TABS.map(tab => (
          <button
            key={tab.metric}
            type="button"
            role="tab"
            aria-selected={metric === tab.metric}
            className={metric === tab.metric ? 'leaderboard-tab leaderboard-tab--active' : 'leaderboard-tab'}
            onClick={() => setMetric(tab.metric)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <LeaderboardTable
        rows={board.rows}
        currentUserId={currentUserId}
        showPlayer={showPlayer}
        isLoading={board.isLoading}
        isLoadingMore={board.isLoadingMore}
        error={board.error}
        hasMore={board.hasMore}
        onLoadMore={board.loadMore}
      />
    </div>
  )
}
