interface Props {
  onConfirm: () => void
  onCancel: () => void
}

export function DeleteAccountModal({ onConfirm, onCancel }: Props) {
  return (
    <div role="dialog" aria-modal="true" aria-label="Confirm Account Deletion" className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">Delete Account</h2>
        <p>Are you sure you want to permanently delete your account? This will also delete all your mazes and cannot be undone.</p>
        <div className="modal-actions-row" style={{ marginTop: '1.5rem' }}>
          <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
          <button type="button" onClick={onConfirm} className="btn-danger">Delete</button>
        </div>
      </div>
    </div>
  )
}
