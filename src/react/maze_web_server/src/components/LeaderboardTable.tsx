import type { ScoreEntry } from '../types/api'
import { formatElapsedMs } from '../utils/scores'

interface LeaderboardTableProps {
  rows: ScoreEntry[]
  currentUserId?: string
  // Show the Player column. True for global boards (Play 3D); false for
  // My-Mazes boards where every row is the caller.
  showPlayer: boolean
  isLoading: boolean
  isLoadingMore: boolean
  error: string | null
  hasMore: boolean
  onLoadMore: () => void
}

function formatCompleted(iso: string): string {
  const d = new Date(iso)
  return Number.isNaN(d.getTime()) ? '' : d.toLocaleString()
}

export function LeaderboardTable({
  rows,
  currentUserId,
  showPlayer,
  isLoading,
  isLoadingMore,
  error,
  hasMore,
  onLoadMore,
}: LeaderboardTableProps) {
  if (error) {
    return <p className="error-msg" role="alert">{error}</p>
  }
  if (isLoading) {
    return <p aria-label="Loading">Loading…</p>
  }
  if (rows.length === 0) {
    return <p className="leaderboard-empty">No winning scores yet</p>
  }

  return (
    <div className="leaderboard-board">
      <div className="leaderboard-table-scroll">
        <table className="leaderboard-table">
          <thead>
            <tr>
              <th scope="col" className="leaderboard-col-rank">#</th>
              {showPlayer && <th scope="col" className="leaderboard-col-player">Player</th>}
              <th scope="col" className="leaderboard-col-time">Time</th>
              <th scope="col" className="leaderboard-col-score">Score</th>
              <th scope="col" className="leaderboard-col-date">Completed</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => {
              const isMe = currentUserId != null && row.user_id === currentUserId
              // Only worth highlighting on a multi-player board (Play 3D); on a
              // My-Mazes board every row is the caller, so it would just be noise.
              const highlight = isMe && showPlayer
              return (
                <tr key={row.id} className={highlight ? 'leaderboard-row leaderboard-row--me' : 'leaderboard-row'}>
                  <td className="leaderboard-col-rank">{i + 1}</td>
                  {showPlayer && (
                    <td className="leaderboard-col-player">{row.username ?? '—'}</td>
                  )}
                  <td className="leaderboard-col-time">{formatElapsedMs(row.elapsed_ms)}</td>
                  <td className="leaderboard-col-score">{row.score}</td>
                  <td className="leaderboard-col-date">{formatCompleted(row.recorded_at)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      {hasMore && (
        <button
          type="button"
          className="btn-secondary leaderboard-load-more"
          onClick={onLoadMore}
          disabled={isLoadingMore}
        >
          {isLoadingMore ? 'Loading…' : 'Load more'}
        </button>
      )}
    </div>
  )
}
