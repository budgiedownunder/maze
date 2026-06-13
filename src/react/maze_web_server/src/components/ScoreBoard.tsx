import { useCallback, useEffect, useState } from 'react'
import { getLeaderboard } from '../api/client'
import { useScoreBoard } from '../hooks/useScoreBoard'
import { ScoreList } from './ScoreList'
import type { ScoreMetric } from '../types/api'

// A leaderboard subject: exactly one of a stored maze or a curated challenge.
export type BoardSubject = { mazeId: string } | { challenge: string }

interface ScoreBoardProps {
  token: string
  subject: BoardSubject
  currentUserId?: string
  // True for global (Play 3D) boards → request + show usernames; false for
  // My-Mazes boards where every row is the caller.
  showPlayer: boolean
  // Reports the first-page load state so a host can show a busy cursor.
  onLoadingChange?: (loading: boolean) => void
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
export function ScoreBoard({ token, subject, currentUserId, showPlayer, onLoadingChange }: ScoreBoardProps) {
  const [metric, setMetric] = useState<ScoreMetric>('time')
  const key = `${metric}|${subjectKey(subject)}`

  const fetchPage = useCallback(
    (limit: number, offset: number) => {
      const query = 'mazeId' in subject ? { mazeId: subject.mazeId } : { challenge: subject.challenge }
      return getLeaderboard(token, { ...query, metric, includeUsernames: showPlayer, limit, offset })
    },
    [token, subject, metric, showPlayer],
  )

  const board = useScoreBoard(key, fetchPage)

  // Surface the first-page load to the host (busy cursor), and clear it on unmount.
  useEffect(() => { onLoadingChange?.(board.isLoading) }, [board.isLoading, onLoadingChange])
  useEffect(() => () => onLoadingChange?.(false), [onLoadingChange])

  return (
    <div className="score-board-view">
      <div className="score-metric-tabs" role="tablist" aria-label="Ranking metric">
        {METRIC_TABS.map(tab => (
          <button
            key={tab.metric}
            type="button"
            role="tab"
            aria-selected={metric === tab.metric}
            className={metric === tab.metric ? 'score-tab score-tab--active' : 'score-tab'}
            onClick={() => setMetric(tab.metric)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <ScoreList
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
