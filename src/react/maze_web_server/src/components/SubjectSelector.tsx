import { PLAY3D_DIFFICULTIES } from '../utils/scores'

export interface PlayedMaze {
  mazeId: string
  name: string
}

// What the player picked. The page turns this into a board subject — for
// `play3d` it resolves the difficulty's fixed seed via `getPlay3dConfig`.
export type SubjectSelection =
  | { gameType: 'my-mazes'; mazeId: string }
  | { gameType: 'play3d'; difficulty: string }

type GameType = SubjectSelection['gameType']

interface SubjectSelectorProps {
  playedMazes: PlayedMaze[]
  value: SubjectSelection | null
  onChange: (selection: SubjectSelection) => void
}

const GAME_TYPES: { value: GameType; label: string }[] = [
  { value: 'my-mazes', label: 'My Mazes' },
  { value: 'play3d', label: 'Play 3D' },
]

function titleCase(s: string): string {
  return s.length === 0 ? s : s.charAt(0).toUpperCase() + s.slice(1)
}

// Cascading Game Type → game selector. Game Type is a fixed set; the game
// dropdown is the player's played mazes (My Mazes) or the fixed curated
// difficulties (Play 3D).
export function SubjectSelector({ playedMazes, value, onChange }: SubjectSelectorProps) {
  const gameType: GameType = value?.gameType ?? 'my-mazes'
  const gameValue = value == null ? '' : value.gameType === 'play3d' ? value.difficulty : value.mazeId

  function handleGameTypeChange(next: GameType) {
    if (next === 'play3d') {
      onChange({ gameType: 'play3d', difficulty: PLAY3D_DIFFICULTIES[0] })
    } else {
      onChange({ gameType: 'my-mazes', mazeId: playedMazes[0]?.mazeId ?? '' })
    }
  }

  function handleGameChange(next: string) {
    onChange(
      gameType === 'play3d'
        ? { gameType: 'play3d', difficulty: next }
        : { gameType: 'my-mazes', mazeId: next },
    )
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
        <select
          className="subject-select"
          aria-label="Game"
          value={gameValue}
          onChange={e => handleGameChange(e.target.value)}
        >
          {PLAY3D_DIFFICULTIES.map(d => (
            <option key={d} value={d}>{titleCase(d)}</option>
          ))}
        </select>
      ) : (
        <select
          className="subject-select"
          aria-label="Game"
          value={gameValue}
          onChange={e => handleGameChange(e.target.value)}
          disabled={playedMazes.length === 0}
        >
          {playedMazes.length === 0 ? (
            <option value="">(no mazes played)</option>
          ) : (
            playedMazes.map(m => (
              <option key={m.mazeId} value={m.mazeId}>{m.name}</option>
            ))
          )}
        </select>
      )}
    </div>
  )
}
