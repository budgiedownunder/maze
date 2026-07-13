import { useEffect, useMemo, useState } from 'react'
import { useToken } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { Avatar } from './Avatar'
import {
  listGameCollectionShares,
  listGameDefinitionShares,
  lookupUsers,
  setGameCollectionShares,
  setGameDefinitionShares,
} from '../api/client'
import { VISIBILITIES, accessDescription, accessLabel, type Visibility } from '../utils/gameDefinitions'
import type { GranteeSummary, UserLookupEntry } from '../types/api'

// The thing whose access is being managed. `kind` selects which endpoints to
// call; `name` titles the modal; `ownerId` is excluded from the people-picker.
export interface ShareSubject {
  kind: 'definition' | 'collection'
  id: string
  name: string
  ownerId: string
}

interface Props {
  subject: ShareSubject
  // The current access tier when the modal opens.
  visibility: Visibility
  // Whether to offer the admin-only Featured tier.
  isAdmin: boolean
  // Persists the chosen tier (the parent PUTs the entity, config preserved).
  onSetVisibility: (visibility: Visibility) => Promise<void>
  // Called after a successful Save (the parent closes + refreshes the row).
  onSaved: () => void
  // Cancel — discards staged edits.
  onClose: () => void
}

const LOOKUP_LIMIT = 8
const DEBOUNCE_MS = 250

// Manage a game's access in one place: an access-tier control (Just me / Specific
// people / Everyone / Featured[admin]) plus, when "Specific people" is chosen, a
// username people-picker. Edits are staged locally and committed together on
// Save (reconcile the share list + set the tier); Cancel discards.
export function ManageSharesModal({ subject, visibility, isAdmin, onSetVisibility, onSaved, onClose }: Props) {
  const token = useToken()

  const api = useMemo(
    () =>
      subject.kind === 'definition'
        ? { list: listGameDefinitionShares, set: setGameDefinitionShares }
        : { list: listGameCollectionShares, set: setGameCollectionShares },
    [subject.kind],
  )

  // The stored grantee list (loaded once) and the staged, in-progress edit.
  const [original, setOriginal] = useState<GranteeSummary[] | null>(null)
  const [staged, setStaged] = useState<GranteeSummary[]>([])
  const [tier, setTier] = useState<Visibility>(visibility)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (!token) return
    let cancelled = false
    api.list(token, subject.id)
      .then(r => { if (!cancelled) { setOriginal(r.grantees); setStaged(r.grantees) } })
      .catch((ex: unknown) => { if (!cancelled) setError((ex as Error).message || 'Failed to load access.') })
    return () => { cancelled = true }
  }, [token, subject.id, api])

  const isLoading = original == null && error == null
  useBusyCursor(busy || isLoading)

  // People-picker: debounced username lookup (results for a blank query are
  // derived, never fetched).
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<UserLookupEntry[]>([])
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

  const stagedIds = new Set(staged.map(g => g.id))
  const pickable = query.trim() === ''
    ? []
    : results.filter(u => u.id !== subject.ownerId && !stagedIds.has(u.id))

  function stageAdd(user: UserLookupEntry) {
    setStaged(prev => [...prev, { id: user.id, username: user.username }])
    setQuery('')
    setResults([])
  }

  function stageRemove(id: string) {
    setStaged(prev => prev.filter(g => g.id !== id))
  }

  // A save is offered only when something changed.
  const originalIds = useMemo(() => new Set((original ?? []).map(g => g.id)), [original])
  const granteesChanged = original != null
    && (staged.length !== original.length || staged.some(g => !originalIds.has(g.id)))
  const dirty = tier !== visibility || granteesChanged

  async function handleSave() {
    setBusy(true)
    setError(null)
    try {
      // "Specific people" with no one staged is functionally private, so persist
      // it as such — otherwise the game reads back as shared with an empty list.
      const effectiveTier: Visibility = tier === 'shared' && staged.length === 0 ? 'private' : tier
      // A non-shared tier keeps no grantee list, so clear it; shared commits the
      // staged set. The server reconciles the stored list to match in one call.
      const userIds = effectiveTier === 'shared' ? staged.map(g => g.id) : []
      await api.set(token!, subject.id, userIds)
      // The chosen tier is authoritative and set explicitly.
      await onSetVisibility(effectiveTier)
      onSaved()
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to save access.')
      setBusy(false)
    }
  }

  const tiers = VISIBILITIES.filter(v => v !== 'curated' || isAdmin)

  return (
    <div role="dialog" aria-modal="true" aria-label={`Access: ${subject.name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm modal-capped">
        <h2 className="modal-title">Access: {subject.name}</h2>

        <div className="share-body">
          <div className="field-group">
            <p className="field-group-title">Who can access this</p>
            <div role="radiogroup" aria-label="Access tier" className="access-tiers">
              {tiers.map(v => (
                <label key={v} className="access-tier">
                  <input
                    type="radio"
                    name="access-tier"
                    checked={tier === v}
                    disabled={busy}
                    onChange={() => setTier(v)}
                  />
                  <span className="access-tier-text">
                    <span className="access-tier-label">{accessLabel(v)}</span>
                    <span className="access-tier-desc">{accessDescription(v)}</span>
                  </span>
                </label>
              ))}
            </div>
          </div>

          {tier === 'shared' && (
            <>
              <div className="field-group">
                <p className="field-group-title">Add User</p>
                <input
                  type="text"
                  className="input"
                  aria-label="Add user"
                  // This is a search field, not a credential entry — stop mobile
                  // OS / password managers offering to fill or save a login here.
                  name="share-user-search"
                  autoComplete="off"
                  autoCorrect="off"
                  autoCapitalize="none"
                  spellCheck={false}
                  value={query}
                  onChange={e => setQuery(e.target.value)}
                  placeholder="Start typing a username…"
                />
                {pickable.length > 0 && (
                  <ul className="share-picker-results">
                    {pickable.map(u => (
                      <li key={u.id}>
                        <button type="button" className="btn-secondary" disabled={busy} onClick={() => stageAdd(u)} aria-label={`Add ${u.username}`}>
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

              <div className="field-group">
                <p className="field-group-title">Shared with</p>
                {isLoading && <p aria-label="Loading">Loading…</p>}
                {!isLoading && staged.length === 0 && <p>No one added yet.</p>}
                {staged.length > 0 && (
                  <ul className="share-grantees">
                    {staged.map(g => (
                      <li key={g.id}>
                        <span className="share-grantee-user">
                          <Avatar userId={g.id} avatarUpdatedAt={g.avatar_updated_at} size={24} />
                          <span>{g.username}</span>
                        </span>
                        <button type="button" className="btn-icon" disabled={busy} onClick={() => stageRemove(g.id)} aria-label={`Remove ${g.username}`}>
                          <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </>
          )}

          {error && <p role="alert" className="error-msg">{error}</p>}
        </div>

        <div className="modal-actions-row" style={{ marginTop: '1.25rem' }}>
          <button type="button" className="btn-gray" onClick={onClose} disabled={busy}>Cancel</button>
          <button type="button" className="btn-primary" onClick={() => void handleSave()} disabled={!dirty || busy || isLoading}>Save</button>
        </div>
      </div>
    </div>
  )
}
