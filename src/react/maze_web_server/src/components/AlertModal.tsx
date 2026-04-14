interface Props {
  title: string
  message: string
  onClose: () => void
}

export function AlertModal({ title, message, onClose }: Props) {
  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay">
      <div className="modal modal-sm">
        <h2 className="modal-title">{title}</h2>
        <p className="modal-message">{message}</p>
        <div className="modal-actions-row">
          <button type="button" className="btn-primary" autoFocus onClick={onClose}>OK</button>
        </div>
      </div>
    </div>
  )
}
