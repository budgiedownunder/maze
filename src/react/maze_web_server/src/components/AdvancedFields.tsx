import type { ReactNode } from 'react'
import { FieldGroup } from './FieldGroup'
import type { DefinitionFormState } from '../utils/definitionConfig'

// The advanced tuning field-groups for the game-definition editor: the combat /
// minimap knobs and the presentation overrides, grouped into cards. Every field
// is a top-level string on the form state (raw input text), so the value is a
// pick of those keys and the consumer merges each patch straight into the form.
// Labels describe the in-game role rather than echoing the config key — `title`
// is the splash screen's heading, `mode` the status-bar label — so their effect
// is clear without knowing the wire shape.

export type AdvancedFieldsValue = Pick<
  DefinitionFormState,
  'maxHp' | 'startingHp' | 'enemyMovePeriodMs' | 'minimapCellPx' | 'minimapRadius' | 'title' | 'mode'
>

interface AdvancedFieldsProps {
  value: AdvancedFieldsValue
  onChange: (patch: Partial<AdvancedFieldsValue>) => void
  /** The game's name, shown as the placeholder for the title / mode overrides
   *  since both default to it when left blank. */
  namePlaceholder: string
  /** Optional group slotted between Health & Enemies and Minimap — the editor
   *  passes the multi-level Levels group here (it owns the levels slice). */
  levelsGroup?: ReactNode
}

export function AdvancedFields({ value, onChange, namePlaceholder, levelsGroup }: AdvancedFieldsProps) {
  const placeholder = namePlaceholder.trim() === '' ? "The game's name" : namePlaceholder.trim()

  // A labelled number input; `min` is an input hint only — the runtime clamps.
  const numberField = (key: keyof AdvancedFieldsValue, label: string, min: number) => (
    <label className="modal-stacked-input">
      {label}
      <input
        type="number"
        className="input"
        value={value[key]}
        min={min}
        onChange={e => onChange({ [key]: e.target.value } as Partial<AdvancedFieldsValue>)}
      />
    </label>
  )

  // A text override that defaults from the game's name when left blank.
  const textField = (key: 'title' | 'mode', label: string) => (
    <label className="modal-stacked-input">
      {label}
      <input
        type="text"
        className="input"
        value={value[key]}
        placeholder={placeholder}
        onChange={e => onChange({ [key]: e.target.value })}
      />
    </label>
  )

  return (
    <>
      <FieldGroup title="Health & Enemies" id="health-enemies">
        {/* Starting HP leads: you choose how much health the player has, then
            the cap it can be healed back to. Blank is a real choice rather than
            an empty field — it starts the player at full health — so the
            placeholder says so, a bare empty number box reading as unset. */}
        <label className="modal-stacked-input">
          Starting HP
          <input
            type="number"
            className="input"
            value={value.startingHp}
            min={1}
            max={value.maxHp === '' ? undefined : Number(value.maxHp)}
            placeholder="Full health"
            onChange={e => onChange({ startingHp: e.target.value })}
          />
        </label>
        {numberField('maxHp', 'Max HP', 1)}
        {numberField('enemyMovePeriodMs', 'Enemy move period (ms)', 1)}
      </FieldGroup>

      {levelsGroup}

      <FieldGroup title="Minimap" id="minimap">
        {numberField('minimapCellPx', 'Cell size (px)', 1)}
        {numberField('minimapRadius', 'Radius (cells)', 1)}
      </FieldGroup>

      <FieldGroup title="Titles" id="titles">
        {textField('title', 'Splash')}
        {textField('mode', 'Status-bar')}
      </FieldGroup>
    </>
  )
}
