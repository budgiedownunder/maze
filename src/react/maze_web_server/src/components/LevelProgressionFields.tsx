import { titleCaseWire } from '../utils/cellEntityStyles'
import {
  DIFFICULTY_CHANGES,
  LEVEL_ALIGNMENTS,
  type DifficultyChange,
  type LevelAlignment,
} from '../utils/gameDefinitions'
import type { DefinitionLevelsFormValue } from '../utils/definitionConfig'

// The per-level progression settings shown in the Layout tab's Levels group:
// how difficulty shifts up the stack, how a smaller upper level is positioned
// over the level below, and whether the stack tapers. Only meaningful for a
// multi-level game, so the editor renders this group only when the level count
// is above 1. Operates on the shared `levels` value shape (the finish rig lives
// on the Objects tab and the per-level toggles in the Advanced tab's Levels
// group; see `LevelSettingsFields`).

interface LevelProgressionFieldsProps {
  value: DefinitionLevelsFormValue
  onChange: (patch: Partial<DefinitionLevelsFormValue>) => void
}

export function LevelProgressionFields({ value, onChange }: LevelProgressionFieldsProps) {
  return (
    <>
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
        Alignment
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
          checked={value.taper}
          onChange={e => onChange({ taper: e.target.checked })}
        />
        <span>Taper</span>
      </label>
    </>
  )
}
