import { titleCaseWire } from '../utils/cellEntityStyles'
import { SKY_TYPES, type SkyType } from '../utils/mazeGameSettings'
import type { DefinitionTopLevelConfig } from '../utils/definitionConfig'

// The final-level scene override (`levels.top`) for a multi-level game: an
// optional override of the top (last-reached) level's sky + perimeter, so the
// climax level can look distinct from the levels below. Lives on the editor's
// Scene tab and only makes sense for a multi-level game, so the editor renders
// it there only when the level count is above 1. `null` means no override (the
// final level looks like the rest); an absent sub-field within an override means
// "inherit the base game's setting", matching the runtime's `TopStartConfig`.

// The perimeter override is a tri-state — the select maps to it, with `inherit`
// leaving the field absent.
const PERIMETER_OPTIONS = [
  { value: 'inherit', label: 'Inherit' },
  { value: 'walled', label: 'Walled' },
  { value: 'open', label: 'Open' },
] as const

interface FinalLevelOverrideFieldsProps {
  value: DefinitionTopLevelConfig | null
  onChange: (top: DefinitionTopLevelConfig | null) => void
}

export function FinalLevelOverrideFields({ value, onChange }: FinalLevelOverrideFieldsProps) {
  const overrideTop = value !== null

  // Build the next `top` from a patch to one of its fields, dropping a key set
  // back to "inherit" so an absent field means inherit (matching the runtime).
  function patchTop(patch: Partial<DefinitionTopLevelConfig>) {
    const next: DefinitionTopLevelConfig = { ...(value ?? {}), ...patch }
    if (next.skyType == null) delete next.skyType
    if (next.perimeterWalls == null) delete next.perimeterWalls
    onChange(next)
  }

  const perimeterValue = value?.perimeterWalls == null ? 'inherit' : value.perimeterWalls ? 'walled' : 'open'

  return (
    <>
      <label className="modal-checkbox">
        <input
          type="checkbox"
          checked={overrideTop}
          // Toggling on seeds an empty override (every field inherits); off
          // clears it back to null so the final level looks like the rest.
          onChange={e => onChange(e.target.checked ? {} : null)}
        />
        <span>Override final level appearance</span>
      </label>

      {overrideTop && (
        <>
          <label className="modal-stacked-input">
            Final Level Sky
            <select
              className="input"
              value={value?.skyType ?? ''}
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
