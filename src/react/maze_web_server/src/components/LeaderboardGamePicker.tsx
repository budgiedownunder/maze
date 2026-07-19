import { useCallback, useEffect, useState } from 'react'
import { usePagedList } from '../hooks/usePagedList'
import { getFeaturedGameItems, getGameCollection, listGameCollections, listGameDefinitions } from '../api/client'
import { useToken } from '../context/AuthContext'
import type { FeaturedGameItem, GameCollection, GameDefinition } from '../types/api'
import type { Rotation } from '../utils/gameDefinitions'

const DEBOUNCE_MS = 300

// The browsable scopes, in tab order. `featured` reads the admin-ordered
// catalogue (one already-merged list of games + collections); the other three
// read the game-definition / game-collection list endpoints at that scope.
const SCOPES = [
  { key: 'featured', label: 'Featured' },
  { key: 'mine', label: 'My Games' },
  { key: 'shared', label: 'Shared with me' },
  { key: 'public', label: 'Community' },
] as const
type Scope = (typeof SCOPES)[number]['key']

// `mine` and `public` filter server-side (`q`) — `public` is the unbounded pool,
// so its search must reach the database. `featured` and `shared` are bounded, so
// they filter the accumulated pages client-side (their endpoints take no `q`).
function filtersOnServer(scope: Scope): boolean {
  return scope === 'mine' || scope === 'public'
}

// The game a leaderboard is shown for. `ownerId` lets the host decide whether the
// caller may reset that board (owner or admin).
export interface PickedGame {
  id: string
  name: string
  ownerId: string
  // Static → one fixed `def:<id>` board; Daily → a per-UTC-day
  // `def:<id>:<date>` board (the leaderboard page adds a date control for it).
  rotation: Rotation
}

interface Props {
  value: PickedGame | null
  onSelect: (game: PickedGame) => void
}

function featuredItemKey(item: FeaturedGameItem): string {
  const id = item.kind === 'definition' ? item.definition!.id : item.collection!.id
  return `${item.kind}:${id}`
}

function featuredItemName(item: FeaturedGameItem): string {
  return item.kind === 'definition' ? item.definition!.name : item.collection!.name
}

function toPicked(def: GameDefinition): PickedGame {
  return { id: def.id, name: def.name, ownerId: def.ownerId, rotation: def.rotation }
}

// A selectable game row.
function GameRow({ def, isSelected, onPick }: { def: GameDefinition; isSelected: boolean; onPick: () => void }) {
  return (
    <li>
      <button
        type="button"
        className={isSelected ? 'lb-picker-game lb-picker-game--selected' : 'lb-picker-game'}
        aria-label={`Show leaderboard for ${def.name}`}
        onClick={onPick}
      >
        {def.name}
      </button>
    </li>
  )
}

// An expandable collection: expanding loads its accessible members (the detail
// endpoint returns membership whole — it's bounded — so members aren't paged).
function CollectionRow({
  collection, selectedId, onPick,
}: { collection: GameCollection; selectedId: string | null; onPick: (def: GameDefinition) => void }) {
  const token = useToken()
  const [isOpen, setIsOpen] = useState(false)
  const [members, setMembers] = useState<GameDefinition[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!isOpen || members != null || error != null || !token) return
    let cancelled = false
    getGameCollection(token, collection.id)
      .then(detail => { if (!cancelled) setMembers(detail.definitions) })
      .catch((ex: unknown) => { if (!cancelled) setError((ex as Error).message || 'Failed to load games') })
    return () => { cancelled = true }
  }, [isOpen, members, error, token, collection.id])

  return (
    <li className="lb-picker-collection">
      <button
        type="button"
        className="lb-picker-collection-toggle"
        aria-expanded={isOpen}
        onClick={() => setIsOpen(o => !o)}
      >
        <span aria-hidden="true">{isOpen ? '▾' : '▸'}</span> {collection.name}
      </button>
      {isOpen && (
        <>
          {members == null && error == null && <p aria-label="Loading">Loading&hellip;</p>}
          {error && <p className="error-msg" role="alert">{error}</p>}
          {members != null && members.length === 0 && <p className="lb-picker-empty">No games you can play.</p>}
          {members != null && members.length > 0 && (
            <ul className="lb-picker-list lb-picker-members">
              {members.map(def => (
                <GameRow key={def.id} def={def} isSelected={def.id === selectedId} onPick={() => onPick(def)} />
              ))}
            </ul>
          )}
        </>
      )}
    </li>
  )
}

