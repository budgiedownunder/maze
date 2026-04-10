import { useState } from 'react'

interface Props {
  title: string
  label: string
  initialValue: string
  confirmLabel: string
  validate?: (value: string) => string | null
  isLoading?: boolean
  error?: string | null
  onConfirm: (value: string) => void
  onCancel: () => void
}

export function PromptModal({
  title,
  label,
  initialValue,
  confirmLabel,
  validate,
  isLoading = false,
  error,
  onConfirm,
  onCancel,
}: Props) {
  const [value, setValue] = useState(initialValue)
  const [validationError, setValidationError] = useState<string | null>(null)

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const trimmed = value.trim()
    if (!trimmed) {
      setValidationError('Name cannot be empty.')
      return
    }
    const customError = validate?.(trimmed)
    if (customError) {
      setValidationError(customError)
      return
    }
    setValidationError(null)
    onConfirm(trimmed)
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">{title}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <label>
            {label}
            <input
              type="text"
              className="input"
              value={value}
              onChange={e => { setValue(e.target.value); setValidationError(null) }}
              autoFocus
            />
          </label>
          {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-submit">{confirmLabel}</button>
          </div>
        </form>
      </div>
    </div>
  )
}
