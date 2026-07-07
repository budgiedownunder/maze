import {
  DOOR_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  KEY_HOLDER_STYLES,
  titleCaseWire,
} from '../utils/cellEntityStyles'
import type { DoorStyle, EnemyType, HealthStyle, KeyHolderStyle } from '../types/cellEntities'
import { SKY_TYPES, WALL_TYPES, type MazeGameSettings, type SkyType, type WallType } from '../utils/mazeGameSettings'

// Controlled field-groups for the 3D game's presentation settings, extracted
// from the game-settings modal so the same panels back both the maze-launch
// settings dialog and the game-definition editor. Each is a `value` + patching
// `onChange` component that renders only its controls (no panel/tab wrapper), so
// a container (a tab panel or a wizard step) can host it. Field names match
// `MazeGameSettings`, keeping the vocabulary in one place.

export type SceneFieldsValue = Pick<
  MazeGameSettings,
  'skyType' | 'wallType' | 'perimeterWalls' | 'wallTint' | 'wallMaterialVariation'
>
export type ObjectsFieldsValue = Pick<MazeGameSettings, 'doorStyle' | 'keyHolder' | 'enemyType' | 'healthStyle'>
export type DecorFieldsValue = Pick<MazeGameSettings, 'deadEndObjects' | 'wallDecorations' | 'floorAccents'>

interface SceneFieldsProps {
  value: SceneFieldsValue
  onChange: (patch: Partial<SceneFieldsValue>) => void
}

// Sky, wall texture, and the wall-appearance toggles. Quadrant wall types
// (per-quadrant materials) overrides the single default texture + varied tints,
// so those two are disabled while it is on; enclosed skies always wall the
// perimeter, so that toggle is forced on + disabled for them.
export function SceneFields({ value, onChange }: SceneFieldsProps) {
  const skyEnclosed = value.skyType === 'dungeon' || value.skyType === 'chamber'
  return (
    <>
      <label className="modal-stacked-input">
        Sky
        <select
          className="input"
          value={value.skyType}
          onChange={e => onChange({ skyType: e.target.value as SkyType })}
        >
          {SKY_TYPES.map(s => (
            <option key={s} value={s}>{titleCaseWire(s)}</option>
          ))}
        </select>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.wallMaterialVariation}
          onChange={e => onChange({ wallMaterialVariation: e.target.checked })}
        />
        <span>Quadrant wall types</span>
      </label>

      <label className="modal-stacked-input">
        Wall Texture (Default)
        <select
          className="input"
          value={value.wallType}
          disabled={value.wallMaterialVariation}
          onChange={e => onChange({ wallType: e.target.value as WallType })}
        >
          {WALL_TYPES.map(w => (
            <option key={w} value={w}>{titleCaseWire(w)}</option>
          ))}
        </select>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.wallTint}
          disabled={value.wallMaterialVariation}
          onChange={e => onChange({ wallTint: e.target.checked })}
        />
        <span>Varied wall tints</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          // Enclosed skies always wall the perimeter — force the box on and
          // disabled for them.
          checked={skyEnclosed ? true : value.perimeterWalls}
          disabled={skyEnclosed}
          onChange={e => onChange({ perimeterWalls: e.target.checked })}
        />
        <span>Perimeter walls</span>
      </label>
    </>
  )
}

interface ObjectsFieldsProps {
  value: ObjectsFieldsValue
  onChange: (patch: Partial<ObjectsFieldsValue>) => void
}

// The default door / key-holder / enemy / health styles for the game's objects.
export function ObjectsFields({ value, onChange }: ObjectsFieldsProps) {
  return (
    <>
      <label className="modal-stacked-input">
        Door Style (Default)
        <select
          className="input"
          value={value.doorStyle}
          onChange={e => onChange({ doorStyle: e.target.value as DoorStyle })}
        >
          {DOOR_STYLES.map(d => (
            <option key={d} value={d}>{titleCaseWire(d)}</option>
          ))}
        </select>
      </label>

      <label className="modal-stacked-input">
        Key Holder (Default)
        <select
          className="input"
          value={value.keyHolder}
          onChange={e => onChange({ keyHolder: e.target.value as KeyHolderStyle })}
        >
          {KEY_HOLDER_STYLES.map(k => (
            <option key={k} value={k}>{titleCaseWire(k)}</option>
          ))}
        </select>
      </label>

      <label className="modal-stacked-input">
        Enemy Type (Default)
        <select
          className="input"
          value={value.enemyType}
          onChange={e => onChange({ enemyType: e.target.value as EnemyType })}
        >
          {ENEMY_TYPES.map(et => (
            <option key={et} value={et}>{titleCaseWire(et)}</option>
          ))}
        </select>
      </label>

      <label className="modal-stacked-input">
        Health Style (Default)
        <select
          className="input"
          value={value.healthStyle}
          onChange={e => onChange({ healthStyle: e.target.value as HealthStyle })}
        >
          {HEALTH_STYLES.map(hs => (
            <option key={hs} value={hs}>{titleCaseWire(hs)}</option>
          ))}
        </select>
      </label>
    </>
  )
}

interface DecorFieldsProps {
  value: DecorFieldsValue
  onChange: (patch: Partial<DecorFieldsValue>) => void
}

// The optional decorative landmarks scattered through the maze.
export function DecorFields({ value, onChange }: DecorFieldsProps) {
  return (
    <>
      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.deadEndObjects}
          onChange={e => onChange({ deadEndObjects: e.target.checked })}
        />
        <span>Dead-end objects</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.wallDecorations}
          onChange={e => onChange({ wallDecorations: e.target.checked })}
        />
        <span>Sparse wall decorations</span>
      </label>

      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={value.floorAccents}
          onChange={e => onChange({ floorAccents: e.target.checked })}
        />
        <span>Floor junction markers</span>
      </label>
    </>
  )
}
