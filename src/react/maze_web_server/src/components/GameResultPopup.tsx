import { useEffect, useRef } from 'react'

interface Props {
  message: string
  /**
   * Visual tone. `'success'` (default) shows the celebration GIF. `'fail'`
   * skips the celebration and uses a muted-red heading so the popup reads as
   * a loss without a separate "sad" asset.
   */
  tone?: 'success' | 'fail'
  onClose: () => void
}

export function GameResultPopup({ message, tone = 'success', onClose }: Props) {
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
      {tone === 'success' && (
        <img src="/images/maze/celebrate.gif" alt="Celebration" width={200} height={200} />
      )}
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
      <button type="button" onClick={onClose} className="btn-gray" style={{ width: '100%' }}>Close</button>
    </dialog>
  )
}
