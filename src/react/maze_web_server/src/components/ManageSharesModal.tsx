import { useEffect, useMemo, useState } from 'react'
import { useToken } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { Avatar } from './Avatar'
import {
  grantGameCollectionShare,
  grantGameDefinitionShare,
  listGameCollectionShares,
  listGameDefinitionShares,
  lookupUsers,
  revokeGameCollectionShare,
  revokeGameDefinitionShare,
} from '../api/client'
import type { GranteeSummary, UserLookupEntry } from '../types/api'

// The thing being shared. `kind` selects which set of share endpoints to call;
// `name` is shown in the modal so the owner knows what they are granting.
export interface ShareSubject {
  kind: 'definition' | 'collection'
  id: string
  name: string
}

interface Props {
  subject: ShareSubject
  onClose: () => void
}

const LOOKUP_LIMIT = 8
const DEBOUNCE_MS = 250

// Grant / revoke / list access for a definition or collection: a live grantee
// list (resolved to usernames by the server) plus a username people-picker that
// searches the B5 lookup as you type. Reused across the games + collections areas.
export function ManageSharesModal({ subject, onClose }: Props) {
  const token = useToken()

  // The share endpoints come in definition / collection pairs with identical
  // shapes; bind the trio to the subject's kind.
  const api = useMemo(
    () =>
      subject.kind === 'definition'
        ? { list: listGameDefinitionShares, grant: grantGameDefinitionShare, revoke: revokeGameDefinitionShare }
        : { list: listGameCollectionShares, grant: grantGameCollectionShare, revoke: revokeGameCollectionShare },
    [subject.kind],
  )

  // Grantee list keyed by a refresh counter, so a reload re-derives the view
  // rather than setting state mid-effect (a grant/revoke bumps the counter).
  const [refresh, setRefresh] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; grantees: GranteeSummary[] } | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (!token) return
    let cancelled = false
    const key = refresh
    api.list(token, subject.id)
      .then(r => { if (!cancelled) setLoaded({ key, grantees: r.grantees }) })
      .catch((ex: unknown) => { if (!cancelled) setError((ex as Error).message || 'Failed to load shares.') })
    return () => { cancelled = true }
  }, [token, subject.id, api, refresh])

  const grantees = loaded != null && loaded.key === refresh ? loaded.grantees : null
  const isLoadingGrantees = grantees == null && error == null

  // Global wait cursor while the grantee list is loading and while a grant/revoke
  // is in flight (a mutation also re-loads the list, so this covers the reload).
  useBusyCursor(busy || isLoadingGrantees)

  // People-picker: debounce the query, then search the username lookup. Results
  // for an empty query are derived (never fetched), so nothing is set in the
  // effect body synchronously.
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<UserLookupEntry[]>([])
  // Whether the server has further matches beyond the page we fetched — drives a
  // "narrow your search" hint, since the picker pages rather than scrolling all.
  const [hasMore, setHasMore] = useState(false)

  useEffect(() => {
    const q = query.trim()
    if (!token || q === '') return
    let cancelled = false
    const handle = setTimeout(() => {
      lookupUsers(token, { username: q, limit: LOOKUP_LIMIT })
        .then(r => { if (!cancelled) { setResults(r.users); setHasMore(r.has_more) } })
        .catch(() => { if (!cancelled) { setResults([]); setHasMore(false) } })
    }, DEBOUNCE_MS)
    return () => { cancelled = true; clearTimeout(handle) }
  }, [token, query])

  // Hide already-granted users from the picker (grant is idempotent server-side,
  // but re-offering them would be confusing).
  const grantedIds = new Set((grantees ?? []).map(g => g.id))
  const pickable = query.trim() === '' ? [] : results.filter(u => !grantedIds.has(u.id))

  async function handleGrant(userId: string) {
    setBusy(true)
    setError(null)
    try {
      await api.grant(token!, subject.id, userId)
      setQuery('')
      setResults([])
      setRefresh(c => c + 1)
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to grant access.')
    } finally {
      setBusy(false)
    }
  }

  async function handleRevoke(userId: string) {
    setBusy(true)
    setError(null)
    try {
      await api.revoke(token!, subject.id, userId)
      setRefresh(c => c + 1)
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to revoke access.')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div role="dialog" aria-modal="true" aria-label={`Share: ${subject.name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">Share: {subject.name}</h2>

        <div className="share-body">
          <div className="field-group">
            <p className="field-group-title">Add User</p>
            <input
              type="text"
              className="input"
              aria-label="Add user"
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder="Start typing a username…"
              autoFocus
            />
            {pickable.length > 0 && (
              <ul className="share-picker-results">
                {pickable.map(u => (
                  <li key={u.id}>
                    <button type="button" className="btn-secondary" disabled={busy} onClick={() => void handleGrant(u.id)} aria-label={`Add ${u.username}`}>
                      {u.username}
                    </button>
                  </li>
                ))}
              </ul>
            )}
            {query.trim() !== '' && hasMore && (
              <p className="share-picker-hint">More matches — keep typing to narrow.</p>
            )}
          </div>

          {error && <p role="alert" className="error-msg">{error}</p>}

          <div className="field-group">
            <p className="field-group-title">Shared with</p>
            {isLoadingGrantees && <p aria-label="Loading">Loading…</p>}
            {grantees != null && grantees.length === 0 && <p>No one has access yet.</p>}
            {grantees != null && grantees.length > 0 && (
              <ul className="share-grantees">
                {grantees.map(g => (
                  <li key={g.id}>
                    <span className="share-grantee-user">
                      <Avatar userId={g.id} avatarUpdatedAt={g.avatar_updated_at} size={24} />
                      <span>{g.username}</span>
                    </span>
                    <button type="button" className="btn-icon" disabled={busy} onClick={() => void handleRevoke(g.id)} aria-label={`Remove ${g.username}`}>
                      <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" width={18} height={18} />
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <div className="modal-actions-row" style={{ marginTop: '1.25rem' }}>
          <button type="button" className="btn-gray" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  )
}
