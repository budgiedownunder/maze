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
  icon?: string
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
            {a.icon && <img className="play3d-card-action-icon" src={a.icon} alt="" aria-hidden="true" />}
            {a.label}
          </button>
        ))}
      </div>
    </li>
  )
}

interface BodyProps<T> {
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
  // Bumping this resets + reloads the list — driven by the page's Refresh
  // button. A page hosting several bodies (one per tab) passes the same token to
  // each so one Refresh refreshes whichever is showing.
  refreshToken: number
}

// The list body of a Play-3D browse surface: an optional filter box, the
// responsive card grid, a Load more button, and loading/empty/error states,
// driven by `usePagedList`. Rendered by `Play3dListPage` (single list) and by
// the tabbed scope pages (one body per tab).
export function Play3dListBody<T>({
  fetchPage, getId, card, searchText, searchPlaceholder = 'Filter…', emptyText, errorText, refreshToken,
}: BodyProps<T>) {
  const token = useToken()
  const [query, setQuery] = useState('')

  const key = token ? `${token}:${refreshToken}` : null
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

  return (
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
  )
}

interface Props<T> extends Omit<BodyProps<T>, 'refreshToken'> {
  title: string
  // The parent's modals/dialogs (e.g. a leaderboard), rendered in the page shell.
  overlays?: ReactNode
}

// The shared shell for a single-list Play-3D browse page (Featured): the page
// chrome (header + Refresh) around one `Play3dListBody`. The tabbed scope pages
// (My Games / Shared with me) build their own shell with a tab strip and one
// body per tab. The play-side analogue of `WorkshopListPage`, rendering cards
// rather than rows.
export function Play3dListPage<T>({ title, overlays, ...body }: Props<T>) {
  const [refreshCount, setRefreshCount] = useState(0)
  return (
    <div className="games-page">
      {overlays}
      <AppHeader title={title}>
        <button className="btn-icon" onClick={() => setRefreshCount(c => c + 1)} aria-label="Refresh" title="Refresh">
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <Play3dListBody<T> {...body} refreshToken={refreshCount} />
    </div>
  )
}
