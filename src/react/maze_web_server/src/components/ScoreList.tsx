import type { ScoreEntry } from '../types/api'
import { formatElapsedMs } from '../utils/scores'

interface ScoreListProps {
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

export function ScoreList({
  rows,
  currentUserId,
  showPlayer,
  isLoading,
  isLoadingMore,
  error,
  hasMore,
  onLoadMore,
}: ScoreListProps) {
  if (error) {
    return <p className="error-msg" role="alert">{error}</p>
  }
  if (isLoading) {
    return <p aria-label="Loading">Loading…</p>
  }
  if (rows.length === 0) {
    return <p className="score-empty">No winning scores yet</p>
  }

  return (
    <div className="score-board">
      <div className="score-table-scroll">
        <table className="score-table">
          <thead>
            <tr>
              <th scope="col" className="score-col-rank">#</th>
              {showPlayer && <th scope="col" className="score-col-player">Player</th>}
              <th scope="col" className="score-col-time">Time</th>
              <th scope="col" className="score-col-score">Score</th>
              <th scope="col" className="score-col-date">Completed</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => {
              const isMe = currentUserId != null && row.user_id === currentUserId
              // Only worth highlighting on a multi-player board (Play 3D); on a
              // My-Mazes board every row is the caller, so it would just be noise.
              const highlight = isMe && showPlayer
              return (
                <tr key={row.id} className={highlight ? 'score-row score-row--me' : 'score-row'}>
                  <td className="score-col-rank">{i + 1}</td>
                  {showPlayer && (
                    <td className="score-col-player">{row.username ?? '—'}</td>
                  )}
                  <td className="score-col-time">{formatElapsedMs(row.elapsed_ms)}</td>
                  <td className="score-col-score">{row.score}</td>
                  <td className="score-col-date">{formatCompleted(row.recorded_at)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      {hasMore && (
        <button
          type="button"
          className="btn-secondary score-load-more"
          onClick={onLoadMore}
          disabled={isLoadingMore}
        >
          {isLoadingMore ? 'Loading…' : 'Load more'}
        </button>
      )}
    </div>
  )
}
