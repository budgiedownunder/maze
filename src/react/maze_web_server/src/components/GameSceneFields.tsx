import { SKY_TYPES, WALL_TYPES, type SkyType, type WallType } from '../utils/mazeGameSettings'
import { titleCaseWire } from '../utils/cellEntityStyles'
import { FieldGroup } from './FieldGroup'
import type { SceneFieldsValue, DecorFieldsValue } from './GameSettingsFields'
import type { DefinitionTopLevelConfig } from '../utils/definitionConfig'

// The Scene tab of the game-definition editor: the sky / walls / decoration
// controls grouped into cards. It composes the base scene settings (form.scene),
// the decorative-landmark toggles (form.decor, folded in here rather than a
// separate Decor tab) and — for a multi-level game — the final-level scene
// override (form.levels.top). The override needs no explicit toggle: for a
// multi-level game the two "Final Level" selects (sky in the Sky group,
// perimeter in the Walls group) are shown outright, each defaulting to "Inherit"
// so leaving them alone keeps the final level looking like the rest. The labels
// are shorter than the shared `SceneFields`/`DecorFields` (which back the single-
// maze settings modal), so this composes its own controls rather than reusing
// those groups.
//
// Two interlocks mirror the maze settings modal: quadrant wall types (per-
// quadrant materials) supersedes the single default texture + varied tints, so
// those are disabled while it is on; and enclosed skies always wall the
// perimeter, so that toggle is forced on + disabled for them.

// The final-level perimeter override is a tri-state; the select maps to it with
// `inherit` leaving the field absent (the final level inherits the base value).
const PERIMETER_OPTIONS = [
  { value: 'inherit', label: 'Inherit' },
  { value: 'walled', label: 'Walled' },
  { value: 'open', label: 'Open' },
] as const

interface GameSceneFieldsProps {
  scene: SceneFieldsValue
  onSceneChange: (patch: Partial<SceneFieldsValue>) => void
  decor: DecorFieldsValue
  onDecorChange: (patch: Partial<DecorFieldsValue>) => void
  /** The final-level override (form.levels.top) — shown only when multi-level. */
  top: DefinitionTopLevelConfig | null
  onTopChange: (top: DefinitionTopLevelConfig | null) => void
  multiLevel: boolean
}

export function GameSceneFields({
  scene, onSceneChange, decor, onDecorChange, top, onTopChange, multiLevel,
}: GameSceneFieldsProps) {
  const skyEnclosed = scene.skyType === 'dungeon' || scene.skyType === 'chamber'

  // Build the next `top` from a patch to one of its fields, dropping a key set
  // back to "inherit" so an absent field means inherit (matching the runtime).
  function patchTop(patch: Partial<DefinitionTopLevelConfig>) {
    const next: DefinitionTopLevelConfig = { ...(top ?? {}), ...patch }
    if (next.skyType == null) delete next.skyType
    if (next.perimeterWalls == null) delete next.perimeterWalls
    onTopChange(next)
  }
  const perimeterValue = top?.perimeterWalls == null ? 'inherit' : top.perimeterWalls ? 'walled' : 'open'

  return (
    <>
      <FieldGroup title="Sky" id="sky">
        {/* No visible label — the group heading already names it; the aria-label
            keeps the control accessible. */}
        <select
          className="input"
          aria-label="Sky"
          value={scene.skyType}
          onChange={e => onSceneChange({ skyType: e.target.value as SkyType })}
        >
          {SKY_TYPES.map(s => (
            <option key={s} value={s}>{titleCaseWire(s)}</option>
          ))}
        </select>

        {/* For a multi-level game the final level's sky can override the base;
            "Inherit" (the default) leaves it matching the rest. */}
        {multiLevel && (
          <label className="modal-stacked-input">
            {/* Visible "Final Level"; the hidden word names the overridden
                attribute so screen readers can tell this apart from the Walls
                group's "Final Level" without repeating the group name. */}
            Final Level<span className="visually-hidden"> Sky</span>
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
        )}
      </FieldGroup>

      <FieldGroup title="Walls" id="walls">
        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={scene.wallMaterialVariation}
            onChange={e => onSceneChange({ wallMaterialVariation: e.target.checked })}
          />
          <span>Quadrant wall types</span>
        </label>

        <label className="modal-stacked-input">
          Texture
          <select
            className="input"
            value={scene.wallType}
            disabled={scene.wallMaterialVariation}
            onChange={e => onSceneChange({ wallType: e.target.value as WallType })}
          >
            {WALL_TYPES.map(w => (
              <option key={w} value={w}>{titleCaseWire(w)}</option>
            ))}
          </select>
        </label>

        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={skyEnclosed ? true : scene.perimeterWalls}
            disabled={skyEnclosed}
            onChange={e => onSceneChange({ perimeterWalls: e.target.checked })}
          />
          <span>Perimeter</span>
        </label>

        {/* The final level's perimeter can override the base, beside it here;
            "Inherit" (the default) leaves it matching the rest. */}
        {multiLevel && (
          <label className="modal-stacked-input">
            {/* Visible "Final Level"; the hidden word names the overridden
                attribute (not the group) so it reads as "Final Level Perimeter"
                to screen readers, distinct from the Sky group's "Final Level". */}
            Final Level<span className="visually-hidden"> Perimeter</span>
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
        )}
      </FieldGroup>

      <FieldGroup title="Decoration" id="decoration">
        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={scene.wallTint}
            disabled={scene.wallMaterialVariation}
            onChange={e => onSceneChange({ wallTint: e.target.checked })}
          />
          <span>Wall tints</span>
        </label>

        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={decor.wallDecorations}
            onChange={e => onDecorChange({ wallDecorations: e.target.checked })}
          />
          <span>Wall objects</span>
        </label>

        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={decor.deadEndObjects}
            onChange={e => onDecorChange({ deadEndObjects: e.target.checked })}
          />
          <span>Dead-end objects</span>
        </label>

        <label className="modal-checkbox">
          <input
            type="checkbox"
            checked={decor.floorAccents}
            onChange={e => onDecorChange({ floorAccents: e.target.checked })}
          />
          <span>Floor junctions</span>
        </label>
      </FieldGroup>
    </>
  )
}
