import { titleCaseWire } from '../utils/cellEntityStyles'
import { SKY_TYPES, type SkyType } from '../utils/mazeGameSettings'
import {
  FINISH_TYPES,
  DIFFICULTY_CHANGES,
  LEVEL_ALIGNMENTS,
  type FinishType,
  type DifficultyChange,
  type LevelAlignment,
} from '../utils/gameDefinitions'
import type { DefinitionLevelsFormValue, DefinitionTopLevelConfig } from '../utils/definitionConfig'

// The multi-level run field-group for the game-definition editor: how the level
// stack behaves. The level **count** lives on the editor's General tab (it is
// the single-vs-multi-level decision, and this whole tab is hidden while the
// count is 1), so it is not repeated here. Field names + wire values match the
// config's `levels` object; the consumer serializes them verbatim.

// The final-level scene override (`levels.top`): a value the perimeter select
// maps to, plus `''` for "inherit the base game's setting".
const PERIMETER_OPTIONS = [
  { value: 'inherit', label: 'Inherit' },
  { value: 'walled', label: 'Walled' },
  { value: 'open', label: 'Open' },
] as const

interface LevelsFieldsProps {
  value: DefinitionLevelsFormValue
  onChange: (patch: Partial<DefinitionLevelsFormValue>) => void
}

export function LevelsFields({ value, onChange }: LevelsFieldsProps) {
  const top = value.top
  const overrideTop = top !== null

  // Build the next `top` from a patch to one of its fields, dropping a key set
  // back to "inherit" so an absent field means inherit (matching the runtime).
  function patchTop(patch: Partial<DefinitionTopLevelConfig>) {
    const next: DefinitionTopLevelConfig = { ...(top ?? {}), ...patch }
    if (next.skyType == null) delete next.skyType
    if (next.perimeterWalls == null) delete next.perimeterWalls
    onChange({ top: next })
  }

  const perimeterValue = top?.perimeterWalls == null ? 'inherit' : top.perimeterWalls ? 'walled' : 'open'

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

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={overrideTop}
          // Toggling on seeds an empty override (every field inherits); off
          // clears it back to null so the final level looks like the rest.
          onChange={e => onChange({ top: e.target.checked ? {} : null })}
        />
        <span>Override final level appearance</span>
      </label>

      {overrideTop && (
        <>
          <label className="modal-stacked-input">
            Final Level Sky
            <select
              className="input"
              value={top?.skyType ?? ''}
              onChange={e => patchTop({ skyType: e.target.value === '' ? null : (e.target.value as SkyType) })}
            >
              <option value="">Inherit</option>
              {SKY_TYPES.map(s => (
                <option key={s} value={s}>{titleCaseWire(s)}</option>
              ))}
            </select>
          </label>

          <label className="modal-stacked-input">
            Final Level Perimeter Walls
            <select
              className="input"
              value={perimeterValue}
              onChange={e => {
                const v = e.target.value
                patchTop({ perimeterWalls: v === 'inherit' ? null : v === 'walled' })
              }}
            >
              {PERIMETER_OPTIONS.map(o => (
                <option key={o.value} value={o.value}>{o.label}</option>
              ))}
            </select>
          </label>
        </>
      )}
    </>
  )
}
