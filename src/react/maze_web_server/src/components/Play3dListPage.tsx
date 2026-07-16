import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { AppHeader } from './AppHeader'
import { useToken } from '../context/AuthContext'
import { usePagedList } from '../hooks/usePagedList'

const DEBOUNCE_MS = 300

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

// How a list's search box narrows results.
//   * `client` — filter the pages already loaded by `text`. Fine for a bounded
//     scope whose endpoint takes no `q`.
//   * `server` — send the (debounced) query to `fetchPage` and reload from the
//     first page. Required for an unbounded scope like Community, where a match
//     may simply not be in the pages loaded so far.
export type Play3dListSearch<T> =
  | { mode: 'client'; text: (item: T) => string }
  | { mode: 'server' }

interface BodyProps<T> {
  // Fetches one page of the scope's items. Paged by `usePagedList`; `hasMore`
  // drives the Load more button. `query` is the debounced search text, and is
  // empty unless the search is in `server` mode.
  fetchPage: (token: string, limit: number, offset: number, query: string) => Promise<{ items: T[]; hasMore: boolean }>
  getId: (item: T) => string
  // Maps an item to its card descriptor (name/description/thumbnail/actions).
  card: (item: T) => Play3dCard
  // Omit for a list with no search box.
  search?: Play3dListSearch<T>
  searchPlaceholder?: string
  emptyText: string
  errorText: string
  // Identifies the current query: the parent folds in whatever it owns (its
  // Refresh counter, the chosen sort, …), and any change resets + reloads the
  // list. A page hosting several bodies (one per tab) passes the same token to
  // each, so one Refresh refreshes whichever is showing.
  queryToken: string | number
}

// The list body of a Play-3D browse surface: an optional filter box, the
// responsive card grid, a Load more button, and loading/empty/error states,
// driven by `usePagedList`. Rendered by `Play3dListPage` (single list) and by
// the tabbed scope pages (one body per tab).
export function Play3dListBody<T>({
  fetchPage, getId, card, search, searchPlaceholder = 'Filter…', emptyText, errorText, queryToken,
}: BodyProps<T>) {
  const token = useToken()
  const [query, setQuery] = useState('')
  const [debounced, setDebounced] = useState('')

  // Debounce only matters for a server-searched list (it refetches per query);
  // a client filter reads `query` directly, so it stays instant.
  useEffect(() => {
    const handle = setTimeout(() => setDebounced(query), DEBOUNCE_MS)
    return () => clearTimeout(handle)
  }, [query])

  // A server-searched list refetches on the query, so it belongs in the key.
  const serverQuery = search?.mode === 'server' ? debounced.trim() : ''
  const key = token ? `${token}:${queryToken}:${serverQuery}` : null
  const doFetch = useCallback(
    (limit: number, offset: number) => fetchPage(token!, limit, offset, serverQuery),
    [token, fetchPage, serverQuery],
  )
  const list = usePagedList<T>(key, doFetch, getId, errorText)

  // A client filter narrows the accumulated pages for display; Load more /
  // `hasMore` stay driven by the unfiltered server page. A server-searched list
  // already came back filtered.
  const trimmed = query.trim()
  const items = search?.mode === 'client' && trimmed !== ''
    ? list.items.filter(i => search.text(i).toLowerCase().includes(trimmed.toLowerCase()))
    : list.items

  return (
    <main className="maze-list-page">
      {search && (
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

interface Props<T> extends Omit<BodyProps<T>, 'queryToken'> {
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
      <Play3dListBody<T> {...body} queryToken={refreshCount} />
    </div>
  )
}
