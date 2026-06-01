import { useState } from 'react'
import {
  DOOR_STYLES,
  ENEMY_TYPES,
  HEALTH_STYLES,
  KEY_HOLDER_STYLES,
  SKY_TYPES,
  WALL_TYPES,
  loadPlay3dCustomLaunchSettings,
  titleCaseWire,
  type DoorStyle,
  type EnemyType,
  type HealthStyle,
  type KeyHolderStyle,
  type Play3dCustomLaunchSettings,
  type SkyType,
  type WallType,
} from '../utils/play3dCustomLaunchSettings'

interface Props {
  mazeName: string
  onPlay: (settings: Play3dCustomLaunchSettings) => void
  onCancel: () => void
}

// Tab identifiers for the launch settings, grouping the style/scene fields so
// the dialog reads as a few short panels rather than one long scrolling list.
// The time limit, validation error and action buttons stay pinned below the
// panels so a timer error is never hidden on an inactive tab.
const TABS = ['scene', 'objects', 'decor'] as const
type LaunchTab = (typeof TABS)[number]
const TAB_LABELS: Record<LaunchTab, string> = {
  scene: 'Scene',
  objects: 'Objects',
  decor: 'Decor',
}

export function Play3dCustomLaunchModal({ mazeName, onPlay, onCancel }: Props) {
  const initial = loadPlay3dCustomLaunchSettings()
  const [activeTab, setActiveTab] = useState<LaunchTab>('scene')
  const [skyType, setSkyType] = useState<SkyType>(initial.skyType)
  const [wallType, setWallType] = useState<WallType>(initial.wallType)
  const [doorStyle, setDoorStyle] = useState<DoorStyle>(initial.doorStyle)
  const [keyHolder, setKeyHolder] = useState<KeyHolderStyle>(initial.keyHolder)
  const [enemyType, setEnemyType] = useState<EnemyType>(initial.enemyType)
  const [healthStyle, setHealthStyle] = useState<HealthStyle>(initial.healthStyle)
  const [wallTint, setWallTint] = useState(initial.wallTint)
  const [wallMaterialVariation, setWallMaterialVariation] = useState(initial.wallMaterialVariation)
  const [deadEndObjects, setDeadEndObjects] = useState(initial.deadEndObjects)
  const [wallDecorations, setWallDecorations] = useState(initial.wallDecorations)
  const [floorAccents, setFloorAccents] = useState(initial.floorAccents)
  const [timerSeconds, setTimerSeconds] = useState<string>(String(initial.timerSeconds))
  const [validationError, setValidationError] = useState<string | null>(null)

  function clearError() {
    if (validationError !== null) setValidationError(null)
  }

  // Arrow-key navigation across the tab strip, matching the WAI-ARIA tabs
  // pattern (Left/Right move between tabs, wrapping at the ends).
  function handleTabKeyDown(e: React.KeyboardEvent, index: number) {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return
    e.preventDefault()
    const delta = e.key === 'ArrowRight' ? 1 : -1
    const next = (index + delta + TABS.length) % TABS.length
    setActiveTab(TABS[next])
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const secs = Number(timerSeconds)
    if (!Number.isFinite(secs) || secs <= 0) {
      setValidationError('Time limit must be a positive number of seconds.')
      return
    }
    setValidationError(null)
    onPlay({
      skyType,
      wallType,
      doorStyle,
      keyHolder,
      enemyType,
      healthStyle,
      wallTint,
      wallMaterialVariation,
      deadEndObjects,
      wallDecorations,
      floorAccents,
      timerSeconds: secs,
    })
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Play 3D — customise launch"
      className="modal-overlay"
      style={{ zIndex: 1200 }}
    >
      <div className="modal modal-sm modal-with-scroll-body">
        <h2 className="modal-title">Play 3D — {mazeName}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <div className="modal-tabs" role="tablist" aria-label="Launch settings">
            {TABS.map((tab, index) => (
              <button
                key={tab}
                type="button"
                role="tab"
                id={`launch-tab-${tab}`}
                aria-selected={activeTab === tab}
                aria-controls={`launch-panel-${tab}`}
                tabIndex={activeTab === tab ? 0 : -1}
                className="modal-tab"
                onClick={() => setActiveTab(tab)}
                onKeyDown={e => handleTabKeyDown(e, index)}
              >
                {TAB_LABELS[tab]}
              </button>
            ))}
          </div>

          {/* Scrollable middle region: only the active tab's controls scroll
              when the viewport is too short; the title, the pinned time-limit
              row and the action buttons stay outside this box. */}
          <div className="modal-scroll-body">
            <div
              role="tabpanel"
              id="launch-panel-scene"
              aria-labelledby="launch-tab-scene"
              hidden={activeTab !== 'scene'}
            >
              <label className="modal-stacked-input">
                Sky
                <select
                  className="input"
                  value={skyType}
                  onChange={e => { setSkyType(e.target.value as SkyType); clearError() }}
                >
                  {SKY_TYPES.map(s => (
                    <option key={s} value={s}>{titleCaseWire(s)}</option>
                  ))}
                </select>
              </label>

              <label className="modal-checkbox">
                <input
                  type="checkbox"
                  checked={wallMaterialVariation}
                  onChange={e => { setWallMaterialVariation(e.target.checked); clearError() }}
                />
                <span>Quadrant wall types</span>
              </label>

              <label className="modal-stacked-input">
                Wall texture
                <select
                  className="input"
                  value={wallType}
                  disabled={wallMaterialVariation}
                  onChange={e => { setWallType(e.target.value as WallType); clearError() }}
                >
                  {WALL_TYPES.map(w => (
                    <option key={w} value={w}>{titleCaseWire(w)}</option>
                  ))}
                </select>
              </label>

              <label className="modal-checkbox">
                <input
                  type="checkbox"
                  checked={wallTint}
                  disabled={wallMaterialVariation}
                  onChange={e => { setWallTint(e.target.checked); clearError() }}
                />
                <span>Varied wall tints</span>
              </label>
            </div>

            <div
              role="tabpanel"
              id="launch-panel-objects"
              aria-labelledby="launch-tab-objects"
              hidden={activeTab !== 'objects'}
            >
              <label className="modal-stacked-input">
                Door style
                <select
                  className="input"
                  value={doorStyle}
                  onChange={e => { setDoorStyle(e.target.value as DoorStyle); clearError() }}
                >
                  {DOOR_STYLES.map(d => (
                    <option key={d} value={d}>{titleCaseWire(d)}</option>
                  ))}
                </select>
              </label>

              <label className="modal-stacked-input">
                Key holder
                <select
                  className="input"
                  value={keyHolder}
                  onChange={e => { setKeyHolder(e.target.value as KeyHolderStyle); clearError() }}
                >
                  {KEY_HOLDER_STYLES.map(k => (
                    <option key={k} value={k}>{titleCaseWire(k)}</option>
                  ))}
                </select>
              </label>

              <label className="modal-stacked-input">
                Enemy type
                <select
                  className="input"
                  value={enemyType}
                  onChange={e => { setEnemyType(e.target.value as EnemyType); clearError() }}
                >
                  {ENEMY_TYPES.map(et => (
                    <option key={et} value={et}>{titleCaseWire(et)}</option>
                  ))}
                </select>
              </label>

              <label className="modal-stacked-input">
                Health style
                <select
                  className="input"
                  value={healthStyle}
                  onChange={e => { setHealthStyle(e.target.value as HealthStyle); clearError() }}
                >
                  {HEALTH_STYLES.map(hs => (
                    <option key={hs} value={hs}>{titleCaseWire(hs)}</option>
                  ))}
                </select>
              </label>
            </div>

            <div
              role="tabpanel"
              id="launch-panel-decor"
              aria-labelledby="launch-tab-decor"
              hidden={activeTab !== 'decor'}
            >
              <label className="modal-checkbox">
                <input
                  type="checkbox"
                  checked={deadEndObjects}
                  onChange={e => { setDeadEndObjects(e.target.checked); clearError() }}
                />
                <span>Dead-end objects</span>
              </label>

              <label className="modal-checkbox">
                <input
                  type="checkbox"
                  checked={wallDecorations}
                  onChange={e => { setWallDecorations(e.target.checked); clearError() }}
                />
                <span>Sparse wall decorations</span>
              </label>

              <label className="modal-checkbox">
                <input
                  type="checkbox"
                  checked={floorAccents}
                  onChange={e => { setFloorAccents(e.target.checked); clearError() }}
                />
                <span>Floor junction markers</span>
              </label>
            </div>
          </div>

          {/* Pinned below the tab panels: time limit, validation error and the
              action buttons stay visible regardless of the active tab. */}
          <div className="modal-tab-footer">
            <label className="modal-stacked-input">
              Time limit (seconds)
              <input
                type="number"
                className="input"
                value={timerSeconds}
                // No `min` attribute on purpose — we want our own inline
                // error message (handled in handleSubmit), not the
                // browser's validation tooltip, when the user submits a
                // non-positive value.
                onChange={e => { setTimerSeconds(e.target.value); clearError() }}
              />
            </label>

            {validationError && (
              <p role="alert" className="error-msg">{validationError}</p>
            )}
          </div>

          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary">Play</button>
          </div>
        </form>
      </div>
    </div>
  )
}
