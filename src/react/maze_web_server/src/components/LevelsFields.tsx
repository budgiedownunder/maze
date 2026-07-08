import { titleCaseWire } from '../utils/cellEntityStyles'
import { FINISH_TYPES, type FinishType } from '../utils/gameDefinitions'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'

// The Levels-tab field-group for the game-definition editor: the interim finish
// rig plus the per-level reset / perimeter / enemy toggles. The level **count**
// lives on the General tab, the difficulty/alignment/taper progression on the
// Layout tab's Levels group (see `LevelProgressionFields`), and the final-level
// scene override (`levels.top`) on the Scene tab (see `FinalLevelOverrideFields`).
// Field names + wire values match the config's `levels` object; the consumer
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
