import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { AppHeader } from './AppHeader'
import { useToken } from '../context/AuthContext'
import { usePagedList } from '../hooks/usePagedList'
import { getGameDefinitionImageObjectUrl, getGameCollectionImageObjectUrl } from '../utils/imageCache'
import { accessDescription, playModeLabel, type PlayMode, type Visibility } from '../utils/gameDefinitions'

// The entity whose uploaded image (when present) replaces the base placeholder
// art. `imageUpdatedAt` is both the has-image gate and the cache-buster; absent
// ⇒ the placeholder shows and no request is made.
export interface ThumbnailImageSubject {
  kind: 'definition' | 'collection'
  id: string
  imageUpdatedAt?: string | null
}

// Resolves a subject's uploaded image to a shared object URL (or null) via the
// guarded-image cache, keyed by kind+id+marker so a re-used row instance never
// shows a stale image after its subject changes.
function useThumbnailImage(subject?: ThumbnailImageSubject): string | null {
  const token = useToken()
  const marker = subject?.imageUpdatedAt ?? null
  const key = subject && marker ? `${subject.kind}:${subject.id}:${marker}` : ''
  const [loaded, setLoaded] = useState<{ key: string; url: string | null }>({ key: '', url: null })

  useEffect(() => {
    if (!key || !token || !subject || !marker) return
    let cancelled = false
    const resolve = subject.kind === 'definition' ? getGameDefinitionImageObjectUrl : getGameCollectionImageObjectUrl
    resolve(token, subject.id, marker).then(url => { if (!cancelled) setLoaded({ key, url }) })
    return () => { cancelled = true }
  }, [key, token, subject, marker])

  return loaded.key === key ? loaded.url : null
}

// A workshop row thumbnail: the base art with the visibility marker overhanging
// its bottom-right corner, titled with the access description. Shared by the
// Games and Collections rows (each supplies its own base art). `showMarker` is
// dropped in contexts where a game's own visibility is irrelevant (the collection
// membership editor, where the collection carries the visibility). `playMode`,
// when given (collection rows), adds the matching badge on the bottom-left corner.
// `imageSubject`, when given, swaps the placeholder base art for the entity's own
// uploaded image once it has one.
export function WorkshopThumbnail({ baseSrc, visibility, showMarker = true, playMode, imageSubject }: { baseSrc: string; visibility: Visibility; showMarker?: boolean; playMode?: PlayMode; imageSubject?: ThumbnailImageSubject }) {
  const imageUrl = useThumbnailImage(imageSubject)
  return (
    <div className="game-thumb" title={showMarker ? accessDescription(visibility) : undefined}>
      <img className="game-thumb-base" src={imageUrl ?? baseSrc} alt="" aria-hidden="true" />
      {showMarker && <img className="game-thumb-marker" src={`/images/workshop/marker-${visibility}.svg`} alt="" aria-hidden="true" />}
      {playMode && <img className="game-thumb-mode" src={`/images/workshop/mode-${playMode}.svg`} alt="" aria-hidden="true" title={playModeLabel(playMode)} />}
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
  // Fetches one page of the list. The parent's `filter` is applied at render
  // time over the accumulated pages, so a derived change re-filters without a
  // refetch; `hasMore` drives the Load more button.
  fetchPage: (token: string, limit: number, offset: number) => Promise<{ items: T[]; hasMore: boolean }>
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
// pages: it owns the keyed paged-load/refresh state machine (via `usePagedList`)
// and the page chrome (header with a New button + Refresh, then the loading /
// error / empty / list / Load more scaffold). Each page supplies its own rows
// (`row`) and modals (`overlays`); the divergent action sets stay in the parent.
export function WorkshopListPage<T>({
  title, newLabel, onNew, fetchPage, filter, getId, emptyText, errorText, row, overlays, banner, onReady,
}: Props<T>) {
  const token = useToken()
  const [refreshCount, setRefreshCount] = useState(0)

  // Key the paged list by token + refresh counter, so a refresh (or sign-in)
  // resets the view by derivation. `fetchPage`'s identity may churn; the hook
  // reads it through a ref and only the key drives reloads.
  const key = token ? `${token}:${refreshCount}` : null
  const doFetch = useCallback(
    (limit: number, offset: number) => fetchPage(token!, limit, offset),
    [token, fetchPage],
  )
  const list = usePagedList<T>(key, doFetch, getId, errorText)

  // `filter` narrows the accumulated pages for display (e.g. Manage Games hides
  // curated); Load more / `hasMore` stay driven by the unfiltered server page.
  const items = filter ? list.items.filter(filter) : list.items

  // Live accessor for the parent's handlers.
  const itemsRef = useRef<T[]>(items)
  useEffect(() => { itemsRef.current = items })
  const getItems = useCallback(() => itemsRef.current, [])

  const refresh = useCallback(() => setRefreshCount(c => c + 1), [])
  const ctx = useMemo<WorkshopListContext<T>>(
    () => ({ refresh, patchItem: list.patchItem, getItems }),
    [refresh, list.patchItem, getItems],
  )
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
        {list.isLoading && <p aria-label="Loading">Loading…</p>}
        {!list.isLoading && list.error && <p className="error-msg" role="alert">{list.error}</p>}
        {!list.isLoading && !list.error && items.length === 0 && <p>{emptyText}</p>}
        {!list.isLoading && !list.error && items.length > 0 && (
          <>
            <ul className="game-list">
              {items.map(item => <WorkshopListRow key={getId(item)} {...row(item)} />)}
            </ul>
            {list.hasMore && (
              <button
                type="button"
                className="btn-secondary workshop-load-more"
                onClick={list.loadMore}
                disabled={list.isLoadingMore}
              >
                {list.isLoadingMore ? 'Loading…' : 'Load more'}
              </button>
            )}
          </>
        )}
      </main>
    </div>
  )
}
