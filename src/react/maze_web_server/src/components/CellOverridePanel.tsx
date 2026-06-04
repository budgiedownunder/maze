import { useState } from 'react'
import {
  DOOR_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  KEY_HOLDER_STYLES,
  titleCaseWire,
} from '../utils/cellEntityStyles'
import type {
  CellEntity,
  DoorStyle,
  EnemyType,
  FeatureChar,
  HealthStyle,
  KeyHolderStyle,
} from '../types/cellEntities'

const TYPE_LABELS: Record<FeatureChar, string> = {
  E: 'Enemy',
  H: 'Health',
  K: 'Key',
  D: 'Door',
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
 * Inline inspector for a single feature cell's per-cell override. Shown to the right
 * of the grid (reflows below it on narrow screens). Edits apply live: each field
 * change writes the override (or clears it when every field is back to default), so
 * there is no Save button — the maze's normal Save persists it.
 *
 * The parent remounts this with a `key` of the active cell, so the field state is
 * seeded from that cell's override once and then owned locally during editing.
 */
export function CellOverridePanel({ cellType, row, col, override, onApply, onClear }: Props) {
  const enemy = override?.type === 'E' ? override : undefined
  const health = override?.type === 'H' ? override : undefined
  const key = override?.type === 'K' ? override : undefined
  const door = override?.type === 'D' ? override : undefined

  const [enemyType, setEnemyType] = useState<string>(enemy?.enemyType ?? '')
  const [damage, setDamage] = useState<string>(enemy?.damage?.toString() ?? '')
  const [movePeriodMs, setMovePeriodMs] = useState<string>(enemy?.movePeriodMs?.toString() ?? '')
  const [healthStyle, setHealthStyle] = useState<string>(health?.healthStyle ?? '')
  const [healAmount, setHealAmount] = useState<string>(health?.healAmount?.toString() ?? '')
  const [keyHolder, setKeyHolder] = useState<string>(key?.keyHolder ?? '')
  const [doorStyle, setDoorStyle] = useState<string>(door?.doorStyle ?? '')

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

  function resetAll() {
    setEnemyType('')
    setDamage('')
    setMovePeriodMs('')
    setHealthStyle('')
    setHealAmount('')
    setKeyHolder('')
    setDoorStyle('')
    onClear()
  }

  return (
    <div className="cell-override-panel" aria-label="Cell overrides">
      <h3 className="cell-override-title">
        {TYPE_LABELS[cellType]} [{row + 1},{col + 1}]
      </h3>

      {cellType === 'E' && (
        <>
          <label className="cell-override-field">
            <span>Type</span>
            <select
              className="cell-override-input"
              value={enemyType}
              onChange={e => { setEnemyType(e.target.value); applyEnemy(e.target.value, damage, movePeriodMs) }}
            >
              <option value="">Default</option>
              {ENEMY_TYPES.map(t => <option key={t} value={t}>{titleCaseWire(t)}</option>)}
            </select>
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
            <select
              className="cell-override-input"
              value={healthStyle}
              onChange={e => { setHealthStyle(e.target.value); applyHealth(e.target.value, healAmount) }}
            >
              <option value="">Default</option>
              {HEALTH_STYLES.map(s => <option key={s} value={s}>{titleCaseWire(s)}</option>)}
            </select>
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

      <button type="button" className="cell-override-reset" onClick={resetAll}>
        Reset to defaults
      </button>
    </div>
  )
}
