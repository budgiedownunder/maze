import { useEffect, useState } from 'react'
import * as api from '../api/client'
import type { UserEmail } from '../types/api'
import { isValidEmail } from '../utils/validation'

interface Props {
  token: string
}

export function EmailAddressesPanel({ token }: Props) {
  const [emails, setEmails] = useState<UserEmail[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busyEmail, setBusyEmail] = useState<string | null>(null)
  const [newEmail, setNewEmail] = useState('')
  const [isAdding, setIsAdding] = useState(false)

  useEffect(() => {
    let cancelled = false
    api.getMyEmails(token)
      .then(res => { if (!cancelled) setEmails(res.emails) })
      .catch(() => { if (!cancelled) setError('Failed to load emails') })
    return () => { cancelled = true }
  }, [token])

  // Optimistic-write helper. The optimistic snapshot is applied immediately
  // so the UI feels instant; on failure we revert and surface a panel-level
  // error message.
  async function runWrite(email: string, optimistic: UserEmail[], fire: () => Promise<{ emails: UserEmail[] }>, errorMessage: string) {
    if (emails === null) return
    const previous = emails
    setBusyEmail(email)
    setError(null)
    setEmails(optimistic)
    try {
      const res = await fire()
      setEmails(res.emails)
    } catch (ex: unknown) {
      setEmails(previous)
      const status = (ex as { status?: number }).status
      const message = (ex as { message?: string }).message
      setError(status && message ? `${errorMessage}: ${message}` : errorMessage)
    } finally {
      setBusyEmail(null)
    }
  }

  function handleMakePrimary(email: string) {
    if (emails === null) return
    const optimistic = emails.map(row => ({ ...row, is_primary: row.email === email }))
    return runWrite(email, optimistic, () => api.setPrimaryEmail(token, email), 'Failed to set primary email')
  }

  function handleRemove(email: string) {
    if (emails === null) return
    const optimistic = emails.filter(row => row.email !== email)
    return runWrite(email, optimistic, () => api.removeMyEmail(token, email), 'Failed to remove email')
  }

  // The add-form path doesn't lend itself to optimistic update — the row's
  // verified/verified_at fields come from the server and we don't want to
  // fabricate them. Send first, set the list from the response.
  async function handleAddEmail(e: React.FormEvent) {
    e.preventDefault()
    if (!isValidEmail(newEmail) || isAdding) return
    setIsAdding(true)
    setError(null)
    try {
      const res = await api.addMyEmail(token, newEmail)
      setEmails(res.emails)
      setNewEmail('')
    } catch (ex: unknown) {
      const status = (ex as { status?: number }).status
      const message = (ex as { message?: string }).message
      const baseMessage = status === 409
        ? 'That email is already in use'
        : status === 400
          ? 'Email format is invalid'
          : 'Failed to add email'
      setError(message && status !== 409 && status !== 400 ? `${baseMessage}: ${message}` : baseMessage)
    } finally {
      setIsAdding(false)
    }
  }

  return (
    <section className="email-section">
      <h3 className="email-section-title">Email Addresses</h3>
      {emails === null && !error && <p>Loading emails...</p>}
      {error && <p role="alert" className="error-msg">{error}</p>}
      {emails !== null && (
        <ul className="email-list">
          {emails.map(row => {
            const isOnly = emails.length === 1
            const canRemove = !row.is_primary && !isOnly
            const makePrimaryDisabled = !row.verified || busyEmail !== null
            const makePrimaryTitle = !row.verified
              ? 'Only verified emails can be promoted to primary'
              : undefined
            return (
              <li key={row.email} className="email-row">
                <div className="email-row-main">
                  <span className="email-row-address">{row.email}</span>
                  {row.is_primary && <span className="badge-primary">Primary</span>}
                  <span className={row.verified ? 'email-pill email-pill--verified' : 'email-pill email-pill--unverified'}>
                    {row.verified ? 'Verified' : 'Unverified'}
                  </span>
                </div>
                <div className="email-row-actions">
                  {!row.is_primary && (
                    <button
                      type="button"
                      className="btn-link"
                      disabled={makePrimaryDisabled}
                      title={makePrimaryTitle}
                      onClick={() => handleMakePrimary(row.email)}
                    >
                      Make Primary
                    </button>
                  )}
                  {canRemove && (
                    <button
                      type="button"
                      className="btn-link"
                      disabled={busyEmail !== null}
                      onClick={() => handleRemove(row.email)}
                    >
                      Remove
                    </button>
                  )}
                  {!row.verified && (
                    <button
                      type="button"
                      className="btn-link"
                      disabled
                      title="Email verification is not yet available"
                    >
                      Resend Verification
                    </button>
                  )}
                </div>
              </li>
            )
          })}
        </ul>
      )}
      {emails !== null && (
        <form onSubmit={handleAddEmail} className="email-add-form">
          <label htmlFor="add-email-input" className="visually-hidden">Add another email</label>
          <input
            id="add-email-input"
            type="email"
            placeholder="Add another email address"
            value={newEmail}
            onChange={e => setNewEmail(e.target.value)}
            disabled={isAdding}
          />
          <button
            type="submit"
            className="btn-link"
            disabled={isAdding || !isValidEmail(newEmail)}
          >
            {isAdding ? 'Adding...' : 'Add Email'}
          </button>
        </form>
      )}
    </section>
  )
}
