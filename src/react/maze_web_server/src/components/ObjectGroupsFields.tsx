import {
  DOOR_STYLES,
  KEY_HOLDER_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  titleCaseWire,
} from '../utils/cellEntityStyles'
import type { DoorStyle, EnemyType, HealthStyle, KeyHolderStyle } from '../types/cellEntities'
import {
  MAX_DOOR_COUNT, MAX_ENEMY_COUNT, MAX_HEALTH_COUNT, MAX_TREASURE_COUNT,
  MAX_SPARE_DOOR_COUNT, MAX_SPARE_KEY_COUNT,
} from '../utils/validation'
import type { MazeGenerationFieldsValue } from './MazeGenerationFields'
import type { ObjectsFieldsValue } from './GameSettingsFields'

// The Objects tab of the game-definition editor: each kind of object configured
// together as a labelled group — how many (the count, from the generation slice)
// next to how it looks (the style, from the objects slice). Counts and styles
// live in two different form slices, so this composes both rather than extending
// the shared `SceneFields`/`ObjectsFields` groups (which also back the
// single-maze settings modal and have no counts).

interface ObjectGroupsFieldsProps {
  // The feature counts (doors/spare/enemies/health/treasure) — the generation slice.
  counts: MazeGenerationFieldsValue
  onCountsChange: (patch: Partial<MazeGenerationFieldsValue>) => void
  // The default object styles — the objects slice.
  styles: ObjectsFieldsValue
  onStylesChange: (patch: Partial<ObjectsFieldsValue>) => void
}

export function ObjectGroupsFields({ counts, onCountsChange, styles, onStylesChange }: ObjectGroupsFieldsProps) {
  // A count field (Count / Spares); `max` is an input hint, the generator clamps.
  const countField = (key: keyof MazeGenerationFieldsValue, label: string, max: number) => (
    <label className="modal-stacked-input">
      {label}
      <input
        type="number"
        className="input"
        value={counts[key]}
        min={0}
        max={max}
        onChange={e => onCountsChange({ [key]: e.target.value } as Partial<MazeGenerationFieldsValue>)}
      />
    </label>
  )

  return (
    <>
      <div className="object-group" role="group" aria-labelledby="gamedef-objgrp-doors">
        <h4 id="gamedef-objgrp-doors" className="object-group-title">Doors</h4>
        {countField('doorCount', 'Count', MAX_DOOR_COUNT)}
        {countField('spareDoors', 'Spares', MAX_SPARE_DOOR_COUNT)}
        <label className="modal-stacked-input">
          Style
          <select
            className="input"
            value={styles.doorStyle}
            onChange={e => onStylesChange({ doorStyle: e.target.value as DoorStyle })}
          >
            {DOOR_STYLES.map(d => (
              <option key={d} value={d}>{titleCaseWire(d)}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="object-group" role="group" aria-labelledby="gamedef-objgrp-keys">
        <h4 id="gamedef-objgrp-keys" className="object-group-title">Keys</h4>
        {countField('spareKeys', 'Spares', MAX_SPARE_KEY_COUNT)}
        <label className="modal-stacked-input">
          Holder
          <select
            className="input"
            value={styles.keyHolder}
            onChange={e => onStylesChange({ keyHolder: e.target.value as KeyHolderStyle })}
          >
            {KEY_HOLDER_STYLES.map(k => (
              <option key={k} value={k}>{titleCaseWire(k)}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="object-group" role="group" aria-labelledby="gamedef-objgrp-enemies">
        <h4 id="gamedef-objgrp-enemies" className="object-group-title">Enemies</h4>
        {countField('enemyCount', 'Count', MAX_ENEMY_COUNT)}
        <label className="modal-stacked-input">
          Type
          <select
            className="input"
            value={styles.enemyType}
            onChange={e => onStylesChange({ enemyType: e.target.value as EnemyType })}
          >
            {ENEMY_TYPES.map(et => (
              <option key={et} value={et}>{titleCaseWire(et)}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="object-group" role="group" aria-labelledby="gamedef-objgrp-health">
        <h4 id="gamedef-objgrp-health" className="object-group-title">Health</h4>
        {countField('healthCount', 'Count', MAX_HEALTH_COUNT)}
        <label className="modal-stacked-input">
          Type
          <select
            className="input"
            value={styles.healthStyle}
            onChange={e => onStylesChange({ healthStyle: e.target.value as HealthStyle })}
          >
            {HEALTH_STYLES.map(hs => (
              <option key={hs} value={hs}>{titleCaseWire(hs)}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="object-group" role="group" aria-labelledby="gamedef-objgrp-treasure">
        <h4 id="gamedef-objgrp-treasure" className="object-group-title">Treasure</h4>
        {countField('treasureCount', 'Count', MAX_TREASURE_COUNT)}
      </div>
    </>
  )
}
