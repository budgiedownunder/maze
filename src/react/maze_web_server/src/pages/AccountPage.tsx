import { useEffect, useRef, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useAuth, useToken } from '../context/AuthContext'
import { AppHeader } from '../components/AppHeader'
import { Avatar } from '../components/Avatar'
import { ChangePasswordModal } from '../components/ChangePasswordModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { EmailAddressesPanel } from '../components/EmailAddressesPanel'
import type { UserProfile } from '../types/api'

type View = 'account' | 'changePassword'

export function AccountPage() {
  const token = useToken() ?? ''
  const { logout, refreshProfile } = useAuth()
  const navigate = useNavigate()
  const location = useLocation()

  // Read the one-shot `welcome` flag from navigation state. Set by
  // `OAuthCallbackPage` for first-time OAuth signups so users notice
  // their auto-generated username and can edit it before doing anything
  // else. Cleared from history immediately so a page refresh doesn't
  // re-show the banner.
  const [welcome, setWelcome] = useState(false)
  useEffect(() => {
    const navState = location.state as { welcome?: boolean } | null
    if (navState?.welcome) {
      setWelcome(true)
      window.history.replaceState({}, '', location.pathname + location.search)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const [view, setView] = useState<View>('account')
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState<UserProfile | null>(null)
  const [username, setUsername] = useState('')
  const [fullName, setFullName] = useState('')

  const fileInputRef = useRef<HTMLInputElement>(null)
  const [avatarBusy, setAvatarBusy] = useState(false)
  const [avatarError, setAvatarError] = useState<string | null>(null)

  useEffect(() => {
    const busy = isSaving || isLoading || isDeleting || avatarBusy
    document.body.classList.toggle('is-busy', busy)
    return () => document.body.classList.remove('is-busy')
  }, [isSaving, isLoading, isDeleting, avatarBusy])

  useEffect(() => {
    api.getMe(token)
      .then(profile => {
        setSaved(profile)
        setUsername(profile.username)
        setFullName(profile.full_name)
      })
      .catch(() => setError('Failed to load profile'))
      .finally(() => setIsLoading(false))
  }, [token])

  const hasChanges = saved !== null && (
    username !== saved.username ||
    fullName !== saved.full_name
  )
  const saveDisabled = isSaving || isLoading || !hasChanges || !username.trim()

  async function handleSave(e: React.FormEvent) {
    e.preventDefault()
    setIsSaving(true)
    setError(null)
    try {
      const updated = await api.updateProfile(token, { username, full_name: fullName })
      setSaved(updated)
      setUsername(updated.username)
      setFullName(updated.full_name)
    } catch (ex: unknown) {
      const err = ex as { status?: number; message?: string }
      setError(err.status === 409 ? 'Username already in use' : (err.message ?? 'Failed to save profile'))
    } finally {
      setIsSaving(false)
    }
  }

  // Mirrors the server's accepted formats + 2 MiB cap so an oversize/ wrong
  // file is rejected before a pointless upload round-trip.
  const ACCEPTED_AVATAR_TYPES = ['image/png', 'image/jpeg']
  const MAX_AVATAR_BYTES = 2 * 1024 * 1024

  async function handleAvatarFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    // Reset the input so picking the *same* file again re-fires `change`.
    e.target.value = ''
    if (!file) return
    setAvatarError(null)
    if (!ACCEPTED_AVATAR_TYPES.includes(file.type)) {
      setAvatarError('Please choose a PNG or JPEG image.')
      return
    }
    if (file.size > MAX_AVATAR_BYTES) {
      setAvatarError('Image must be 2 MB or smaller.')
      return
    }
    setAvatarBusy(true)
    try {
      const { avatar_updated_at } = await api.uploadAvatar(token, file)
      setSaved(s => (s ? { ...s, avatar_updated_at } : s))
      await refreshProfile() // update the header avatar too
    } catch (ex: unknown) {
      setAvatarError((ex as { message?: string }).message ?? 'Failed to upload avatar')
    } finally {
      setAvatarBusy(false)
    }
  }

  async function handleAvatarRemove() {
    setAvatarError(null)
    setAvatarBusy(true)
    try {
      await api.deleteAvatar(token)
      setSaved(s => (s ? { ...s, avatar_updated_at: null } : s))
      await refreshProfile()
    } catch (ex: unknown) {
      setAvatarError((ex as { message?: string }).message ?? 'Failed to remove avatar')
    } finally {
      setAvatarBusy(false)
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
    // saved is non-null here because the changePassword view is only
    // reachable via the trigger button, which is disabled while loading.
    return <ChangePasswordModal
      onClose={() => setView('account')}
      hasPassword={saved?.has_password ?? true}
      onSuccess={() => setSaved(s => s ? { ...s, has_password: true } : s)}
    />
  }

  return (
    <div className="account-page" style={{ cursor: (isSaving || isLoading) ? 'wait' : undefined }}>
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
      <AppHeader title="My Account" titleAs="h1" />
      <main className="account-main">
        {welcome && (
          <p role="status" className="account-welcome-banner">
            Welcome to Maze! Take a moment to set your username and full name.
          </p>
        )}

        {!isLoading && saved && <h3 className="account-section-title">Profile</h3>}

        {!isLoading && saved && (
          <section className="account-avatar" aria-label="Avatar">
            <button
              type="button"
              className="account-avatar-button"
              disabled={avatarBusy}
              onClick={() => fileInputRef.current?.click()}
              aria-label={saved.avatar_updated_at ? 'Change avatar' : 'Upload avatar'}
              title={saved.avatar_updated_at ? 'Change avatar' : 'Upload avatar'}
            >
              <Avatar userId={saved.id} avatarUpdatedAt={saved.avatar_updated_at} size={96} alt="Your avatar" />
            </button>
            <div className="account-avatar-actions">
              <button
                type="button"
                className="btn-gray"
                disabled={avatarBusy}
                onClick={() => fileInputRef.current?.click()}
              >
                {avatarBusy ? 'Working...' : saved.avatar_updated_at ? 'Change' : 'Upload'}
              </button>
              {saved.avatar_updated_at && (
                <button type="button" className="btn-link" disabled={avatarBusy} onClick={handleAvatarRemove}>
                  Remove
                </button>
              )}
              <input
                ref={fileInputRef}
                type="file"
                accept="image/png,image/jpeg"
                onChange={handleAvatarFile}
                hidden
              />
              {avatarError && <p role="alert" className="error-msg">{avatarError}</p>}
            </div>
          </section>
        )}

        {isLoading ? (
          <p>Loading profile...</p>
        ) : (
          <form onSubmit={handleSave} className="account-form">
            <label htmlFor="acc-username">Username</label>
            <input id="acc-username" value={username} onChange={e => setUsername(e.target.value)} disabled={isSaving} />

            <label htmlFor="acc-fullname">Full Name</label>
            <input id="acc-fullname" value={fullName} onChange={e => setFullName(e.target.value)} disabled={isSaving} />

            {saved?.is_admin && (
              <span className="badge-admin">Administrator</span>
            )}

            {error && <p role="alert" className="error-msg">{error}</p>}

            <button type="submit" disabled={saveDisabled} className="btn-gray">
              {isSaving ? 'Saving...' : 'Save Profile'}
            </button>
          </form>
        )}

        {!isLoading && <EmailAddressesPanel token={token} />}

        <div className="account-actions">
          <button type="button" onClick={() => setView('changePassword')} disabled={isLoading} className="btn-link">
            {saved?.has_password === false ? 'Set Password' : 'Change Password'}
          </button>
          <button type="button" onClick={() => setShowDeleteConfirm(true)} disabled={isLoading} className="btn-danger">
            Delete Account
          </button>
        </div>
      </main>
    </div>
  )
}
