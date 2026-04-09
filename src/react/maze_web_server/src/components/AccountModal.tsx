import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useAuth, useToken } from '../context/AuthContext'
import { ChangePasswordModal } from './ChangePasswordModal'
import type { UserProfile } from '../types/api'

interface Props {
  onClose: () => void
}

type ModalView = 'account' | 'changePassword' | 'confirmDelete'

export function AccountModal({ onClose }: Props) {
  const token = useToken() ?? ''
  const { logout } = useAuth()
  const navigate = useNavigate()
  const [view, setView] = useState<ModalView>('account')
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState<UserProfile | null>(null)
  const [username, setUsername] = useState('')
  const [fullName, setFullName] = useState('')
  const [email, setEmail] = useState('')

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
  const saveDisabled = isSaving || isLoading || !hasChanges || !username.trim()

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
      const status = (ex as { status?: number }).status
      setError(status === 409 ? 'Username or email already in use' : 'Failed to save profile')
    } finally {
      setIsSaving(false)
    }
  }

  async function handleDeleteConfirm() {
    try {
      await api.deleteMe(token)
      await logout()
      navigate('/login', { replace: true })
    } catch {
      setError('Failed to delete account')
      setView('account')
    }
  }

  if (view === 'changePassword') {
    return <ChangePasswordModal onClose={() => setView('account')} />
  }

  return (
    <div role="dialog" aria-modal="true" aria-label="My Account" style={overlayStyle}>
      <div style={modalStyle}>
        <h2 style={{ marginTop: 0 }}>My Account</h2>

        {isLoading ? (
          <p>Loading profile...</p>
        ) : (
          <form onSubmit={handleSave} style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {saved?.is_admin && (
              <span style={{ alignSelf: 'flex-start', background: '#4f46e5', color: '#fff', borderRadius: '0.25rem', padding: '0.1rem 0.5rem', fontSize: '0.8rem' }}>
                Administrator
              </span>
            )}

            <label htmlFor="acc-username">Username</label>
            <input id="acc-username" value={username} onChange={e => setUsername(e.target.value)} disabled={isSaving} />

            <label htmlFor="acc-fullname">Full Name</label>
            <input id="acc-fullname" value={fullName} onChange={e => setFullName(e.target.value)} disabled={isSaving} />

            <label htmlFor="acc-email">Email</label>
            <input id="acc-email" type="email" value={email} onChange={e => setEmail(e.target.value)} disabled={isSaving} />

            {error && <p role="alert" style={{ color: 'red', margin: 0 }}>{error}</p>}

            <button type="submit" disabled={saveDisabled}>
              {isSaving ? 'Saving...' : 'Save Profile'}
            </button>
          </form>
        )}

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1rem', width: '100%' }}>
          <button type="button" onClick={() => setView('changePassword')} disabled={isLoading}>
            Change Password
          </button>

          {view === 'confirmDelete' ? (
            <>
              <p style={{ color: '#b91c1c', margin: '0.5rem 0' }}>
                Are you sure? This will delete all your mazes and cannot be undone.
              </p>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button type="button" onClick={handleDeleteConfirm} style={{ background: '#b91c1c', color: '#fff' }}>
                  Confirm
                </button>
                <button type="button" onClick={() => setView('account')}>Cancel</button>
              </div>
            </>
          ) : (
            <button type="button" onClick={() => setView('confirmDelete')} style={{ color: '#b91c1c', borderColor: '#b91c1c' }}>
              Delete Account
            </button>
          )}

          <button type="button" onClick={onClose} style={{ marginTop: '0.25rem' }}>Close</button>
        </div>
      </div>
    </div>
  )
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed', inset: 0,
  background: 'rgba(0,0,0,0.5)',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  zIndex: 1000,
}

const modalStyle: React.CSSProperties = {
  background: '#fff',
  borderRadius: '0.5rem',
  padding: '2rem',
  minWidth: '300px',
  maxWidth: '420px',
  width: '100%',
}
