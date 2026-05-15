import { useState } from 'react'

/// Difficulty labels, in display order. These are passed verbatim as the
/// `?difficulty=` query to `/game/`; the server maps them to presets via
/// `GET /api/v1/game/play3d-config`. This modal is display-only — it never
/// knows the per-difficulty maze size / timer / seed.
const DIFFICULTIES = ['easy', 'tricky', 'hard'] as const
type Difficulty = (typeof DIFFICULTIES)[number]

const DEFAULT_DIFFICULTY: Difficulty = 'easy'

interface Props {
  onPlay: (difficulty: Difficulty) => void
  onCancel: () => void
}

function label(d: Difficulty): string {
  return d.charAt(0).toUpperCase() + d.slice(1)
}

export function Play3dDifficultyModal({ onPlay, onCancel }: Props) {
  const [difficulty, setDifficulty] = useState<Difficulty>(DEFAULT_DIFFICULTY)

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    onPlay(difficulty)
  }

  return (
    <div role="dialog" aria-modal="true" aria-label="Choose Difficulty" className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">Choose Difficulty</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <div role="radiogroup" aria-label="Difficulty" className="modal-radio-group">
            {DIFFICULTIES.map(d => (
              <label key={d} className="modal-radio">
                <input
                  type="radio"
                  name="difficulty"
                  value={d}
                  checked={difficulty === d}
                  onChange={() => setDifficulty(d)}
                />
                <span>{label(d)}</span>
              </label>
            ))}
          </div>
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary">Play</button>
          </div>
        </form>
      </div>
    </div>
  )
}
