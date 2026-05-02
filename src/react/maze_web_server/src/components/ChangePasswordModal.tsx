import { useEffect, useState } from 'react'
import { PasswordInput } from './PasswordInput'
import * as api from '../api/client'
import { useToken } from '../context/AuthContext'
import { validateChangePasswordForm, validateSetPasswordForm } from '../utils/passwordValidation'
import lockIcon from '../assets/icon_lock.svg'

interface Props {
  onClose: () => void
  /**
   * `true` when the authenticated user already has a password
   * (`UserProfile.has_password`). `false` for OAuth-only users adding a
   * password as a second login method — the popup then renders the
   * "Set Password" variant: no Current Password field, and the request
   * body omits `current_password` (the server returns 400 if it's
   * present in the set-initial flow).
   */
  hasPassword: boolean
  /**
   * Fired after a successful Set or Change. Lets the parent learn that
   * `has_password` is now `true` (matters for the Set path — the parent's
   * cached profile would otherwise stay stale until the modal is reopened).
   * The parent can flip its local state idempotently rather than re-fetching.
   */
  onSuccess?: () => void
}

export function ChangePasswordModal({ onClose, hasPassword, onSuccess }: Props) {
  const token = useToken() ?? ''
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const heading = hasPassword ? 'Change Password' : 'Set Password'
  const submitIdle = hasPassword ? 'Change Password' : 'Set Password'
  const submitBusy = hasPassword ? 'Changing...' : 'Setting...'
  const dialogLabel = hasPassword ? 'Change Password' : 'Set Password'

  useEffect(() => {
    document.body.classList.toggle('is-busy', isLoading)
    return () => document.body.classList.remove('is-busy')
  }, [isLoading])

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const err = hasPassword
      ? validateChangePasswordForm({ currentPassword, newPassword, confirmPassword })
      : validateSetPasswordForm({ newPassword, confirmPassword })
    if (err) { setError(err); return }

    setIsLoading(true)
    setError(null)
    try {
      const body = hasPassword
        ? { current_password: currentPassword, new_password: newPassword }
        : { new_password: newPassword }
      await api.changePassword(token, body)
      onSuccess?.()
      onClose()
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      if (hasPassword && status === 401) {
        setError('Current password is incorrect')
      } else {
        setError(hasPassword ? 'Failed to change password' : 'Failed to set password')
      }
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div role="dialog" aria-modal="true" aria-label={dialogLabel} className="modal-overlay" style={{ zIndex: 1100, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal modal-md modal-centered">
        <img src={lockIcon} alt="" width={64} height={64} className="auth-logo" />
        <h2 className="modal-title" style={{ marginTop: '1rem', marginBottom: '1rem' }}>{heading}</h2>
        <form onSubmit={handleSubmit} className="modal-form">
          {hasPassword && (
            <>
              <label htmlFor="currentPassword">Current Password</label>
              <PasswordInput id="currentPassword" value={currentPassword} onChange={setCurrentPassword} disabled={isLoading} />
            </>
          )}

          <label htmlFor="newPassword">New Password</label>
          <PasswordInput id="newPassword" value={newPassword} onChange={setNewPassword} disabled={isLoading} />

          <label htmlFor="confirmPassword">Confirm New Password</label>
          <PasswordInput id="confirmPassword" value={confirmPassword} onChange={setConfirmPassword} disabled={isLoading} />

          {error && <p role="alert" className="error-msg">{error}</p>}

          <div className="modal-actions">
            <button type="submit" className="btn-gray" disabled={isLoading}>
              {isLoading ? submitBusy : submitIdle}
            </button>
            <button type="button" className="btn-link" onClick={onClose} disabled={isLoading}>Back</button>
          </div>
        </form>
      </div>
    </div>
  )
}
