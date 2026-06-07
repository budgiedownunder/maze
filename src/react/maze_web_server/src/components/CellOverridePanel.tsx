import { useState } from 'react'
import {
  DOOR_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  KEY_HOLDER_STYLES,
  WALL_SOLID_TEXTURES,
  WALL_SPECIAL_TYPES,
  titleCaseWire,
} from '../utils/cellEntityStyles'
import { cellSprite } from '../utils/cellSprite'
import type {
  CellEntity,
  DoorStyle,
  EnemyType,
  FeatureChar,
  HealthStyle,
  KeyHolderStyle,
  WallType,
} from '../types/cellEntities'

const TYPE_LABELS: Record<FeatureChar, string> = {
  E: 'Enemy',
  H: 'Health',
  K: 'Key',
  D: 'Door',
  W: 'Wall',
}

// Whether a wall type is one of the special (non-occluding) types — those select
// directly via the "Type" dropdown; the solid textures sit under "Wall".
function isSpecialWallType(t: string): boolean {
  return (WALL_SPECIAL_TYPES as readonly string[]).includes(t)
}

interface Props {
  /** The selected cell's feature character. */
  cellType: FeatureChar
  /** Selected cell coordinates (zero-based). */
  row: number
  col: number
  /** The cell's current override, used to seed the fields. */
  override: CellEntity | undefined
  /** Apply an override to the cell (called live on each field change). */
  onApply: (entity: CellEntity) => void
  /** Remove the cell's override (all fields back to default). */
  onClear: () => void
  /** When editing the top-left of a multi-cell selection, propagate the (live-applied)
   * override to every selected cell. Omitted for a single cell. */
  onApplyToAll?: () => void
  /** Number of cells in the selection (> 1), shown in the Apply-to-all label. */
  selectionCount?: number
}

// "" means "Default (inherit)" for a rig select; "" / invalid means inherit for a
// numeric field. Parsers reject blanks and out-of-range values back to undefined.
function parseNonNegInt(s: string): number | undefined {
  const t = s.trim()
  if (t === '') return undefined
  const n = Number(t)
  return Number.isInteger(n) && n >= 0 ? n : undefined
}

function parseNonNegFloat(s: string): number | undefined {
  const t = s.trim()
  if (t === '') return undefined
  const n = Number(t)
  return Number.isFinite(n) && n >= 0 ? n : undefined
}

/**
 * Inline inspector for a feature cell's per-cell override. Shown to the right of the
 * grid (reflows below it on narrow screens). Edits apply live: each field change
 * writes the override (or clears it when every field is back to default), so there is
 * no Save button — the maze's normal Save persists it.
 *
 * The parent remounts this with a `key` of the (top-left) cell, so the field state is
 * seeded from that cell's override once and then owned locally during editing. When a
 * rectangular block of same-type cells is selected, the parent passes `onApplyToAll`
 * + `selectionCount`: the panel seeds from / live-applies to the top-left cell and
 * shows an "Apply to all" link that propagates that override to the whole block.
 */
