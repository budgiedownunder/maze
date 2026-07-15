import { useState } from 'react'
import { Play3dListPage, type Play3dCard } from '../components/Play3dListPage'
import { WorkshopThumbnail } from '../components/WorkshopListPage'
import { GameLeaderboardModal } from '../components/GameLeaderboardModal'
import { getFeaturedGameItems } from '../api/client'
import { launchDefinition } from '../utils/play3dLaunch'
import { useToken, useAuth } from '../context/AuthContext'
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
// launches a game (or a single-game collection); a game card also opens the
// leaderboard. A multi-game collection can't be played until its Arcade picker
// (D4.4) / Campaign modal (D4.5) lands, so its Play is disabled for now.
export function Play3dFeaturedPage() {
  const token = useToken()
  const { profile } = useAuth()
  const [viewingBoard, setViewingBoard] = useState<{ id: string; name: string } | null>(null)

  const card = (item: FeaturedGameItem): Play3dCard => {
    if (item.kind === 'definition' && item.definition) {
      const d = item.definition
      return {
        name: d.name,
        description: d.description,
        thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={d.visibility} />,
        actions: [
          { key: 'play', label: 'Play', ariaLabel: `Play ${d.name}`, variant: 'primary', icon: '/images/icons/icon_play_3d.png', onClick: () => launchDefinition(d.id) },
          { key: 'board', label: 'Leaderboard', ariaLabel: `Leaderboard for ${d.name}`, icon: '/images/icons/icon_leaderboard.svg', onClick: () => setViewingBoard({ id: d.id, name: d.name }) },
        ],
      }
    }
    const c = item.collection!
    // A single-game collection plays its one member directly; a multi-game one
    // needs the not-yet-built picker, so Play is disabled with an explanation.
    const soleMember = c.items.length === 1 ? c.items[0].definitionId : null
    return {
      name: c.name,
      description: c.description,
      thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game-collection.svg" visibility={c.visibility} playMode={c.playMode} />,
      actions: [
        soleMember
          ? { key: 'play', label: 'Play', ariaLabel: `Play ${c.name}`, variant: 'primary', icon: '/images/icons/icon_play_3d.png', onClick: () => launchDefinition(soleMember) }
          : { key: 'play', label: 'Play', ariaLabel: `Play ${c.name}`, variant: 'primary', icon: '/images/icons/icon_play_3d.png', onClick: () => {}, disabled: true, title: 'Choosing a game to play is coming soon' },
      ],
    }
  }

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
      overlays={viewingBoard && (
        <GameLeaderboardModal
          token={token!}
          gameId={viewingBoard.id}
          name={viewingBoard.name}
          currentUserId={profile?.id}
          onClose={() => setViewingBoard(null)}
        />
      )}
    />
  )
}
