import type { DefinitionFormState } from '../utils/definitionConfig'

// The advanced tuning field-group for the game-definition editor: the HUD /
// combat / minimap knobs plus the optional presentation overrides. Every field
// is a top-level string on the form state (raw input text), so the value is a
// pick of those keys and the consumer merges each patch straight into the form.
// Labels describe the in-game role rather than echoing the config key — `title`
// is the splash screen's heading, `mode` the status-bar label — so their effect
// is clear without knowing the wire shape.

export type AdvancedFieldsValue = Pick<
  DefinitionFormState,
  'maxHp' | 'enemyMovePeriodMs' | 'minimapCellPx' | 'minimapRadius' | 'title' | 'mode'
>

// The numeric knobs share the same labelled-number-input shape, so they render
// from a table. `min` is an input hint only — the runtime clamps regardless.
const NUMBER_FIELDS: { key: keyof AdvancedFieldsValue; label: string; min: number }[] = [
  { key: 'maxHp', label: 'Max HP', min: 1 },
  { key: 'enemyMovePeriodMs', label: 'Enemy move period (ms)', min: 1 },
  { key: 'minimapCellPx', label: 'Minimap cell size (px)', min: 1 },
  { key: 'minimapRadius', label: 'Minimap radius (cells)', min: 1 },
]

interface AdvancedFieldsProps {
  value: AdvancedFieldsValue
  onChange: (patch: Partial<AdvancedFieldsValue>) => void
  /** The game's name, shown as the placeholder for the title / mode overrides
   *  since both default to it when left blank. */
  namePlaceholder: string
}

export function AdvancedFields({ value, onChange, namePlaceholder }: AdvancedFieldsProps) {
  const placeholder = namePlaceholder.trim() === '' ? "The game's name" : namePlaceholder.trim()
  return (
    <>
      {NUMBER_FIELDS.map(({ key, label, min }) => (
        <label key={key} className="modal-stacked-input">
          {label}
          <input
            type="number"
            className="input"
            value={value[key]}
            min={min}
            onChange={e => onChange({ [key]: e.target.value } as Partial<AdvancedFieldsValue>)}
          />
        </label>
      ))}

      <label className="modal-stacked-input">
        Splash title
        <input
          type="text"
          className="input"
          value={value.title}
          placeholder={placeholder}
          onChange={e => onChange({ title: e.target.value })}
        />
      </label>

      <label className="modal-stacked-input">
        Status-bar label
        <input
          type="text"
          className="input"
          value={value.mode}
          placeholder={placeholder}
          onChange={e => onChange({ mode: e.target.value })}
        />
      </label>
    </>
  )
}
