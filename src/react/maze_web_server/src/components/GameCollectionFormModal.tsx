import { useEffect, useRef, useState } from 'react'
import { useToken } from '../context/AuthContext'
import { WorkshopThumbnail } from './WorkshopListPage'
import { GameImageEditor } from './GameImageEditor'
import { getGameCollection, listGameDefinitions } from '../api/client'
import type { GameDefinition, PlayMode } from '../types/api'
import { PLAY_MODES, playModeDescription, playModeLabel } from '../utils/gameDefinitions'

// The Add picker loads the owner's whole game set once (in the background, paged)
// on open, then filters + excludes already-added members entirely in memory — so
// searching never hits the server after init. It pages until the server signals
// no more (rather than assuming the 500-per-user cap), so it stays correct if
// that cap ever changes; games are fetched with excludeDefinitions so we don't
// pull their heavy config blobs.
const PICK_PAGE_SIZE = 100

interface Props {
  title: string
  confirmLabel: string
  initialName?: string
  initialDescription?: string
  initialPlayMode?: PlayMode
  // Edit mode: the collection whose membership this modal also manages. Omitted
  // for Create (a new collection has no id to attach members to yet).
  collectionId?: string
  // Edit mode: the collection's current image marker + a change reporter, so the
  // modal can show the image control (mirrors the definition editor). Omitted for
  // Create.
  imageUpdatedAt?: string | null
  onImageChange?: (imageUpdatedAt: string | null) => void
  isLoading?: boolean
  error?: string | null
  // `memberIds` is supplied (Edit only) when the membership changed — the parent
  // then reconciles it in one call alongside the metadata update.
  onSubmit: (name: string, description: string | null, playMode: PlayMode, memberIds?: string[]) => void
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
  initialPlayMode = 'arcade',
  collectionId,
  imageUpdatedAt,
  onImageChange,
  isLoading = false,
  error,
  onSubmit,
  onCancel,
}: Props) {
  const token = useToken()
  const [name, setName] = useState(initialName)
  const [description, setDescription] = useState(initialDescription)
  const [playMode, setPlayMode] = useState<PlayMode>(initialPlayMode)
  const [validationError, setValidationError] = useState<string | null>(null)

  // Membership (Edit mode only). `members` is null while loading; `ownerGames`
  // holds the caller's whole game set (null until the background load finishes),
  // which the Add picker filters in memory.
  const [members, setMembers] = useState<GameDefinition[] | null>(null)
  const [originalIds, setOriginalIds] = useState<string[]>([])
  const [ownerGames, setOwnerGames] = useState<GameDefinition[] | null>(null)
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

  // Background load of the owner's whole game set (paged, light — no config), so
  // the Add picker searches + excludes members in memory. Pages until the server
  // signals no more; an empty page also stops it (a defensive guard so a
  // misbehaving `hasMore` can't spin). On error the picker shows whatever loaded.
  useEffect(() => {
    if (!token || !collectionId) return
    let cancelled = false
    void (async () => {
      const all: GameDefinition[] = []
      try {
        for (let offset = 0; ; offset += PICK_PAGE_SIZE) {
          const res = await listGameDefinitions(token, { scope: 'mine', excludeDefinitions: true, limit: PICK_PAGE_SIZE, offset })
          all.push(...res.definitions)
          if (!res.hasMore || res.definitions.length === 0) break
        }
      } catch { /* leave `all` as whatever loaded; picker shows those */ }
      if (!cancelled) setOwnerGames(all)
    })()
    return () => { cancelled = true }
  }, [token, collectionId])

  const memberIds = members?.map(m => m.id) ?? []
  const membersDirty = members != null
    && (memberIds.length !== originalIds.length || memberIds.some((id, i) => id !== originalIds[i]))
  const metaDirty = name !== initialName || description !== initialDescription || playMode !== initialPlayMode
  // Edit mode with no metadata/membership change → nothing to save. Hide Save and
  // relabel Cancel "Close" (the image is a separate, immediately-saved resource,
  // so it never counts as an unsaved change). Create always offers its button.
  const nothingToSave = collectionId != null && !metaDirty && !membersDirty

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

  // Membership editing is enabled only once the owner's game set has finished
  // loading, so the picker (and the add/remove/reorder buttons) act on the
  // complete list.
  const gamesLoaded = ownerGames != null
  const stagedIds = new Set(memberIds)
  const trimmedQuery = query.trim().toLowerCase()
  // Available = the owner's games not already staged, filtered by the search box
  // — all in memory (no server request after the initial load).
  const pickable = (ownerGames ?? [])
    .filter(g => !stagedIds.has(g.id))
    .filter(g => trimmedQuery === '' || g.name.toLowerCase().includes(trimmedQuery))

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const trimmedName = name.trim()
    if (!trimmedName) {
      setValidationError('Name cannot be empty.')
      return
    }
    setValidationError(null)
    const trimmedDescription = description.trim()
    onSubmit(trimmedName, trimmedDescription === '' ? null : trimmedDescription, playMode, membersDirty ? memberIds : undefined)
  }

  const displayError = validationError ?? error

  return (
    <div role="dialog" aria-modal="true" aria-label={title} className="modal-overlay" style={{ zIndex: 1200, cursor: isLoading ? 'wait' : undefined }}>
      <div className="modal collection-form-modal modal-capped">
        <h2 className="modal-title">{title}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <div className="collection-form-body">
          {collectionId && onImageChange && (
            <GameImageEditor
              kind="collection"
              id={collectionId}
              imageUpdatedAt={imageUpdatedAt}
              onChange={onImageChange}
            />
          )}
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
          <label>
            Play mode
            <select
              className="input"
              value={playMode}
              onChange={e => setPlayMode(e.target.value as PlayMode)}
            >
              {PLAY_MODES.map(mode => (
                <option key={mode} value={mode}>{playModeLabel(mode)}</option>
              ))}
            </select>
          </label>
          <p className="access-tier-desc">{playModeDescription(playMode)}</p>

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
                      <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={m.visibility} showMarker={false} imageSubject={{ kind: 'definition', id: m.id, imageUpdatedAt: m.imageUpdatedAt }} />
                      <span className="collection-member-name" title={m.name}>{m.name}</span>
                      <div className="collection-member-actions">
                        <button type="button" className="btn-icon" aria-label={`Move ${m.name} up`} disabled={!gamesLoaded || i === 0} onClick={e => { e.stopPropagation(); setSelectedId(m.id); move(i, -1) }}>
                          <img src="/images/icons/icon_move_up.svg" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                        <button type="button" className="btn-icon" aria-label={`Move ${m.name} down`} disabled={!gamesLoaded || i === members.length - 1} onClick={e => { e.stopPropagation(); setSelectedId(m.id); move(i, 1) }}>
                          <img src="/images/icons/icon_move_down.svg" alt="" aria-hidden="true" width={18} height={18} />
                        </button>
                        <button type="button" className="btn-icon" aria-label={`Remove ${m.name}`} disabled={!gamesLoaded} onClick={e => { e.stopPropagation(); stageRemove(m.id) }}>
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
                  {!gamesLoaded && <p aria-label="Loading available games">Loading…</p>}
                  {gamesLoaded && (
                    <>
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
                              <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={g.visibility} showMarker={false} imageSubject={{ kind: 'definition', id: g.id, imageUpdatedAt: g.imageUpdatedAt }} />
                              <span className="collection-member-name" title={g.name}>{g.name}</span>
                              <div className="collection-member-actions">
                                <button type="button" className="btn-icon collection-add-btn" onClick={e => { e.stopPropagation(); stageAdd(g) }} aria-label={`Add ${g.name}`}>+</button>
                              </div>
                            </li>
                          ))}
                        </ul>
                      )}
                    </>
                  )}
                </>
              )}
            </div>
          )}
          </div>

          {displayError && <p role="alert" className="error-msg">{displayError}</p>}
          <div className="modal-actions-row">
            <button type="button" onClick={onCancel} className="btn-gray" disabled={isLoading}>{nothingToSave ? 'Close' : 'Cancel'}</button>
            {!nothingToSave && <button type="submit" className="btn-primary" disabled={isLoading}>{confirmLabel}</button>}
          </div>
        </form>
      </div>
    </div>
  )
}
