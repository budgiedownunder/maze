import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useAuth, useToken } from '../context/AuthContext'
import { ChangePasswordModal } from './ChangePasswordModal'
import { ConfirmModal } from './ConfirmModal'
import type { UserProfile } from '../types/api'
import { isValidEmail } from '../utils/validation'

interface Props {
  onClose: () => void
  /**
   * When true, render a one-line welcome banner above the form. Used by the
   * post-OAuth-signup auto-open flow (see `OAuthCallbackPage` →
   * `MazesPage`). The banner only shows on the auto-opened modal; manual
   * opens via the burger menu always pass `welcome={false}` (default).
   */
  welcome?: boolean
}

type ModalView = 'account' | 'changePassword'

export function AccountModal({ onClose, welcome = false }: Props) {
  const token = useToken() ?? ''
  const { logout } = useAuth()
  const navigate = useNavigate()
  const [view, setView] = useState<ModalView>('account')
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState<UserProfile | null>(null)
  const [username, setUsername] = useState('')
  const [fullName, setFullName] = useState('')
  const [email, setEmail] = useState('')

  useEffect(() => {
    const busy = isSaving || isLoading || isDeleting
    document.body.classList.toggle('is-busy', busy)
    return () => document.body.classList.remove('is-busy')
  }, [isSaving, isLoading, isDeleting])

  useEffect(() => {
    api.getMe(token)
      .then(profile => {
        setSaved(profile)
        setUsername(profile.username)
        setFullName(profile.full_name)
        setEmail(profile.email)
      })
      .catch(() => setError('Failed to load profile'))
      .finally(() => setIsLoading(false))
  }, [token])

  const hasChanges = saved !== null && (
    username !== saved.username ||
    fullName !== saved.full_name ||
    email !== saved.email
  )
  const emailValid = isValidEmail(email)
  const saveDisabled = isSaving || isLoading || !hasChanges || !username.trim() || !emailValid

  async function handleSave(e: React.FormEvent) {
    e.preventDefault()
    setIsSaving(true)
    setError(null)
    try {
      const updated = await api.updateProfile(token, { username, full_name: fullName, email })
      setSaved(updated)
      setUsername(updated.username)
      setFullName(updated.full_name)
      setEmail(updated.email)
    } catch (ex: unknown) {
      const err = ex as { status?: number; message?: string }
      setError(err.status === 409 ? 'Username or email already in use' : (err.message ?? 'Failed to save profile'))
    } finally {
      setIsSaving(false)
    }
  }

  async function handleDeleteConfirm() {
    setIsDeleting(true)
    try {
      await api.deleteMe(token)
      await logout()
      navigate('/login', { replace: true })
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to delete account')
      setShowDeleteConfirm(false)
    } finally {
      setIsDeleting(false)
    }
  }

  if (view === 'changePassword') {
    return <ChangePasswordModal onClose={() => setView('account')} />
  }

  return (
    <>
    {showDeleteConfirm && (
      <ConfirmModal
        title="Delete Account"
        message="Are you sure you want to permanently delete your account? This will also delete all your mazes and cannot be undone."
        confirmLabel="Delete"
        isDangerous
        onConfirm={handleDeleteConfirm}
        onCancel={() => setShowDeleteConfirm(false)}
      />
    )}
    <div role="dialog" aria-modal="true" aria-label="My Account" className="modal-overlay" style={{ cursor: (isSaving || isLoading) ? 'wait' : undefined }}>
      <div className="modal modal-md">
        <h2 className="modal-title">My Account</h2>

        {welcome && (
          <p role="status" className="account-welcome-banner">
            Welcome to Maze! Take a moment to set your username and full name.
          </p>
        )}

        {isLoading ? (
          <p>Loading profile...</p>
        ) : (
          <form onSubmit={handleSave} className="modal-form">
            <label htmlFor="acc-username">Username</label>
            <input id="acc-username" value={username} onChange={e => setUsername(e.target.value)} disabled={isSaving} />

            <label htmlFor="acc-fullname">Full Name</label>
            <input id="acc-fullname" value={fullName} onChange={e => setFullName(e.target.value)} disabled={isSaving} />

            <label htmlFor="acc-email">Email</label>
            <input id="acc-email" type="email" value={email} onChange={e => setEmail(e.target.value)} disabled={isSaving} />
            {!isLoading && !emailValid && (
              <p role="alert" className="error-msg">{email.trim() ? 'Please enter a valid email address' : 'Email is required'}</p>
            )}

            {saved?.is_admin && (
              <span className="badge-admin">Administrator</span>
            )}

            {error && <p role="alert" className="error-msg">{error}</p>}

            <button type="submit" disabled={saveDisabled} className="btn-gray">
              {isSaving ? 'Saving...' : 'Save Profile'}
            </button>
          </form>
        )}

        <div className="modal-actions">
          <button type="button" onClick={() => setView('changePassword')} disabled={isLoading} className="btn-link">
            Change Password
          </button>
          <button type="button" onClick={() => setShowDeleteConfirm(true)} disabled={isLoading} className="btn-danger">
            Delete Account
          </button>
          <button type="button" onClick={onClose} className="btn-gray">Close</button>
        </div>
      </div>
    </div>
    </>
  )
}
