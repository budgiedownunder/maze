import type { ReactNode } from 'react'
import { LeaderboardGamePicker, type PickedGame } from './LeaderboardGamePicker'

export interface MazeOption {
  mazeId: string
  name: string
}

// What the player picked. The page turns this into a board subject: a maze board
// keys on the maze id, a 3D game board on the game's `def:<id>` challenge.
export type SubjectSelection =
  | { gameType: 'my-mazes'; mazeId: string }
  | { gameType: 'play3d'; game: PickedGame | null }

type GameType = SubjectSelection['gameType']

interface SubjectSelectorProps {
  mazes: MazeOption[]
  value: SubjectSelection | null
  onChange: (selection: SubjectSelection) => void
  // Optional trailing content rendered in the selector row (e.g. a Play button).
  children?: ReactNode
}

const GAME_TYPES: { value: GameType; label: string }[] = [
  { value: 'my-mazes', label: 'Mazes' },
  { value: 'play3d', label: '3D Games' },
]

// Cascading Game Type → game selector. Game Type is a fixed set; the game picker
// is all the player's mazes (Mazes) or the scoped, searchable stored-game picker
// (3D Games), whose board is that game's `def:<id>`.
export function SubjectSelector({ mazes, value, onChange, children }: SubjectSelectorProps) {
  const gameType: GameType = value?.gameType ?? 'my-mazes'

  function handleGameTypeChange(next: GameType) {
    if (next === 'play3d') {
      onChange({ gameType: 'play3d', game: null })
    } else {
      onChange({ gameType: 'my-mazes', mazeId: mazes[0]?.mazeId ?? '' })
    }
  }

  return (
    <div className="subject-selector">
      <select
        className="subject-select"
        aria-label="Game Type"
        value={gameType}
        onChange={e => handleGameTypeChange(e.target.value as GameType)}
      >
        {GAME_TYPES.map(t => (
          <option key={t.value} value={t.value}>{t.label}</option>
        ))}
      </select>
      <span className="subject-arrow" aria-hidden="true">→</span>
      {gameType === 'play3d' ? (
        <LeaderboardGamePicker
          value={value != null && value.gameType === 'play3d' ? value.game : null}
          onSelect={game => onChange({ gameType: 'play3d', game })}
        />
      ) : (
        <select
          className="subject-select"
          aria-label="Game"
          value={value != null && value.gameType === 'my-mazes' ? value.mazeId : ''}
          onChange={e => onChange({ gameType: 'my-mazes', mazeId: e.target.value })}
          disabled={mazes.length === 0}
        >
          {mazes.length === 0 ? (
            <option value="">(no mazes)</option>
          ) : (
            mazes.map(m => (
              <option key={m.mazeId} value={m.mazeId}>{m.name}</option>
            ))
          )}
        </select>
      )}
      {children}
    </div>
  )
}
