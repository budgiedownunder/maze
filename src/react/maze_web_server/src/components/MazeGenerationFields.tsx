// The parametric size + solution field-group for the game-definition editor:
// maze size (rows/cols) and the minimum start→finish distance. The feature
// counts (doors/spare/enemies/health/treasure) that share this value shape live
// on the editor's Objects tab (see `FeatureCountFields`), grouped with their
// related object dropdowns. Values are strings (raw input text); the consumer
// validates + parses them with `validateMazeGenerationFields`. Field names match
// the `Play3dConfigResponse` config keys the editor assembles.

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

interface MazeGenerationFieldsProps {
  value: MazeGenerationFieldsValue
  onChange: (patch: Partial<MazeGenerationFieldsValue>) => void
  // Upper bound for the rows/cols spinners. Optional because only the game
  // definition editor has one — an authored 2D maze is bounded by the store's
  // cell cap, which is a product of both fields and so cannot be an attribute.
  // Cosmetic either way: `max` bounds the arrows, not what can be typed, and
  // `validateMazeGenerationFields` is what refuses the value.
  maxDimension?: number
}

export function MazeGenerationFields({ value, onChange, maxDimension }: MazeGenerationFieldsProps) {
  return (
    <>
      <label>
        Rows
        <input
          type="number"
          className="input"
          value={value.rows}
          min={3}
          max={maxDimension}
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
          max={maxDimension}
          onChange={e => onChange({ cols: e.target.value })}
        />
      </label>
      <label>
        Min Start to Finish Distance
        <input
          type="number"
          className="input"
          value={value.minSolutionLength}
          min={0}
          onChange={e => onChange({ minSolutionLength: e.target.value })}
        />
      </label>
    </>
  )
}
