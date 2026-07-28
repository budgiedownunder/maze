import { useState } from 'react'
import { MAZE_GAME_SETTINGS_DEFAULTS, type MazeGameSettings } from '../utils/mazeGameSettings'
import { ModalTabStrip } from './ModalTabs'
import { modalTabPanelProps, type ModalTab } from '../utils/modalTabs'
import {
  SceneFields,
  ObjectsFields,
  DecorFields,
  type SceneFieldsValue,
  type ObjectsFieldsValue,
  type DecorFieldsValue,
} from './GameSettingsFields'

interface Props {
  mazeName: string
  // Seed values for the form — the maze's saved settings, passed by both the
  // Settings editor and the launch chooser's "Custom Run". Falls back to the
  // built-in defaults when omitted.
  initialSettings?: MazeGameSettings
  // Dialog heading and submit-button label, so the same modal serves both the
  // per-maze settings editor ("Apply" — the maze's own Save persists it) and
  // the one-off Custom Run launch ("Play", the default).
  title?: string
  submitLabel?: string
  onSubmit: (settings: MazeGameSettings) => void
  onCancel: () => void
}

// Tab identifiers for the game settings, grouping the style/scene fields so
// the dialog reads as a few short panels rather than one long scrolling list.
// The time limit, validation error and action buttons stay pinned below the
// panels so a timer error is never hidden on an inactive tab.
const TABS = [
  { id: 'scene', label: 'Scene' },
  { id: 'objects', label: 'Objects' },
  { id: 'decor', label: 'Decor' },
] as const satisfies readonly ModalTab[]
type LaunchTab = (typeof TABS)[number]['id']

export function MazeGameSettingsModal({ mazeName, initialSettings, title, submitLabel, onSubmit, onCancel }: Props) {
  const initial = initialSettings ?? MAZE_GAME_SETTINGS_DEFAULTS
  const dialogTitle = title ?? `Play 3D — ${mazeName}`
  const [activeTab, setActiveTab] = useState<LaunchTab>('scene')
  const [scene, setScene] = useState<SceneFieldsValue>({
    skyType: initial.skyType,
    wallType: initial.wallType,
    perimeterWalls: initial.perimeterWalls,
    wallTint: initial.wallTint,
    wallMaterialVariation: initial.wallMaterialVariation,
  })
  const [objects, setObjects] = useState<ObjectsFieldsValue>({
    doorStyle: initial.doorStyle,
    keyHolder: initial.keyHolder,
    enemyType: initial.enemyType,
    healthStyle: initial.healthStyle,
  })
  const [decor, setDecor] = useState<DecorFieldsValue>({
    deadEndObjects: initial.deadEndObjects,
    wallDecorations: initial.wallDecorations,
    floorAccents: initial.floorAccents,
  })
  const [timerSeconds, setTimerSeconds] = useState<string>(String(initial.timerSeconds))
  const [validationError, setValidationError] = useState<string | null>(null)

  function clearError() {
    if (validationError !== null) setValidationError(null)
  }

  // Each field-group patches its slice and clears any pending timer error, so a
  // stale validation message doesn't linger while the user changes a setting.
  const patchScene = (patch: Partial<SceneFieldsValue>) => { setScene(s => ({ ...s, ...patch })); clearError() }
  const patchObjects = (patch: Partial<ObjectsFieldsValue>) => { setObjects(o => ({ ...o, ...patch })); clearError() }
  const patchDecor = (patch: Partial<DecorFieldsValue>) => { setDecor(d => ({ ...d, ...patch })); clearError() }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const secs = Number(timerSeconds)
    if (!Number.isFinite(secs) || secs <= 0) {
      setValidationError('Time limit must be a positive number of seconds.')
      return
    }
    setValidationError(null)
    onSubmit({ ...scene, ...objects, ...decor, timerSeconds: secs })
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title ?? 'Play 3D — customise launch'}
      className="modal-overlay"
      style={{ zIndex: 1200 }}
    >
      <div className="modal modal-sm modal-with-scroll-body">
        <h2 className="modal-title">{dialogTitle}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <ModalTabStrip
            tabs={TABS}
            activeTab={activeTab}
            onSelect={setActiveTab}
            idPrefix="launch"
            ariaLabel="Launch settings"
          />

          {/* Scrollable middle region: only the active tab's controls scroll
              when the viewport is too short; the title, the pinned time-limit
              row and the action buttons stay outside this box. */}
          <div className="modal-scroll-body">
            <div {...modalTabPanelProps('launch', 'scene', activeTab)}>
              <SceneFields value={scene} onChange={patchScene} />
            </div>

            <div {...modalTabPanelProps('launch', 'objects', activeTab)}>
              <ObjectsFields value={objects} onChange={patchObjects} />
            </div>

            <div {...modalTabPanelProps('launch', 'decor', activeTab)}>
              <DecorFields value={decor} onChange={patchDecor} />
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
            <button type="submit" className="btn-primary">{submitLabel ?? 'Play'}</button>
          </div>
        </form>
      </div>
    </div>
  )
}
