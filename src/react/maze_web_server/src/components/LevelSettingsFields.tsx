import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'

// The per-level behaviour toggles shown in the Advanced tab's Levels group (only
// for a multi-level game): whether the item bag resets each level, whether the
// perimeter walls are re-randomised per level, and whether enemies on cleared
// levels are hidden. Operates on the shared `levels` value shape; the interim
// finish rig (`finishType`) lives on the Objects tab and the difficulty/
// alignment/taper progression on the Layout tab.

interface LevelSettingsFieldsProps {
  value: DefinitionLevelsFormValue
  onChange: (patch: Partial<DefinitionLevelsFormValue>) => void
}

export function LevelSettingsFields({ value, onChange }: LevelSettingsFieldsProps) {
  return (
    <>
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
