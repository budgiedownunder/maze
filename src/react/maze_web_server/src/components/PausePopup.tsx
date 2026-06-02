import { useEffect, useRef } from 'react'

interface Props {
  onResume: () => void
  /** Restarts the current maze from the beginning. */
  onRestart: () => void
}

/**
 * Modal shown while the 2D game is paused. Mirrors {@link GameResultPopup}'s
 * `<dialog>` pattern. Esc / Space are handled by the page's global key handler
 * (which toggles pause), so the dialog's native Esc-cancel is suppressed.
 */
export function PausePopup({ onResume, onRestart }: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null)

  useEffect(() => {
    dialogRef.current?.showModal()
  }, [])

  function handleCancel(e: React.SyntheticEvent) {
    e.preventDefault()
  }

  return (
    <dialog
      ref={dialogRef}
      onCancel={handleCancel}
      style={{ borderRadius: 12, padding: 24, border: 'none', textAlign: 'center', maxWidth: 360 }}
    >
      <p style={{ fontSize: 20, fontWeight: 600, marginBottom: '1.5rem' }}>Paused</p>
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button type="button" onClick={onResume} className="btn-primary" style={{ flex: 1 }}>Resume</button>
        <button type="button" onClick={onRestart} className="btn-gray" style={{ flex: 1 }}>Restart</button>
      </div>
    </dialog>
  )
}
