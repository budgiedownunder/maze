import { titleCaseWire } from '../utils/cellEntityStyles'
import {
  FINISH_TYPES,
  DIFFICULTY_CHANGES,
  LEVEL_ALIGNMENTS,
  type FinishType,
  type DifficultyChange,
  type LevelAlignment,
} from '../utils/gameDefinitions'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'

// The multi-level run field-group for the game-definition editor: how the level
// stack behaves. The level **count** lives on the editor's General tab (it is
// the single-vs-multi-level decision, and this whole tab is hidden while the
// count is 1), so it is not repeated here. The final-level scene override
// (`levels.top`) lives on the Scene tab (see `FinalLevelOverrideFields`). Field
// names + wire values match the config's `levels` object; the consumer
// serializes them verbatim.

interface LevelsFieldsProps {
  value: DefinitionLevelsFormValue
  onChange: (patch: Partial<DefinitionLevelsFormValue>) => void
}

export function LevelsFields({ value, onChange }: LevelsFieldsProps) {
  return (
    <>
      <label className="modal-stacked-input">
        Finish Type
        <select
          className="input"
          value={value.finishType}
          onChange={e => onChange({ finishType: e.target.value as FinishType })}
        >
          {FINISH_TYPES.map(f => (
            <option key={f} value={f}>{titleCaseWire(f)}</option>
          ))}
        </select>
      </label>

      <label className="modal-stacked-input">
        Difficulty Change
        <select
          className="input"
          value={value.difficultyChange}
          onChange={e => onChange({ difficultyChange: e.target.value as DifficultyChange })}
        >
          {DIFFICULTY_CHANGES.map(d => (
            <option key={d} value={d}>{titleCaseWire(d)}</option>
          ))}
        </select>
      </label>

      <label className="modal-stacked-input">
        Level Alignment
        <select
          className="input"
          value={value.alignment}
          onChange={e => onChange({ alignment: e.target.value as LevelAlignment })}
        >
          {LEVEL_ALIGNMENTS.map(a => (
            <option key={a} value={a}>{titleCaseWire(a)}</option>
          ))}
        </select>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.resetBag}
          onChange={e => onChange({ resetBag: e.target.checked })}
        />
        <span>Reset item bag each level</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.taper}
          onChange={e => onChange({ taper: e.target.checked })}
        />
        <span>Taper upper levels</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.perimeterRandom}
          onChange={e => onChange({ perimeterRandom: e.target.checked })}
        />
        <span>Randomise perimeter each level</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.hideCompletedEnemies}
          onChange={e => onChange({ hideCompletedEnemies: e.target.checked })}
        />
        <span>Hide cleared-level enemies</span>
      </label>
    </>
  )
}
