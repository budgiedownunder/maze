import { useEffect, useRef } from 'react'

interface Props {
  message: string
  onClose: () => void
}

export function GameResultPopup({ message, onClose }: Props) {
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
      <img src="/images/maze/celebrate.gif" alt="Celebration" width={200} height={200} />
      <p style={{ fontSize: 16 }}>{message}</p>
      <button type="button" onClick={onClose}>Close</button>
    </dialog>
  )
}
