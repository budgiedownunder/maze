import { useState } from 'react'
import {
  SKY_TYPES,
  WALL_TYPES,
  loadPlay3dCustomLaunchSettings,
  titleCaseWire,
  type Play3dCustomLaunchSettings,
  type SkyType,
  type WallType,
} from '../utils/play3dCustomLaunchSettings'

interface Props {
  mazeName: string
  onPlay: (settings: Play3dCustomLaunchSettings) => void
  onCancel: () => void
}

export function Play3dCustomLaunchModal({ mazeName, onPlay, onCancel }: Props) {
  const initial = loadPlay3dCustomLaunchSettings()
  const [skyType, setSkyType] = useState<SkyType>(initial.skyType)
  const [wallType, setWallType] = useState<WallType>(initial.wallType)
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
      <div className="modal modal-sm">
        <h2 className="modal-title">Play 3D — {mazeName}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
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

          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary">Play</button>
          </div>
        </form>
      </div>
    </div>
  )
}
