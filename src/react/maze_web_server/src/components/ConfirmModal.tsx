interface Props {
  title: string
  message: string
  confirmLabel: string
  cancelLabel?: string
  isDangerous?: boolean
  error?: string | null
  onConfirm: () => void
  onCancel: () => void
}

export function ConfirmModal({
  title,
  message,
  confirmLabel,
  cancelLabel = 'Cancel',
  isDangerous = false,
  error,
  onConfirm,
  onCancel,
}: Props) {
  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">{title}</h2>
        <p>{message}</p>
        {error && <p role="alert" className="error-msg">{error}</p>}
        <div className="modal-actions-row" style={{ marginTop: '1.5rem' }}>
          <button type="button" onClick={onCancel} className="btn-gray">{cancelLabel}</button>
          <button type="button" onClick={onConfirm} className={isDangerous ? 'btn-danger' : 'btn-primary'}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  )
}
