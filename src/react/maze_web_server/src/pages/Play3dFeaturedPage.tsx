import { useState } from 'react'
import { Play3dListPage, type Play3dCard } from '../components/Play3dListPage'
import { WorkshopThumbnail } from '../components/WorkshopListPage'
import { GameLeaderboardModal } from '../components/GameLeaderboardModal'
import { ArcadeCollectionModal } from '../components/ArcadeCollectionModal'
import { CampaignCollectionModal } from '../components/CampaignCollectionModal'
import { getFeaturedGameItems, getGameCollection, getCompletedChallenges } from '../api/client'
import { launchDefinition } from '../utils/play3dLaunch'
import { gameChallengeKey } from '../utils/gameDefinitions'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { useToken, useAuth } from '../context/AuthContext'
import type { FeaturedGameItem, GameCollection, GameDefinition } from '../types/api'

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
// launches a game (or a single-game collection); a multi-game Arcade collection
// opens a free-choice picker; a multi-game Campaign collection opens the ordered
// campaign modal; a game card also opens the leaderboard.
export function Play3dFeaturedPage() {
  const token = useToken()
  const { profile } = useAuth()
  const [viewingBoard, setViewingBoard] = useState<{ id: string; name: string } | null>(null)
  const [arcadePicker, setArcadePicker] = useState<{ name: string; definitions: GameDefinition[] } | null>(null)
  const [campaign, setCampaign] = useState<{ name: string; definitions: GameDefinition[]; completed: string[] } | null>(null)
  const [resolving, setResolving] = useState(false)
  useBusyCursor(resolving)

  // Decide how to play a collection from its *accessible* members (the raw
  // membership can reference games the viewer can't see), fetched on click:
  // exactly one accessible game launches directly; a Campaign collection opens
  // the ordered modal (with each game's completion resolved); otherwise the
  // Arcade picker — or, with no accessible games, its guarded state. A load
  // failure is treated as nothing playable.
  async function playCollection(c: GameCollection): Promise<void> {
    setResolving(true)
    try {
      const detail = await getGameCollection(token!, c.id)
      const defs = detail.definitions
      if (defs.length === 1) {
        launchDefinition(defs[0].id)
      } else if (defs.length > 1 && c.playMode === 'campaign') {
        // Resolve per-game completion in one query; a failure shows no progress.
        let completed: string[] = []
        try {
          completed = (await getCompletedChallenges(token!, defs.map(d => gameChallengeKey(d.id)))).completed
        } catch { /* treat as none completed */ }
        setCampaign({ name: c.name, definitions: defs, completed })
      } else {
        setArcadePicker({ name: c.name, definitions: defs })
      }
    } catch {
      setArcadePicker({ name: c.name, definitions: [] })
    } finally {
      setResolving(false)
    }
  }

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
    // Every collection resolves its accessible members on click (see
    // playCollection) — launch the sole game, open the Campaign modal or the
    // Arcade picker, or the picker's guarded state — so an inaccessible member is
    // never launched.
    const play = { key: 'play', label: 'Play', ariaLabel: `Play ${c.name}`, variant: 'primary' as const, icon: '/images/icons/icon_play_3d.png', onClick: () => void playCollection(c) }
    return {
      name: c.name,
      description: c.description,
      thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game-collection.svg" visibility={c.visibility} playMode={c.playMode} />,
      actions: [play],
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
      overlays={<>
        {viewingBoard && (
          <GameLeaderboardModal
            token={token!}
            gameId={viewingBoard.id}
            name={viewingBoard.name}
            currentUserId={profile?.id}
            onClose={() => setViewingBoard(null)}
          />
        )}
        {arcadePicker && (
          <ArcadeCollectionModal
            name={arcadePicker.name}
            definitions={arcadePicker.definitions}
            onClose={() => setArcadePicker(null)}
          />
        )}
        {campaign && (
          <CampaignCollectionModal
            name={campaign.name}
            definitions={campaign.definitions}
            completed={campaign.completed}
            onClose={() => setCampaign(null)}
          />
        )}
      </>}
    />
  )
}
