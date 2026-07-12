import { useState } from 'react'

interface Props {
  title: string
  confirmLabel: string
  initialName?: string
  initialDescription?: string
  isLoading?: boolean
  error?: string | null
  onSubmit: (name: string, description: string | null) => void
  onCancel: () => void
}

// A game collection's own metadata (name + optional description). Shared by the
// create and edit flows; membership is managed separately.
export function GameCollectionFormModal({
  title,
  confirmLabel,
  initialName = '',
  initialDescription = '',
  isLoading = false,
  error,
  onSubmit,
  onCancel,
}: Props) {
  const [name, setName] = useState(initialName)
  const [description, setDescription] = useState(initialDescription)
  const [validationError, setValidationError] = useState<string | null>(null)

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const trimmedName = name.trim()
    if (!trimmedName) {
      setValidationError('Name cannot be empty.')
      return
    }
    setValidationError(null)
    const trimmedDescription = description.trim()
    onSubmit(trimmedName, trimmedDescription === '' ? null : trimmedDescription)
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">{title}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <label>
            Name
            <input
              type="text"
              className="input"
              value={name}
              onChange={e => { setName(e.target.value); setValidationError(null) }}
              autoFocus
            />
          </label>
          <label>
            Description (optional)
            <textarea
              className="input"
              rows={3}
              value={description}
              onChange={e => setDescription(e.target.value)}
            />
          </label>
          {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray" disabled={isLoading}>Cancel</button>
            <button type="submit" className="btn-primary" disabled={isLoading}>{confirmLabel}</button>
          </div>
        </form>
      </div>
    </div>
  )
}