// The Leaderboards page's 3D-game subject picker: a compact "selected game +
// Change" summary that expands into a scoped, searchable, paged browser.
//
// Each scope shows its **collections** (expandable to their member games) above
// its **games**, each list independently paged — a collection's games are only
// reachable through it, since a game inside a Featured collection need not be
// Featured itself. Featured instead renders the single mixed list its
// admin-ordered catalogue endpoint already returns. Selecting a game collapses
// the panel.
export function LeaderboardGamePicker({ value, onSelect }: Props) {
  const token = useToken()
  const [isOpen, setIsOpen] = useState(false)
  const [scope, setScope] = useState<Scope>('featured')
  const [query, setQuery] = useState('')
  const [debounced, setDebounced] = useState('')

  // Debounce the box so a server-filtered scope doesn't refetch per keystroke.
  useEffect(() => {
    const handle = setTimeout(() => setDebounced(query), DEBOUNCE_MS)
    return () => clearTimeout(handle)
  }, [query])

  const serverQuery = filtersOnServer(scope) ? debounced.trim() : ''
  // Only the active scope's lists load; a null key parks the others. The server
  // query is part of the key, so changing it reloads from the first page.
  const key = token && isOpen ? `${token}:${scope}:${serverQuery}` : null
  const featuredKey = scope === 'featured' ? key : null
  const listKey = scope === 'featured' ? null : key

  const fetchFeatured = useCallback(
    (limit: number, offset: number) =>
      getFeaturedGameItems(token!, { limit, offset }).then(p => ({ items: p.items, hasMore: p.hasMore })),
    [token],
  )
  const fetchCollections = useCallback(
    (limit: number, offset: number) => {
      if (scope === 'featured') return Promise.resolve({ items: [] as GameCollection[], hasMore: false })
      return listGameCollections(token!, { scope, limit, offset, q: serverQuery || undefined })
        .then(p => ({ items: p.collections, hasMore: p.hasMore }))
    },
    [token, scope, serverQuery],
  )
  const fetchGames = useCallback(
    (limit: number, offset: number) => {
      if (scope === 'featured') return Promise.resolve({ items: [] as GameDefinition[], hasMore: false })
      return listGameDefinitions(token!, { scope, limit, offset, q: serverQuery || undefined })
        .then(p => ({ items: p.definitions, hasMore: p.hasMore }))
    },
    [token, scope, serverQuery],
  )

  const featured = usePagedList<FeaturedGameItem>(featuredKey, fetchFeatured, featuredItemKey, 'Failed to load featured items')
  const collections = usePagedList<GameCollection>(listKey, fetchCollections, c => c.id, 'Failed to load collections')
  const games = usePagedList<GameDefinition>(listKey, fetchGames, d => d.id, 'Failed to load games')

  // A client-filtered scope narrows its accumulated pages here; a server-filtered
  // one already came back filtered.
  const trimmed = debounced.trim().toLowerCase()
  const narrow = <T,>(items: T[], name: (item: T) => string): T[] =>
    !filtersOnServer(scope) && trimmed !== '' ? items.filter(i => name(i).toLowerCase().includes(trimmed)) : items

  const featuredItems = narrow(featured.items, featuredItemName)
  const collectionItems = narrow(collections.items, c => c.name)
  const gameItems = narrow(games.items, d => d.name)

  function pick(def: GameDefinition) {
    onSelect(toPicked(def))
    setIsOpen(false)
  }

  const selectedId = value?.id ?? null

  const loadMore = (label: string, list: { hasMore: boolean; isLoadingMore: boolean; loadMore: () => void }) =>
    list.hasMore && (
      <button type="button" className="btn-secondary workshop-load-more" onClick={list.loadMore} disabled={list.isLoadingMore}>
        {list.isLoadingMore ? 'Loading…' : `Load more ${label}`}
      </button>
    )

  return (
    <div className="lb-picker">
      <div className="lb-picker-summary">
        <span className="lb-picker-selected">{value ? value.name : 'No game selected'}</span>
        <button type="button" className="btn-secondary" onClick={() => setIsOpen(o => !o)} aria-expanded={isOpen}>
          {isOpen ? 'Close' : value ? 'Change' : 'Choose a game'}
        </button>
      </div>

      {isOpen && (
        <div className="lb-picker-panel">
          <div className="play3d-tabs" role="tablist" aria-label="Game scope">
            {SCOPES.map(s => (
              <button
                key={s.key}
                type="button"
                role="tab"
                aria-selected={scope === s.key}
                className={scope === s.key ? 'play3d-tab play3d-tab--active' : 'play3d-tab'}
                onClick={() => setScope(s.key)}
              >
                {s.label}
              </button>
            ))}
          </div>
          <input
            type="text"
            className="input play3d-search"
            aria-label="Search games"
            placeholder="Search games…"
            value={query}
            onChange={e => setQuery(e.target.value)}
          />

          {scope === 'featured' ? (
            <section className="lb-picker-section">
              {featured.isLoading && <p aria-label="Loading">Loading&hellip;</p>}
              {!featured.isLoading && featured.error && <p className="error-msg" role="alert">{featured.error}</p>}
              {!featured.isLoading && !featured.error && featuredItems.length === 0 && (
                <p className="lb-picker-empty">{trimmed !== '' ? 'No matches.' : 'Nothing featured yet.'}</p>
              )}
              {featuredItems.length > 0 && (
                <ul className="lb-picker-list">
                  {featuredItems.map(item =>
                    item.kind === 'definition' && item.definition ? (
                      <GameRow
                        key={featuredItemKey(item)}
                        def={item.definition}
                        isSelected={item.definition.id === selectedId}
                        onPick={() => pick(item.definition!)}
                      />
                    ) : (
                      <CollectionRow
                        key={featuredItemKey(item)}
                        collection={item.collection!}
                        selectedId={selectedId}
                        onPick={pick}
                      />
                    ),
                  )}
                </ul>
              )}
              {loadMore('featured', featured)}
            </section>
          ) : (
            <>
              <section className="lb-picker-section">
                <h3 className="lb-picker-heading">Collections</h3>
                {collections.isLoading && <p aria-label="Loading">Loading&hellip;</p>}
                {!collections.isLoading && collections.error && <p className="error-msg" role="alert">{collections.error}</p>}
                {!collections.isLoading && !collections.error && collectionItems.length === 0 && (
                  <p className="lb-picker-empty">No collections.</p>
                )}
                {collectionItems.length > 0 && (
                  <ul className="lb-picker-list">
                    {collectionItems.map(c => (
                      <CollectionRow key={c.id} collection={c} selectedId={selectedId} onPick={pick} />
                    ))}
                  </ul>
                )}
                {loadMore('collections', collections)}
              </section>

              <section className="lb-picker-section">
                <h3 className="lb-picker-heading">Games</h3>
                {games.isLoading && <p aria-label="Loading">Loading&hellip;</p>}
                {!games.isLoading && games.error && <p className="error-msg" role="alert">{games.error}</p>}
                {!games.isLoading && !games.error && gameItems.length === 0 && (
                  <p className="lb-picker-empty">No games.</p>
                )}
                {gameItems.length > 0 && (
                  <ul className="lb-picker-list">
                    {gameItems.map(d => (
                      <GameRow key={d.id} def={d} isSelected={d.id === selectedId} onPick={() => pick(d)} />
                    ))}
                  </ul>
                )}
                {loadMore('games', games)}
              </section>
            </>
          )}
        </div>
      )}
    </div>
  )
}
