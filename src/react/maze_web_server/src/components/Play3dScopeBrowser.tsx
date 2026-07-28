import { useState } from 'react'
import { AppHeader } from './AppHeader'
import { Play3dListBody, type Play3dListSearch } from './Play3dListPage'
import { usePlay3dCards } from '../hooks/usePlay3dCards'
import { listGameDefinitions, listGameCollections } from '../api/client'
import type { GameCollection, GameDefinition } from '../types/api'

type Scope = 'mine' | 'shared' | 'public'
type Tab = 'games' | 'collections'
type Sort = 'name' | 'newest'

const SORTS: { key: Sort; label: string }[] = [
  { key: 'name', label: 'A–Z' },
  { key: 'newest', label: 'Newest' },
]

// Community is the one unbounded scope, so it searches server-side (`q`) and
// offers a sort; the owned/shared scopes are bounded, so they filter the loaded
// pages client-side and stay name-ordered.
function isUnbounded(scope: Scope): boolean {
  return scope === 'public'
}

// A tabbed Play-3D browse page for one ownership scope (My Games = `mine`,
// Shared with me = `shared`, Community = `public`). Games and collections are
// separate name-ordered, independently-paged lists behind a Games / Collections
// tab strip, so each is a simple paged list with no cross-entity merge.
// Play/Leaderboard behaviour is the shared usePlay3dCards hook (same as the
// Featured page).
export function Play3dScopeBrowser({ scope, title }: { scope: Scope; title: string }) {
  const { definitionCard, collectionCard, overlays } = usePlay3dCards()
  const [tab, setTab] = useState<Tab>('games')
  const [sort, setSort] = useState<Sort>('name')
  // One Refresh reloads whichever tab is showing (both bodies read this token).
  const [refreshCount, setRefreshCount] = useState(0)

  const unbounded = isUnbounded(scope)
  // The sort is a server-side query parameter, so it belongs in the list key
  // alongside the refresh counter — changing either reloads from page 1.
  const queryToken = `${refreshCount}:${sort}`
  const sortParam = unbounded ? sort : undefined

  const gamesSearch: Play3dListSearch<GameDefinition> = unbounded
    ? { mode: 'server' }
    : { mode: 'client', text: d => d.name }
  const collectionsSearch: Play3dListSearch<GameCollection> = unbounded
    ? { mode: 'server' }
    : { mode: 'client', text: c => c.name }

  const gamesEmpty = scope === 'mine'
    ? "You haven't created any 3D games yet."
    : scope === 'shared'
      ? 'No games have been shared with you yet.'
      : 'No games have been published yet.'
  const collectionsEmpty = scope === 'mine'
    ? "You haven't created any collections yet."
    : scope === 'shared'
      ? 'No collections have been shared with you yet.'
      : 'No collections have been published yet.'

  return (
    <div className="games-page">
      {overlays}
      <AppHeader title={title}>
        <button className="btn-icon" onClick={() => setRefreshCount(c => c + 1)} aria-label="Refresh" title="Refresh">
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <div className="play3d-tabs" role="tablist" aria-label={`${title} content`}>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'games'}
          className={tab === 'games' ? 'play3d-tab play3d-tab--active' : 'play3d-tab'}
          onClick={() => setTab('games')}
        >
          Games
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'collections'}
          className={tab === 'collections' ? 'play3d-tab play3d-tab--active' : 'play3d-tab'}
          onClick={() => setTab('collections')}
        >
          Collections
        </button>
        {unbounded && (
          <label className="play3d-sort">
            Sort
            <select
              className="subject-select"
              aria-label="Sort"
              value={sort}
              onChange={e => setSort(e.target.value as Sort)}
            >
              {SORTS.map(s => <option key={s.key} value={s.key}>{s.label}</option>)}
            </select>
          </label>
        )}
      </div>
      {tab === 'games' ? (
        <Play3dListBody<GameDefinition>
          key="games"
          fetchPage={(t, limit, offset, query) =>
            listGameDefinitions(t, { scope, limit, offset, q: query || undefined, sort: sortParam })
              .then(p => ({ items: p.definitions, hasMore: p.hasMore }))}
          getId={d => d.id}
          card={definitionCard}
          search={gamesSearch}
          searchPlaceholder={unbounded ? 'Search games…' : 'Filter games…'}
          emptyText={gamesEmpty}
          errorText="Failed to load games"
          queryToken={queryToken}
        />
      ) : (
        <Play3dListBody<GameCollection>
          key="collections"
          fetchPage={(t, limit, offset, query) =>
            listGameCollections(t, { scope, limit, offset, q: query || undefined, sort: sortParam })
              .then(p => ({ items: p.collections, hasMore: p.hasMore }))}
          getId={c => c.id}
          card={collectionCard}
          search={collectionsSearch}
          searchPlaceholder={unbounded ? 'Search collections…' : 'Filter collections…'}
          emptyText={collectionsEmpty}
          errorText="Failed to load collections"
          queryToken={queryToken}
        />
      )}
    </div>
  )
}
