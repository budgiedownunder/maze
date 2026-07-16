import { Play3dListPage, type Play3dCard } from '../components/Play3dListPage'
import { usePlay3dCards } from '../hooks/usePlay3dCards'
import { getFeaturedGameItems } from '../api/client'
import type { FeaturedGameItem } from '../types/api'

// Kind-namespaced list key so a definition and a collection with the same id
// never collide (mirrors the workshop Features page's key).
function featuredKey(item: FeaturedGameItem): string {
  const id = item.kind === 'definition' ? item.definition!.id : item.collection!.id
  return `${item.kind}:${id}`
}

function featuredName(item: FeaturedGameItem): string {
  return item.kind === 'definition' ? item.definition!.name : item.collection!.name
}

// The Featured browse page: the admin-ordered catalogue of curated games +
// collections, rendered as gallery cards via the reusable Play3dListPage. Play
// and Leaderboard behaviour comes from the shared usePlay3dCards hook (a game
// or single-game collection launches directly; a multi-game Arcade collection
// opens a free-choice picker; a multi-game Campaign collection opens the ordered
// campaign modal; a game card also opens the leaderboard).
export function Play3dFeaturedPage() {
  const { definitionCard, collectionCard, overlays } = usePlay3dCards()

  const card = (item: FeaturedGameItem): Play3dCard =>
    item.kind === 'definition' && item.definition
      ? definitionCard(item.definition)
      : collectionCard(item.collection!)

  return (
    <Play3dListPage<FeaturedGameItem>
      title="Featured"
      fetchPage={(t, limit, offset) => getFeaturedGameItems(t, { limit, offset }).then(p => ({ items: p.items, hasMore: p.hasMore }))}
      getId={featuredKey}
      card={card}
      searchText={featuredName}
      searchPlaceholder="Filter featured…"
      emptyText="No featured games or collections yet."
      errorText="Failed to load featured items"
      overlays={overlays}
    />
  )
}
