import type { BoardDateOption } from '../utils/gameDefinitions'

interface Props {
  // The selectable days: "Today" pinned first, then the days that have boards
  // (most-recent first). Built by the page from `getBoardDates`.
  options: BoardDateOption[]
  // Selected day, `yyyy-mm-dd` (UTC) — keys the board to `def:<id>:<date>`.
  value: string
  onChange: (date: string) => void
}

// Date control for a daily game's leaderboard: a dropdown of the days known to
// have a board. A native calendar can't disable the sparse days with no scores,
// so this is a list, not a calendar — Today is always offered (pinned first),
// followed by the past days that have runs.
export function DailyBoardDatePicker({ options, value, onChange }: Props) {
  return (
    <label className="daily-board-date">
      <span className="daily-board-label">Day</span>
      <select
        className="subject-select"
        value={value}
        onChange={e => onChange(e.target.value)}
      >
        {options.map(o => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </label>
  )
}
