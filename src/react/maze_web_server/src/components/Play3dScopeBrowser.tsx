import { useState } from 'react'
import { AppHeader } from './AppHeader'
import { Play3dListBody } from './Play3dListPage'
import { usePlay3dCards } from '../hooks/usePlay3dCards'
import { listGameDefinitions, listGameCollections } from '../api/client'
import type { GameCollection, GameDefinition } from '../types/api'

type Scope = 'mine' | 'shared'
type Tab = 'games' | 'collections'

// A tabbed Play-3D browse page for one ownership scope (My Games = `mine`,
// Shared with me = `shared`). Games and collections are separate name-ordered,
// independently-paged lists behind a Games / Collections tab strip, so each is a
// simple paged list with no cross-entity merge. Play/Leaderboard behaviour is
// the shared usePlay3dCards hook (same as the Featured page).
export function Play3dScopeBrowser({ scope, title }: { scope: Scope; title: string }) {
  const { definitionCard, collectionCard, overlays } = usePlay3dCards()
  const [tab, setTab] = useState<Tab>('games')
  // One Refresh reloads whichever tab is showing (both bodies read this token).
  const [refreshCount, setRefreshCount] = useState(0)

  const gamesEmpty = scope === 'mine' ? "You haven't created any 3D games yet." : 'No games have been shared with you yet.'
  const collectionsEmpty = scope === 'mine' ? "You haven't created any collections yet." : 'No collections have been shared with you yet.'

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
      </div>
      {tab === 'games' ? (
        <Play3dListBody<GameDefinition>
          key="games"
          fetchPage={(t, limit, offset) =>
            listGameDefinitions(t, { scope, limit, offset }).then(p => ({ items: p.definitions, hasMore: p.hasMore }))}
          getId={d => d.id}
          card={definitionCard}
          searchText={d => d.name}
          searchPlaceholder="Filter games…"
          emptyText={gamesEmpty}
          errorText="Failed to load games"
          refreshToken={refreshCount}
        />
      ) : (
        <Play3dListBody<GameCollection>
          key="collections"
          fetchPage={(t, limit, offset) =>
            listGameCollections(t, { scope, limit, offset }).then(p => ({ items: p.collections, hasMore: p.hasMore }))}
          getId={c => c.id}
          card={collectionCard}
          searchText={c => c.name}
          searchPlaceholder="Filter collections…"
          emptyText={collectionsEmpty}
          errorText="Failed to load collections"
          refreshToken={refreshCount}
        />
      )}
    </div>
  )
}
