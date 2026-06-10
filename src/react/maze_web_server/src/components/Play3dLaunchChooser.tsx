interface Props {
  mazeName: string
  onRun: () => void
  onCustomRun: () => void
  onCancel: () => void
}

// Shown when Play 3D is chosen: Run launches with the maze's saved settings,
// Custom Run opens the settings modal for a one-off (non-persisted) launch, and
// Cancel dismisses. Buttons are stacked vertically as full-width tap targets so
// they're comfortable to hit on touch.
export function Play3dLaunchChooser({ mazeName, onRun, onCustomRun, onCancel }: Props) {
  const title = `Play 3D — ${mazeName}`
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={title}
      className="modal-overlay"
      style={{ zIndex: 1200 }}
    >
      <div className="modal modal-sm">
        <h2 className="modal-title">{title}</h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1.5rem' }}>
          <button type="button" className="btn-primary" onClick={onRun}>Run</button>
          <button type="button" className="btn-gray" onClick={onCustomRun}>Custom Run…</button>
          <button type="button" className="btn-gray" onClick={onCancel}>Cancel</button>
        </div>
      </div>
    </div>
  )
}