export function CellOverridePanel({
  cellType,
  row,
  col,
  override,
  onApply,
  onClear,
  onApplyToAll,
  selectionCount,
}: Props) {
  const enemy = override?.type === 'E' ? override : undefined
  const health = override?.type === 'H' ? override : undefined
  const key = override?.type === 'K' ? override : undefined
  const door = override?.type === 'D' ? override : undefined
  const wall = override?.type === 'W' ? override : undefined

  const [enemyType, setEnemyType] = useState<string>(enemy?.enemyType ?? '')
  const [damage, setDamage] = useState<string>(enemy?.damage?.toString() ?? '')
  const [movePeriodMs, setMovePeriodMs] = useState<string>(enemy?.movePeriodMs?.toString() ?? '')
  const [healthStyle, setHealthStyle] = useState<string>(health?.healthStyle ?? '')
  const [healAmount, setHealAmount] = useState<string>(health?.healAmount?.toString() ?? '')
  const [keyHolder, setKeyHolder] = useState<string>(key?.keyHolder ?? '')
  const [doorStyle, setDoorStyle] = useState<string>(door?.doorStyle ?? '')
  // Wall is two-tier: `wallKind` is 'wall' (a solid texture, chosen via wallTexture) or
  // one of the special types; `wallTexture` is the solid texture when kind is 'wall'.
  const initialWallType = wall?.wallType
  const [wallKind, setWallKind] = useState<string>(
    initialWallType !== undefined && isSpecialWallType(initialWallType) ? initialWallType : 'wall',
  )
  const [wallTexture, setWallTexture] = useState<string>(
    initialWallType !== undefined && !isSpecialWallType(initialWallType) ? initialWallType : '',
  )

  // Re-seed the fields to defaults when the override is cleared externally (e.g. the
  // toolbar re-stamps the same cell type, which drops the override but keeps the cell
  // selected, so this panel is not remounted). Done as a render-time state adjustment
  // (React's "storing info from previous renders" idiom) — it fires only when the
  // override transitions away, never during the user's own edits (the override stays
  // defined while typing).
  const [seenOverride, setSeenOverride] = useState(override)
  if (seenOverride !== override) {
    setSeenOverride(override)
    if (override === undefined) {
      setEnemyType('')
      setDamage('')
      setMovePeriodMs('')
      setHealthStyle('')
      setHealAmount('')
      setKeyHolder('')
      setDoorStyle('')
      setWallKind('wall')
      setWallTexture('')
    }
  }

  // Emit the built entity, or clear when it carries no override field at all.
  function emit(entity: CellEntity) {
    if (Object.keys(entity).length > 1) onApply(entity)
    else onClear()
  }

  function applyEnemy(et: string, dmg: string, mp: string) {
    const entity: CellEntity = { type: 'E' }
    if (et) entity.enemyType = et as EnemyType
    const d = parseNonNegInt(dmg)
    if (d !== undefined) entity.damage = d
    const m = parseNonNegFloat(mp)
    if (m !== undefined) entity.movePeriodMs = m
    emit(entity)
  }

  function applyHealth(hs: string, amount: string) {
    const entity: CellEntity = { type: 'H' }
    if (hs) entity.healthStyle = hs as HealthStyle
    const a = parseNonNegInt(amount)
    if (a !== undefined) entity.healAmount = a
    emit(entity)
  }

  function applyKey(holder: string) {
    const entity: CellEntity = { type: 'K' }
    if (holder) entity.keyHolder = holder as KeyHolderStyle
    emit(entity)
  }

  function applyDoor(style: string) {
    const entity: CellEntity = { type: 'D' }
    if (style) entity.doorStyle = style as DoorStyle
    emit(entity)
  }

  // The single flat wallType the two-tier UI resolves to: a special kind directly, or
  // the chosen solid texture under "Wall" (or none → default).
  function effectiveWallType(kind: string, texture: string): WallType | undefined {
    if (kind === 'wall') return texture ? (texture as WallType) : undefined
    return kind as WallType
  }

  function applyWall(kind: string, texture: string) {
    const wallType = effectiveWallType(kind, texture)
    if (wallType) onApply({ type: 'W', wallType })
    else onClear() // "Wall" + Default texture = no override (varied default)
  }

  function resetAll() {
    setEnemyType('')
    setDamage('')
    setMovePeriodMs('')
    setHealthStyle('')
    setHealAmount('')
    setKeyHolder('')
    setDoorStyle('')
    setWallKind('wall')
    setWallTexture('')
    onClear()
  }

  return (
    <div className="cell-override-panel" aria-label="Cell overrides">
      <h3 className="cell-override-title">
        {TYPE_LABELS[cellType]} [{row + 1},{col + 1}]
        {onApplyToAll && ` +${(selectionCount ?? 1) - 1} more`}
      </h3>

      {cellType === 'E' && (
        <>
          <label className="cell-override-field">
            <span>Type</span>
            <div className="cell-override-select-row">
              <img
                className="cell-override-preview"
                src={cellSprite('E', { type: 'E', enemyType: enemyType ? (enemyType as EnemyType) : undefined })?.src}
                alt="" aria-hidden="true"
              />
              <select
                className="cell-override-input"
                value={enemyType}
                onChange={e => { setEnemyType(e.target.value); applyEnemy(e.target.value, damage, movePeriodMs) }}
              >
                <option value="">Default</option>
                {ENEMY_TYPES.map(t => <option key={t} value={t}>{titleCaseWire(t)}</option>)}
              </select>
            </div>
          </label>
          <label className="cell-override-field">
            <span>Damage</span>
            <input
              type="number" min="0" className="cell-override-input" placeholder="Default"
              value={damage}
              onChange={e => { setDamage(e.target.value); applyEnemy(enemyType, e.target.value, movePeriodMs) }}
            />
          </label>
          <label className="cell-override-field">
            <span>Move Interval (ms)</span>
            <input
              type="number" min="0" className="cell-override-input" placeholder="Default"
              value={movePeriodMs}
              onChange={e => { setMovePeriodMs(e.target.value); applyEnemy(enemyType, damage, e.target.value) }}
            />
          </label>
        </>
      )}

      {cellType === 'H' && (
        <>
          <label className="cell-override-field">
            <span>Style</span>
            <div className="cell-override-select-row">
              <img
                className="cell-override-preview"
                src={cellSprite('H', { type: 'H', healthStyle: healthStyle ? (healthStyle as HealthStyle) : undefined })?.src}
                alt="" aria-hidden="true"
              />
              <select
                className="cell-override-input"
                value={healthStyle}
                onChange={e => { setHealthStyle(e.target.value); applyHealth(e.target.value, healAmount) }}
              >
                <option value="">Default</option>
                {HEALTH_STYLES.map(s => <option key={s} value={s}>{titleCaseWire(s)}</option>)}
              </select>
            </div>
          </label>
          <label className="cell-override-field">
            <span>Heal Amount</span>
            <input
              type="number" min="0" className="cell-override-input" placeholder="Default"
              value={healAmount}
              onChange={e => { setHealAmount(e.target.value); applyHealth(healthStyle, e.target.value) }}
            />
          </label>
        </>
      )}

      {cellType === 'K' && (
        <label className="cell-override-field">
          <span>Holder</span>
          <select
            className="cell-override-input"
            value={keyHolder}
            onChange={e => { setKeyHolder(e.target.value); applyKey(e.target.value) }}
          >
            <option value="">Default</option>
            {KEY_HOLDER_STYLES.map(s => <option key={s} value={s}>{titleCaseWire(s)}</option>)}
          </select>
        </label>
      )}

      {cellType === 'D' && (
        <label className="cell-override-field">
          <span>Style</span>
          <select
            className="cell-override-input"
            value={doorStyle}
            onChange={e => { setDoorStyle(e.target.value); applyDoor(e.target.value) }}
          >
            <option value="">Default</option>
            {DOOR_STYLES.map(s => <option key={s} value={s}>{titleCaseWire(s)}</option>)}
          </select>
        </label>
      )}

      {cellType === 'W' && (
        <>
          <label className="cell-override-field">
            <span>Type</span>
            <div className="cell-override-select-row">
              <img
                className="cell-override-preview"
                src={cellSprite('W', { type: 'W', wallType: effectiveWallType(wallKind, wallTexture) })?.src}
                alt="" aria-hidden="true"
              />
              <select
                className="cell-override-input"
                value={wallKind}
                onChange={e => { setWallKind(e.target.value); applyWall(e.target.value, wallTexture) }}
              >
                <option value="wall">Wall</option>
                {WALL_SPECIAL_TYPES.map(t => <option key={t} value={t}>{titleCaseWire(t)}</option>)}
              </select>
            </div>
          </label>
          {wallKind === 'wall' && (
            <label className="cell-override-field">
              <span>Texture</span>
              <select
                className="cell-override-input"
                value={wallTexture}
                onChange={e => { setWallTexture(e.target.value); applyWall(wallKind, e.target.value) }}
              >
                <option value="">Default</option>
                {WALL_SOLID_TEXTURES.map(t => <option key={t} value={t}>{titleCaseWire(t)}</option>)}
              </select>
            </label>
          )}
        </>
      )}

      {onApplyToAll && (
        <button type="button" className="cell-override-apply-all" onClick={onApplyToAll}>
          Apply to all {selectionCount} cells
        </button>
      )}
      <button type="button" className="cell-override-reset" onClick={resetAll}>
        Reset to defaults
      </button>
    </div>
  )
}
