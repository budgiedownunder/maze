import { useState } from 'react'
import { PasswordInput } from './PasswordInput'
import * as api from '../api/client'
import { useToken } from '../context/AuthContext'
import lockIcon from '../assets/icon_lock.svg'

export function validateChangePasswordForm(fields: {
  currentPassword: string
  newPassword: string
  confirmPassword: string
}): string | null {
  if (!fields.currentPassword || !fields.newPassword || !fields.confirmPassword) {
    return 'All fields are required'
  }
  if (fields.newPassword !== fields.confirmPassword) {
    return 'New passwords do not match'
  }
  if (fields.newPassword.length < 8) {
    return 'Password must be at least 8 characters'
  }
  if (!/[A-Z]/.test(fields.newPassword)) return 'Password must contain an uppercase letter'
  if (!/[a-z]/.test(fields.newPassword)) return 'Password must contain a lowercase letter'
  if (!/[0-9]/.test(fields.newPassword)) return 'Password must contain a digit'
  if (!/[^A-Za-z0-9]/.test(fields.newPassword)) return 'Password must contain a special character'
  return null
}

interface Props {
  onClose: () => void
}

export function ChangePasswordModal({ onClose }: Props) {
  const token = useToken() ?? ''
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const validationError = validateChangePasswordForm({ currentPassword, newPassword, confirmPassword })

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const err = validateChangePasswordForm({ currentPassword, newPassword, confirmPassword })
    if (err) { setError(err); return }

    setIsLoading(true)
    setError(null)
    try {
      await api.changePassword(token, { current_password: currentPassword, new_password: newPassword })
      onClose()
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      setError(status === 401 ? 'Current password is incorrect' : 'Failed to change password')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div role="dialog" aria-modal="true" aria-label="Change Password" className="modal-overlay" style={{ zIndex: 1100 }}>
      <div className="modal modal-md modal-centered">
        <img src={lockIcon} alt="" width={64} height={64} className="auth-logo" />
        <h2 className="modal-title" style={{ marginTop: '1rem', marginBottom: '1rem' }}>Change Password</h2>
        <form onSubmit={handleSubmit} className="modal-form">
          <label htmlFor="currentPassword">Current Password</label>
          <PasswordInput id="currentPassword" value={currentPassword} onChange={setCurrentPassword} disabled={isLoading} />

          <label htmlFor="newPassword">New Password</label>
          <PasswordInput id="newPassword" value={newPassword} onChange={setNewPassword} disabled={isLoading} />

          <label htmlFor="confirmPassword">Confirm New Password</label>
          <PasswordInput id="confirmPassword" value={confirmPassword} onChange={setConfirmPassword} disabled={isLoading} />

          {error && <p role="alert" className="error-msg">{error}</p>}

          <div className="modal-actions">
            <button type="submit" className="btn-gray" disabled={!!validationError || isLoading}>
              {isLoading ? 'Changing...' : 'Change Password'}
            </button>
            <button type="button" className="btn-link" onClick={onClose} disabled={isLoading}>Back</button>
          </div>
        </form>
      </div>
    </div>
  )
}
