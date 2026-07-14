import { useEffect, useRef, useState } from 'react'
import { useToken } from '../context/AuthContext'
import { WorkshopThumbnail } from './WorkshopListPage'
import { getGameCollection, listGameDefinitions } from '../api/client'
import type { GameDefinition } from '../types/api'

// The Add picker searches the owner's games server-side, debounced. It fetches
// up to the server's max page so its (scrollable) list holds all the owner's
// available games — as members are added they drop out and the next available
// ones stay visible; the "keep typing to narrow" hint appears only when there
// are more matches than that (i.e. a broad query worth narrowing).
const PICK_LIMIT = 100
const PICK_DEBOUNCE_MS = 250

interface Props {
  title: string
  confirmLabel: string
  initialName?: string
  initialDescription?: string
  // Edit mode: the collection whose membership this modal also manages. Omitted
  // for Create (a new collection has no id to attach members to yet).
  collectionId?: string
  isLoading?: boolean
  error?: string | null
  // `memberIds` is supplied (Edit only) when the membership changed — the parent
  // then reconciles it in one call alongside the metadata update.
  onSubmit: (name: string, description: string | null, memberIds?: string[]) => void
  onCancel: () => void
}

// A game collection's metadata (name + optional description) plus, in Edit mode,
// its membership — the games it contains, with add / remove / reorder staged
// locally and committed together on Save (mirroring the Access modal). Create
// mode is metadata-only.
export function GameCollectionFormModal({
  title,
  confirmLabel,
  initialName = '',
  initialDescription = '',
  collectionId,
  isLoading = false,
  error,
  onSubmit,
  onCancel,
}: Props) {
  const token = useToken()
  const [name, setName] = useState(initialName)
  const [description, setDescription] = useState(initialDescription)
  const [validationError, setValidationError] = useState<string | null>(null)

  // Membership (Edit mode only). `members` is null while loading; the picker
  // searches the owner's games server-side into `pickerResults`.
  const [members, setMembers] = useState<GameDefinition[] | null>(null)
  const [originalIds, setOriginalIds] = useState<string[]>([])
  const [pickerResults, setPickerResults] = useState<GameDefinition[]>([])
  const [pickerHasMore, setPickerHasMore] = useState(false)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [query, setQuery] = useState('')
  // The clicked member row; the highlight follows the game (by id) when moved.
  const [selectedId, setSelectedId] = useState<string | null>(null)
  // The clicked source (available-games) row — highlight only.
  const [selectedSourceId, setSelectedSourceId] = useState<string | null>(null)
  // A just-added game to scroll into view once its row renders (kept in a ref so
  // consuming it doesn't setState in the effect).
  const rowRefs = useRef(new Map<string, HTMLLIElement>())
  const pendingScrollId = useRef<string | null>(null)

  useEffect(() => {
    const id = pendingScrollId.current
    if (!id) return
    pendingScrollId.current = null
    rowRefs.current.get(id)?.scrollIntoView?.({ block: 'nearest' })
  }, [members])

  useEffect(() => {
    if (!token || !collectionId) return
    let cancelled = false
    getGameCollection(token, collectionId)
      .then(detail => {
        if (cancelled) return
        setMembers(detail.definitions)
        setOriginalIds(detail.definitions.map(d => d.id))
      })
      .catch((ex: unknown) => { if (!cancelled) setLoadError((ex as Error).message || 'Failed to load games') })
    return () => { cancelled = true }
  }, [token, collectionId])

  // Debounced server-side search of the owner's own games for the Add picker.
  useEffect(() => {
    if (!token || !collectionId) return
    let cancelled = false
    const handle = setTimeout(() => {
      listGameDefinitions(token, { scope: 'mine', q: query, limit: PICK_LIMIT })
        .then(page => { if (!cancelled) { setPickerResults(page.definitions); setPickerHasMore(page.hasMore) } })
        .catch(() => { if (!cancelled) { setPickerResults([]); setPickerHasMore(false) } })
    }, PICK_DEBOUNCE_MS)
    return () => { cancelled = true; clearTimeout(handle) }
  }, [token, collectionId, query])

  const memberIds = members?.map(m => m.id) ?? []
  const membersDirty = members != null
    && (memberIds.length !== originalIds.length || memberIds.some((id, i) => id !== originalIds[i]))
  const metaDirty = name !== initialName || description !== initialDescription
  // Create Save stays always-enabled (validates on submit); Edit gates on dirty.
  const saveDisabled = isLoading || (collectionId != null && !metaDirty && !membersDirty)

  function stageAdd(game: GameDefinition) {
    // Insert after the highlighted row (if any), else append.
    setMembers(prev => {
      if (!prev) return [game]
      const at = selectedId ? prev.findIndex(m => m.id === selectedId) : -1
      if (at === -1) return [...prev, game]
      return [...prev.slice(0, at + 1), game, ...prev.slice(at + 1)]
    })
    setSelectedId(game.id)
    setSelectedSourceId(null)
    pendingScrollId.current = game.id
    setQuery('')
  }
  function stageRemove(id: string) {
    setMembers(prev => (prev ? prev.filter(m => m.id !== id) : prev))
    if (selectedId === id) setSelectedId(null)
  }
  function move(index: number, delta: number) {
    setMembers(prev => {
      if (!prev) return prev
      const next = [...prev]
      const target = index + delta
      if (target < 0 || target >= next.length) return prev
      ;[next[index], next[target]] = [next[target], next[index]]
      return next
    })
  }

  const stagedIds = new Set(memberIds)
  // The server already applied the name filter (q); just drop already-staged games.
  const pickable = pickerResults.filter(g => !stagedIds.has(g.id))

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const trimmedName = name.trim()
    if (!trimmedName) {
      setValidationError('Name cannot be empty.')
      return
    }
    setValidationError(null)
    const trimmedDescription = description.trim()
    onSubmit(trimmedName, trimmedDescription === '' ? null : trimmedDescription, membersDirty ? memberIds : undefined)
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal collection-form-modal modal-capped">
        <h2 className="modal-title">{title}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <div className="collection-form-body">
          <label>
            Name
            <input
              type="text"
              className="input"
              value={name}
              onChange={e => { setName(e.target.value); setValidationError(null) }}
              autoFocus
            />
          </label>
          <label>
            Description (optional)
            <textarea
              className="input"
              rows={3}
              value={description}
              onChange={e => setDescription(e.target.value)}
            />
          </label>

          {collectionId != null && (
            <div className="field-group">
              <p className="field-group-title">Games in this collection</p>
              {members == null && loadError == null && <p aria-label="Loading">Loading…</p>}
              {loadError && <p role="alert" className="error-msg">{loadError}</p>}
              {members != null && members.length === 0 && <p>No games yet.</p>}
              {members != null && members.length > 0 && (
                <ul className="collection-members">
                  {members.map((m, i) => (
                    <li
                      key={m.id}
                      ref={el => { const map = rowRefs.current; if (el) map.set(m.id, el); else map.delete(m.id) }}
                      className={`collection-member${selectedId === m.id ? ' selected' : ''}`}
                      onClick={() => setSelectedId(m.id)}
                    >
                      <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={m.visibility} showMarker={false} />
                      <span className="collection-member-name" title={m.name}>{m.name}</span>
                      <div className="collection-member-actions">
                        <button type="button" className="btn-icon" aria-label={`Move ${m.name} up`} disabled={i === 0} onClick={e => { e.stopPropagation(); setSelectedId(m.id); move(i, -1) }}>
                          <img src="/images/icons/icon_move_up.svg" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                        <button type="button" className="btn-icon" aria-label={`Move ${m.name} down`} disabled={i === members.length - 1} onClick={e => { e.stopPropagation(); setSelectedId(m.id); move(i, 1) }}>
                          <img src="/images/icons/icon_move_down.svg" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                        <button type="button" className="btn-icon" aria-label={`Remove ${m.name}`} onClick={e => { e.stopPropagation(); stageRemove(m.id) }}>
                          <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
              {members != null && (
                <>
                  <p className="field-group-title">Available Games</p>
                  <input
                    type="text"
                    className="input"
                    aria-label="Add game"
                    value={query}
                    onChange={e => setQuery(e.target.value)}
                    placeholder="Add a game — type to filter…"
                  />
                  {pickable.length > 0 && (
                    <ul className="collection-member-picker">
                      {pickable.map(g => (
                        <li
                          key={g.id}
                          className={`collection-picker-item${selectedSourceId === g.id ? ' selected' : ''}`}
                          onClick={() => setSelectedSourceId(g.id)}
                        >
                          <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={g.visibility} showMarker={false} />
                          <span className="collection-member-name" title={g.name}>{g.name}</span>
                          <div className="collection-member-actions">
                            <button type="button" className="btn-icon collection-add-btn" onClick={e => { e.stopPropagation(); stageAdd(g) }} aria-label={`Add ${g.name}`}>+</button>
                          </div>
                        </li>
                      ))}
                    </ul>
                  )}
                  {pickerHasMore && (
                    <p className="share-picker-hint">More matches — keep typing to narrow.</p>
                  )}
                </>
              )}
            </div>
          )}
          </div>

          {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray" disabled={isLoading}>Cancel</button>
            <button type="submit" className="btn-primary" disabled={saveDisabled}>{confirmLabel}</button>
          </div>
        </form>
      </div>
    </div>
  )
}
