import { useState } from 'react'
import { PasswordInput } from './PasswordInput'
import * as api from '../api/client'
import { useToken } from '../context/AuthContext'

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
    <div role="dialog" aria-modal="true" aria-label="Change Password" style={overlayStyle}>
      <div style={modalStyle}>
        <h2 style={{ marginTop: 0 }}>Change Password</h2>
        <form onSubmit={handleSubmit} style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
          <label htmlFor="currentPassword">Current Password</label>
          <PasswordInput id="currentPassword" value={currentPassword} onChange={setCurrentPassword} disabled={isLoading} />

          <label htmlFor="newPassword">New Password</label>
          <PasswordInput id="newPassword" value={newPassword} onChange={setNewPassword} disabled={isLoading} />

          <label htmlFor="confirmPassword">Confirm New Password</label>
          <PasswordInput id="confirmPassword" value={confirmPassword} onChange={setConfirmPassword} disabled={isLoading} />

          {error && <p role="alert" style={{ color: 'red', margin: 0 }}>{error}</p>}

          <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end', marginTop: '0.5rem' }}>
            <button type="button" onClick={onClose} disabled={isLoading}>Back</button>
            <button type="submit" disabled={!!validationError || isLoading}>
              {isLoading ? 'Changing...' : 'Change Password'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed', inset: 0,
  background: 'rgba(0,0,0,0.6)',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  zIndex: 1100,
}

const modalStyle: React.CSSProperties = {
  background: '#fff',
  borderRadius: '0.5rem',
  padding: '2rem',
  minWidth: '300px',
  maxWidth: '400px',
  width: '100%',
}
