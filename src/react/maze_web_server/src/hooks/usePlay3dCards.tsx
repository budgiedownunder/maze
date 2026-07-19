import { useState, type ReactNode } from 'react'
import { WorkshopThumbnail } from '../components/WorkshopListPage'
import { GameLeaderboardModal } from '../components/GameLeaderboardModal'
import { ArcadeCollectionModal } from '../components/ArcadeCollectionModal'
import { CampaignCollectionModal } from '../components/CampaignCollectionModal'
import type { Play3dCard } from '../components/Play3dListPage'
import { getGameCollection, getCompletedChallenges } from '../api/client'
import { launchDefinition } from '../utils/play3dLaunch'
import { gameChallengeKey } from '../utils/gameDefinitions'
import { useBusyCursor } from './useBusyCursor'
import { useToken, useAuth } from '../context/AuthContext'
import type { GameCollection, GameDefinition } from '../types/api'

// Shared Play/Leaderboard behaviour for the Play-3D browse pages (Featured, My
// Games, Shared with me): maps a game or collection to its `Play3dCard`, decides
// how to play a collection from its *accessible* members, and renders the
// leaderboard / Arcade-picker / Campaign overlays. Each page supplies whatever
// list source it wants and reuses these mappers + overlays so the play logic
// lives in one place.
export function usePlay3dCards(): {
  definitionCard: (d: GameDefinition) => Play3dCard
  collectionCard: (c: GameCollection) => Play3dCard
  overlays: ReactNode
} {
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
          completed = (await getCompletedChallenges(token!, defs.map(d => gameChallengeKey(d.id, d.rotation)))).completed
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

  const definitionCard = (d: GameDefinition): Play3dCard => ({
    name: d.name,
    description: d.description,
    thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={d.visibility} />,
    actions: [
      { key: 'play', label: 'Play', ariaLabel: `Play ${d.name}`, variant: 'primary', icon: '/images/icons/icon_play_3d.png', onClick: () => launchDefinition(d.id) },
      { key: 'board', label: 'Leaderboard', ariaLabel: `Leaderboard for ${d.name}`, icon: '/images/icons/icon_leaderboard.svg', onClick: () => setViewingBoard({ id: d.id, name: d.name }) },
    ],
  })

  const collectionCard = (c: GameCollection): Play3dCard => ({
    name: c.name,
    description: c.description,
    thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game-collection.svg" visibility={c.visibility} playMode={c.playMode} />,
    // Every collection resolves its accessible members on click (see
    // playCollection) — launch the sole game, open the Campaign modal or the
    // Arcade picker, or the picker's guarded state — so an inaccessible member is
    // never launched.
    actions: [
      { key: 'play', label: 'Play', ariaLabel: `Play ${c.name}`, variant: 'primary', icon: '/images/icons/icon_play_3d.png', onClick: () => void playCollection(c) },
    ],
  })

  const overlays = (
    <>
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
    </>
  )

  return { definitionCard, collectionCard, overlays }
}
