import { useEffect, useRef } from 'react'

interface Props {
  message: string
  /**
   * Visual tone. `'success'` shows the celebration GIF; `'fail'` shows the
   * game-over GIF with a muted-red heading.
   */
  tone?: 'success' | 'fail'
  onClose: () => void
  /** When provided, a "Play Again" button restarts the current maze. */
  onPlayAgain?: () => void
}

export function GameResultPopup({ message, tone = 'success', onClose, onPlayAgain }: Props) {
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
      <img
        src={tone === 'fail' ? '/images/maze/game_over.gif' : '/images/maze/celebrate.gif'}
        alt={tone === 'fail' ? 'Game over' : 'Celebration'}
        width={200}
        height={200}
      />
      <p
        style={{
          fontSize: tone === 'fail' ? 24 : 16,
          fontWeight: tone === 'fail' ? 600 : 400,
          color: tone === 'fail' ? '#c0392b' : undefined,
          marginBottom: '1.5rem',
        }}
      >
        {message}
      </p>
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button type="button" onClick={onClose} className="btn-gray" style={{ flex: 1 }}>Close</button>
        {onPlayAgain && (
          <button type="button" onClick={onPlayAgain} className="btn-primary" style={{ flex: 1 }}>Play Again</button>
        )}
      </div>
    </dialog>
  )
}
