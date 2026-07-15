import { useCallback, useState, type ReactNode } from 'react'
import { AppHeader } from './AppHeader'
import { useToken } from '../context/AuthContext'
import { usePagedList } from '../hooks/usePagedList'

// One action button on a browse card (e.g. Play, Leaderboard). `primary` renders
// the accented button; anything else is a secondary button. `disabled` + `title`
// support the "coming soon" state (e.g. a multi-game collection whose picker
// isn't built yet).
export interface Play3dCardAction {
  key: string
  label: string
  ariaLabel: string
  onClick: () => void
  variant?: 'primary' | 'secondary'
  disabled?: boolean
  title?: string
}

// A declarative description of one browse card: the base renders the shared
// `.play3d-card` shell (thumbnail + name/description + action buttons) from it.
export interface Play3dCard {
  name: string
  description?: string
  // Optional card thumbnail (e.g. the game/collection art + badges).
  thumbnail?: ReactNode
  actions: Play3dCardAction[]
}

// The shared vertical gallery card, driven entirely by a `Play3dCard` descriptor.
function Play3dCardView({ name, description, thumbnail, actions }: Play3dCard) {
  return (
    <li className="play3d-card">
      {thumbnail && <div className="play3d-card-thumb">{thumbnail}</div>}
      <div className="play3d-card-body">
        <span className="play3d-card-name" title={name}>{name}</span>
        {description && <p className="play3d-card-desc">{description}</p>}
      </div>
      <div className="play3d-card-actions">
        {actions.map(a => (
          <button
            key={a.key}
            type="button"
            className={a.variant === 'primary' ? 'btn-primary' : 'btn-secondary'}
            onClick={a.onClick}
            aria-label={a.ariaLabel}
            disabled={a.disabled}
            title={a.title}
          >
            {a.label}
          </button>
        ))}
      </div>
    </li>
  )
}

interface Props<T> {
  title: string
  // Fetches one page of the scope's items. Paged by `usePagedList`; `hasMore`
  // drives the Load more button.
  fetchPage: (token: string, limit: number, offset: number) => Promise<{ items: T[]; hasMore: boolean }>
  getId: (item: T) => string
  // Maps an item to its card descriptor (name/description/thumbnail/actions).
  card: (item: T) => Play3dCard
  // When provided, renders a filter box that narrows the accumulated pages
  // client-side by this text; omit it for scopes with no filter. (Server-side
  // search — the Community scope — is layered on when that page is built.)
  searchText?: (item: T) => string
  searchPlaceholder?: string
  emptyText: string
  errorText: string
  // The parent's modals/dialogs (e.g. a leaderboard), rendered in the page shell.
  overlays?: ReactNode
}

// The shared shell for the Play-3D browse pages (Featured, and later My Games /
// Shared with me / Community): it owns the keyed paged-load state machine (via
// `usePagedList`) and the page chrome (header + Refresh, then the loading /
// error / empty / card-grid / Load more scaffold + an optional client-side
// filter). Each page supplies its own `fetchPage` + `card` mapping and any
// `overlays`. The play-side analogue of `WorkshopListPage`, rendering cards
// rather than rows.
export function Play3dListPage<T>({
  title, fetchPage, getId, card, searchText, searchPlaceholder = 'Filter…', emptyText, errorText, overlays,
}: Props<T>) {
  const token = useToken()
  const [refreshCount, setRefreshCount] = useState(0)
  const [query, setQuery] = useState('')

  const key = token ? `${token}:${refreshCount}` : null
  const doFetch = useCallback(
    (limit: number, offset: number) => fetchPage(token!, limit, offset),
    [token, fetchPage],
  )
  const list = usePagedList<T>(key, doFetch, getId, errorText)

  // The filter narrows the accumulated pages for display; Load more / `hasMore`
  // stay driven by the unfiltered server page.
  const trimmed = query.trim().toLowerCase()
  const items = searchText && trimmed !== ''
    ? list.items.filter(i => searchText(i).toLowerCase().includes(trimmed))
    : list.items

  const refresh = () => setRefreshCount(c => c + 1)

  return (
    <div className="games-page">
      {overlays}
      <AppHeader title={title}>
        <button className="btn-icon" onClick={refresh} aria-label="Refresh" title="Refresh">
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <main className="maze-list-page">
        {searchText && (
          <input
            type="text"
            className="input play3d-search"
            aria-label={searchPlaceholder}
            placeholder={searchPlaceholder}
            value={query}
            onChange={e => setQuery(e.target.value)}
          />
        )}
        {list.isLoading && <p aria-label="Loading">Loading&hellip;</p>}
        {!list.isLoading && list.error && <p className="error-msg" role="alert">{list.error}</p>}
        {!list.isLoading && !list.error && items.length === 0 && (
          <p>{trimmed !== '' ? 'No matches.' : emptyText}</p>
        )}
        {!list.isLoading && !list.error && items.length > 0 && (
          <>
            <ul className="play3d-grid">
              {items.map(item => <Play3dCardView key={getId(item)} {...card(item)} />)}
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
