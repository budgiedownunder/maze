import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { AppHeader } from './AppHeader'
import { useToken } from '../context/AuthContext'
import { accessDescription, type Visibility } from '../utils/gameDefinitions'

// A workshop row thumbnail: the base art with the visibility marker overhanging
// its corner, titled with the access description. Shared by the Games and
// Collections rows (each supplies its own base art). `showMarker` is dropped in
// contexts where a game's own visibility is irrelevant (the collection
// membership editor, where the collection carries the visibility).
export function WorkshopThumbnail({ baseSrc, visibility, showMarker = true }: { baseSrc: string; visibility: Visibility; showMarker?: boolean }) {
  return (
    <div className="game-thumb" title={showMarker ? accessDescription(visibility) : undefined}>
      <img className="game-thumb-base" src={baseSrc} alt="" aria-hidden="true" />
      {showMarker && <img className="game-thumb-marker" src={`/images/workshop/marker-${visibility}.svg`} alt="" aria-hidden="true" />}
    </div>
  )
}

// The context a workshop list page hands back to its parent so the parent's
// create / edit / delete / access handlers can drive the shared list state.
export interface WorkshopListContext<T> {
  // Re-fetch the list from the server.
  refresh: () => void
  // Patch a single already-loaded row in place (by id) — e.g. after an access
  // save updates one row's tier, without a full refetch/flash.
  patchItem: (id: string, patch: Partial<T>) => void
  // Live accessor for the current filtered list (reads the latest render).
  getItems: () => T[]
}

// One button in a row's action cluster. `variant` picks the danger styling for a
// destructive action; everything else is a secondary button.
export interface RowAction {
  key: string
  label: string
  ariaLabel: string
  icon: string
  onClick: () => void
  variant?: 'secondary' | 'danger'
}

// A declarative description of one list row: the base renders the shared
// `.game-list-item` shell (thumbnail + name/subtitle + action buttons) from it.
export interface WorkshopRow {
  name: string
  subtitle: string
  // Optional row thumbnail (e.g. the game/collection art + visibility marker).
  thumbnail?: ReactNode
  // Mouse-click-on-row convenience (usually the same as the Edit action); the
  // action buttons stop propagation so they never also trigger it.
  onOpen?: () => void
  actions: RowAction[]
}

// The shared row: the `.game-list-item` structure both workshop lists use, driven
// entirely by a `WorkshopRow` descriptor.
function WorkshopListRow({ name, subtitle, thumbnail, onOpen, actions }: WorkshopRow) {
  return (
    <li
      className="game-list-item"
      onClick={onOpen}
      style={onOpen ? undefined : { cursor: 'default' }}
    >
      {thumbnail}
      <div className="maze-item-text">
        <span className="maze-item-name" title={name}>{name}</span>
        <span className="maze-item-subtitle">{subtitle}</span>
      </div>
      <div className="game-item-actions">
        {actions.map(a => (
          <button
            key={a.key}
            type="button"
            className={`maze-item-action ${a.variant === 'danger' ? 'btn-danger-outline' : 'btn-secondary'}`}
            onClick={e => { e.stopPropagation(); a.onClick() }}
            aria-label={a.ariaLabel}
          >
            <img src={a.icon} alt="" aria-hidden="true" />
            <span className="maze-item-action-label">{a.label}</span>
          </button>
        ))}
      </div>
    </li>
  )
}

interface Props<T> {
  title: string
  newLabel: string
  onNew: () => void
  // Fetches the raw list; the parent's `filter` is applied at render time so a
  // profile / derived change re-filters without a refetch.
  load: (token: string) => Promise<T[]>
  filter?: (item: T) => boolean
  getId: (item: T) => string
  emptyText: string
  errorText: string
  // Maps an item to its row descriptor (name/subtitle/thumbnail/actions).
  row: (item: T) => WorkshopRow
  // The parent's modals/dialogs, rendered at the top of the page container.
  overlays?: ReactNode
  // Optional banner shown above the list (e.g. an action error).
  banner?: ReactNode
  // Receives the list context (stable) so the parent's handlers can refresh/patch.
  onReady: (ctx: WorkshopListContext<T>) => void
}

// The shared shell for the workshop's Manage Games / Manage Game Collections
// pages: it owns the keyed load/refresh state machine and the page chrome
// (header with a New button + Refresh, then the loading / error / empty / list
// scaffold). Each page supplies its own rows (`renderItem`) and modals
// (`overlays`); the divergent action sets stay in the parent.
export function WorkshopListPage<T>({
  title, newLabel, onNew, load, filter, getId, emptyText, errorText, row, overlays, banner, onReady,
}: Props<T>) {
  const token = useToken()

  // Load state keyed by the refresh counter, so a refresh resets the view by
  // derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; items: T[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const error = errorFor != null && errorFor.key === refreshCount ? errorFor.message : null
  const current = loaded != null && loaded.key === refreshCount ? loaded : null
  const rawItems = current?.items ?? []
  const items = filter ? rawItems.filter(filter) : rawItems
  const isLoading = current == null && error == null

  // The latest `load` / `getId` closures and derived `items` are kept in refs so
  // the fetch effect and the stable context callbacks can read current values —
  // without `load` identity re-triggering the fetch, and without accessing refs
  // during render. Synced after each commit; the fetch re-runs only on token /
  // refresh change, matching the pre-refactor pages.
  const loadRef = useRef(load)
  const getIdRef = useRef(getId)
  const itemsRef = useRef<T[]>(items)
  useEffect(() => {
    loadRef.current = load
    getIdRef.current = getId
    itemsRef.current = items
  })

  useEffect(() => {
    if (!token) return
    let cancelled = false
    const key = refreshCount
    loadRef.current(token)
      .then(fetched => { if (!cancelled) setLoaded({ key, items: fetched }) })
      .catch(ex => { if (!cancelled) setErrorFor({ key, message: (ex as Error).message || errorText }) })
    return () => { cancelled = true }
  }, [token, refreshCount, errorText])

  const refresh = useCallback(() => setRefreshCount(c => c + 1), [])
  const patchItem = useCallback((id: string, patch: Partial<T>) => {
    setLoaded(prev =>
      prev == null
        ? prev
        : { ...prev, items: prev.items.map(it => (getIdRef.current(it) === id ? { ...it, ...patch } : it)) })
  }, [])
  const getItems = useCallback(() => itemsRef.current, [])

  const ctx = useMemo<WorkshopListContext<T>>(() => ({ refresh, patchItem, getItems }), [refresh, patchItem, getItems])
  useEffect(() => { onReady(ctx) }, [ctx, onReady])

  return (
    <div className="games-page">
      {overlays}
      <AppHeader title={title}>
        <button type="button" className="btn-primary" onClick={onNew}>{newLabel}</button>
        <button
          className="btn-icon"
          onClick={refresh}
          aria-label="Refresh"
          title="Refresh"
        >
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <main className="maze-list-page">
        {banner}
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && <p className="error-msg" role="alert">{error}</p>}
        {!isLoading && !error && items.length === 0 && <p>{emptyText}</p>}
        {!isLoading && !error && items.length > 0 && (
          <ul className="game-list">
            {items.map(item => <WorkshopListRow key={getId(item)} {...row(item)} />)}
          </ul>
        )}
      </main>
    </div>
  )
}
