import { useEffect, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import * as api from '../api/client'
import { useAuth, useToken } from '../context/AuthContext'
import { useTheme } from '../context/ThemeContext'
import { useMenuVariant } from '../hooks/useMenuVariant'
import { HamburgerMenu } from '../components/HamburgerMenu'
import { ChangePasswordModal } from '../components/ChangePasswordModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { EmailAddressesPanel } from '../components/EmailAddressesPanel'
import type { UserProfile } from '../types/api'

type View = 'account' | 'changePassword'

export function AccountPage() {
  const token = useToken() ?? ''
  const { logout } = useAuth()
  const navigate = useNavigate()
  const location = useLocation()
  const menuVariant = useMenuVariant()
  const { theme, toggleTheme } = useTheme()

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
      <header className="app-header">
        <div className="header-actions">
          {menuVariant === 'hamburger' && <HamburgerMenu />}
        </div>
        <h1 className="app-header-title">My Account</h1>
        <div className="header-actions">
          <button
            className="theme-toggle"
            onClick={toggleTheme}
            aria-label={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
            title={theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {theme === 'dark' ? '☀' : '☾'}
          </button>
        </div>
      </header>
      <main className="account-main">
        {welcome && (
          <p role="status" className="account-welcome-banner">
            Welcome to Maze! Take a moment to set your username and full name.
          </p>
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
