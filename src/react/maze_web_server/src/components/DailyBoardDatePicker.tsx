import { useEffect, useState } from 'react'
import { getBoardDates } from '../api/client'
import { todayUtc } from '../utils/gameDefinitions'

interface Props {
  token: string
  // The daily game whose dated boards to browse.
  gameId: string
  // Selected day, `yyyy-mm-dd` (UTC) — keys the board to `def:<id>:<date>`.
  value: string
  onChange: (date: string) => void
}

// Date control for a daily game's leaderboard: a native date input (capped at
// today, UTC — the day boundary the server uses) plus a quick-pick row of the
// days that actually have runs (from `getBoardDates`). Browsers can't style the
// days-with-runs in the calendar itself, so they're surfaced as chips beside the
// input so a user can jump straight to a populated board.
export function DailyBoardDatePicker({ token, gameId, value, onChange }: Props) {
  // Days-with-runs tagged with the game they belong to, so a switch to a new
  // game shows no chips until its own dates load (derived below — avoids a
  // synchronous reset in the effect).
  const [loaded, setLoaded] = useState<{ id: string; dates: string[] }>({ id: '', dates: [] })

  // Load the days-with-runs for this game (most recent first). A failure just
  // leaves the quick-picks empty — the date input still works.
  useEffect(() => {
    let cancelled = false
    getBoardDates(token, gameId)
      .then(res => { if (!cancelled) setLoaded({ id: gameId, dates: res.dates }) })
      .catch(() => { if (!cancelled) setLoaded({ id: gameId, dates: [] }) })
    return () => { cancelled = true }
  }, [token, gameId])

  const dates = loaded.id === gameId ? loaded.dates : []

  return (
    <div className="daily-board-picker">
      <label className="daily-board-date">
        <span className="daily-board-label">Day</span>
        <input
          type="date"
          className="input"
          max={todayUtc()}
          value={value}
          // Ignore a cleared input — an empty date can't key a board.
          onChange={e => { if (e.target.value) onChange(e.target.value) }}
        />
      </label>
      {dates.length > 0 && (
        <div className="daily-board-quickpicks" role="group" aria-label="Days with scores">
          {dates.map(d => (
            <button
              key={d}
              type="button"
              className={d === value ? 'daily-board-chip daily-board-chip--active' : 'daily-board-chip'}
              aria-pressed={d === value}
              onClick={() => onChange(d)}
            >
              {d}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
