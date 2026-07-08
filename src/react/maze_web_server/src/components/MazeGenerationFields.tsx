import {
  MAX_DOOR_COUNT, MAX_ENEMY_COUNT, MAX_HEALTH_COUNT, MAX_TREASURE_COUNT,
  MAX_SPARE_DOOR_COUNT, MAX_SPARE_KEY_COUNT,
} from '../utils/validation'

// The parametric generation field-group for the game-definition editor: maze
// size + solution length + the feature counts. Unlike the maze Generate dialog
// this has **no start/finish positions and no grid** — a definition is generated
// from its (hidden, auto-minted) seed, so there is nothing to place. Values are
// strings (raw input text); the consumer validates + parses them with
// `validateMazeGenerationFields`. Field names match the `Play3dConfigResponse`
// config keys the editor assembles.

export interface MazeGenerationFieldsValue {
  rows: string
  cols: string
  minSolutionLength: string
  doorCount: string
  spareDoors: string
  spareKeys: string
  enemyCount: string
  healthCount: string
  treasureCount: string
}

// The count fields all share the same 0..max shape, so they render from a table.
const COUNT_FIELDS: { key: keyof MazeGenerationFieldsValue; label: string; max: number }[] = [
  { key: 'doorCount', label: 'Doors', max: MAX_DOOR_COUNT },
  { key: 'spareDoors', label: 'Spare Doors', max: MAX_SPARE_DOOR_COUNT },
  { key: 'spareKeys', label: 'Spare Keys', max: MAX_SPARE_KEY_COUNT },
  { key: 'enemyCount', label: 'Enemies', max: MAX_ENEMY_COUNT },
  { key: 'healthCount', label: 'Health', max: MAX_HEALTH_COUNT },
  { key: 'treasureCount', label: 'Treasure', max: MAX_TREASURE_COUNT },
]

interface MazeGenerationFieldsProps {
  value: MazeGenerationFieldsValue
  onChange: (patch: Partial<MazeGenerationFieldsValue>) => void
}

export function MazeGenerationFields({ value, onChange }: MazeGenerationFieldsProps) {
  return (
    <>
      <label>
        Rows
        <input
          type="number"
          className="input"
          value={value.rows}
          min={3}
          onChange={e => onChange({ rows: e.target.value })}
        />
      </label>
      <label>
        Columns
        <input
          type="number"
          className="input"
          value={value.cols}
          min={3}
          onChange={e => onChange({ cols: e.target.value })}
        />
      </label>
      <label>
        Min Solution Length
        <input
          type="number"
          className="input"
          value={value.minSolutionLength}
          min={1}
          onChange={e => onChange({ minSolutionLength: e.target.value })}
        />
      </label>

      {COUNT_FIELDS.map(({ key, label, max }) => (
        <label key={key}>
          {label}
          <input
            type="number"
            className="input"
            value={value[key]}
            min={0}
            max={max}
            onChange={e => onChange({ [key]: e.target.value } as Partial<MazeGenerationFieldsValue>)}
          />
        </label>
      ))}
    </>
  )
}
