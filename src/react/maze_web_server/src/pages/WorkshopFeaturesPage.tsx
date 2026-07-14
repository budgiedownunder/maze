import { useCallback, useState } from 'react'
import { Navigate } from 'react-router-dom'
import { AppHeader } from '../components/AppHeader'
import { WorkshopThumbnail } from '../components/WorkshopListPage'
import { GameDefinitionEditor } from '../components/GameDefinitionEditor'
import { GameCollectionFormModal } from '../components/GameCollectionFormModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { GameLeaderboardModal } from '../components/GameLeaderboardModal'
import { useToken, useAuth } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { usePagedList } from '../hooks/usePagedList'
import {
  getFeaturedGameItems,
  setFeaturedGameItemsOrder,
  getGameDefinition,
  getLeaderboard,
  reshuffleGameDefinition,
  updateGameDefinition,
  updateGameCollection,
  setGameCollectionItems,
} from '../api/client'
import { parseDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import { launchDefinitionPreview } from '../utils/definitionPreview'
import { launchDefinition } from '../utils/play3dLaunch'
import { type Visibility } from '../utils/gameDefinitions'
import type { FeaturedGameItem, FeaturedGameItemEntry, GameCollection, GameDefinition, GameDefinitionRequest, GamePlayResponse } from '../types/api'

// Server hard cap on a page — used to walk the whole catalogue when a reorder
// needs the complete order (the list is displayed paged, but a reorder replaces
// the entire order, so it must submit every entry, not just the loaded page).
const MAX_PAGE = 100

// The entity (def or collection) behind a featured row, flattened to the fields
// the row + modals need regardless of kind.
function featuredEntity(item: FeaturedGameItem): { id: string; name: string; ownerId: string; visibility: Visibility } {
  if (item.kind === 'definition' && item.definition) {
    const d = item.definition
    return { id: d.id, name: d.name, ownerId: d.ownerId, visibility: d.visibility }
  }
  const c = item.collection!
  return { id: c.id, name: c.name, ownerId: c.ownerId, visibility: c.visibility }
}

// A stable list key — kind-namespaced so a def and a collection never collide.
function featuredKey(item: FeaturedGameItem): string {
  return `${item.kind}:${featuredEntity(item).id}`
}

// A one-line summary shown under the name: the kind, the game's level/rotation or
// the collection's game count, and who owns it.
function featuredSummary(item: FeaturedGameItem): string {
  let base: string
  if (item.kind === 'definition' && item.definition) {
    const d = item.definition
    const count = Number((d.config.levels as { count?: number } | undefined)?.count) || 1
    const levels = count <= 1 ? 'Single level' : `${count} levels`
    base = `Game · ${levels} · ${d.rotation === 'daily' ? 'Daily' : 'Static'}`
  } else {
    const n = item.collection?.items.length ?? 0
    base = `Collection · ${n === 1 ? '1 game' : `${n} games`}`
  }
  return `${base} · by ${item.ownerUsername}`
}

// The admin-only Manage Features page: the featured catalogue (curated games +
// collections) as one admin-ordered list, with reorder (↑/↓), per-row Edit
// (admin-override — reuse the game editor / collection form), Unfeature (reset
// the item's access to its owner-only Private tier, dropping it off the list),
// and Play / Leaderboard for games. The list is paged (Load more); a reorder
// submits the whole order.
export function WorkshopFeaturesPage() {
  const token = useToken()
  const { profile } = useAuth()

  const [refreshCount, setRefreshCount] = useState(0)
  // Only load for an admin (the render bounces a non-admin below); keying by the
  // refresh counter resets the view by derivation.
  const key = token && profile?.is_admin ? `${token}:${refreshCount}` : null
  const fetchPage = useCallback(
    (limit: number, offset: number) => getFeaturedGameItems(token!, { limit, offset }).then(p => ({ items: p.items, hasMore: p.hasMore })),
    [token],
  )
  const list = usePagedList<FeaturedGameItem>(key, fetchPage, featuredKey, 'Failed to load featured items')
  const items = list.items
  const refresh = useCallback(() => setRefreshCount(c => c + 1), [])

  const [editingDef, setEditingDef] = useState<{ id: string; form: DefinitionFormState; hasScores: boolean } | null>(null)
  const [editingCol, setEditingCol] = useState<{ collection: GameCollection; busy: boolean; error: string | null } | null>(null)
  const [unfeaturing, setUnfeaturing] = useState<{ item: FeaturedGameItem; busy: boolean; error: string | null } | null>(null)
  const [viewingBoard, setViewingBoard] = useState<GameDefinition | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [actionBusy, setActionBusy] = useState(false)
  useBusyCursor(actionBusy || !!editingCol?.busy)

  // Whether a definition's board already has scores drives the stronger
  // reshuffle / board-reset confirm wording inside the editor.
  async function hasScores(def: GamePlayResponse): Promise<boolean> {
    const board = def.leaderboardTracked ? await getLeaderboard(token!, { challenge: def.challengeKey, limit: 1 }) : null
    return (board?.scores.length ?? 0) > 0
  }

  // Walk every page so a reorder submits the complete order (the display is
  // paged, but the order replace must carry the whole catalogue or it would drop
  // the unloaded tail).
  async function fetchAllEntries(): Promise<FeaturedGameItemEntry[]> {
    const entries: FeaturedGameItemEntry[] = []
    let offset = 0
    for (;;) {
      const page = await getFeaturedGameItems(token!, { limit: MAX_PAGE, offset })
      for (const it of page.items) entries.push({ kind: it.kind, id: featuredEntity(it).id })
      if (!page.hasMore) break
      offset += MAX_PAGE
    }
    return entries
  }

  async function moveFeatured(item: FeaturedGameItem, direction: -1 | 1) {
    setActionError(null)
    setActionBusy(true)
    try {
      const entries = await fetchAllEntries()
      const id = featuredEntity(item).id
      const i = entries.findIndex(e => e.kind === item.kind && e.id === id)
      const j = i + direction
      if (i < 0 || j < 0 || j >= entries.length) return
      ;[entries[i], entries[j]] = [entries[j], entries[i]]
      await setFeaturedGameItemsOrder(token!, entries)
      refresh()
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to reorder.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleEditDefinition(id: string) {
    setActionError(null)
    setActionBusy(true)
    try {
      const def = await getGameDefinition(token!, id)
      const form = parseDefinitionConfig(def.config, {
        name: def.name,
        description: def.description,
        visibility: def.visibility,
        rotation: def.rotation,
      })
      // The play-fetch splices an effective seed into config, so hydrate the seed
      // from the record's own field (as Manage Games does).
      setEditingDef({ id, form: { ...form, seed: def.seed }, hasScores: await hasScores(def) })
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to load game.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleSaveDefinition(request: GameDefinitionRequest) {
    if (!editingDef) return
    setActionBusy(true)
    try {
      await updateGameDefinition(token!, editingDef.id, request)
      setEditingDef(null)
      refresh()
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to save game.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleSaveCollection(name: string, description: string | null, memberIds?: string[]) {
    if (!editingCol) return
    const c = editingCol.collection
    setEditingCol({ ...editingCol, busy: true, error: null })
    try {
      const metaDirty = name !== c.name || (description ?? null) !== (c.description ?? null)
      if (metaDirty) {
        await updateGameCollection(token!, c.id, { name, description, visibility: c.visibility })
      }
      if (memberIds) {
        await setGameCollectionItems(token!, c.id, memberIds)
      }
      setEditingCol(null)
      refresh()
    } catch (ex) {
      setEditingCol(e => (e ? { ...e, busy: false, error: (ex as Error).message || 'Failed to save collection' } : e))
    }
  }

  // Re-tier a featured item (visibility-only; the stored config/metadata is sent
  // unchanged, so no board reset). Leaving the curated tier un-features it.
  async function setFeaturedVisibility(item: FeaturedGameItem, visibility: Visibility): Promise<void> {
    if (item.kind === 'definition' && item.definition) {
      const d = item.definition
      await updateGameDefinition(token!, d.id, { name: d.name, description: d.description ?? null, visibility, rotation: d.rotation, config: d.config })
    } else if (item.collection) {
      const c = item.collection
      await updateGameCollection(token!, c.id, { name: c.name, description: c.description ?? null, visibility })
    }
  }

  // Un-feature: reset the item to Private (owner-only), which drops it off the
  // featured list. Reversible — the owner (or an admin) can re-feature it.
  async function handleConfirmUnfeature() {
    if (!unfeaturing) return
    setUnfeaturing(u => (u ? { ...u, busy: true, error: null } : u))
    try {
      await setFeaturedVisibility(unfeaturing.item, 'private')
      setUnfeaturing(null)
      refresh()
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to unfeature.'
      setUnfeaturing(u => (u ? { ...u, busy: false, error: message } : u))
    }
  }

  // Hooks above run unconditionally; the admin gate is applied to the render so a
  // non-admin who navigates here directly is bounced back to the hub.
  if (!profile?.is_admin) return <Navigate to="/workshop" replace />

  // The confirm names where access lands: "Just me" when the admin owns it, else
  // "the owner (username)".
  let unfeatureMessage = ''
  if (unfeaturing) {
    const owner = featuredEntity(unfeaturing.item)
    const target = owner.ownerId === profile.id ? 'Just me' : `the owner (${unfeaturing.item.ownerUsername})`
    unfeatureMessage = `Unfeature “${owner.name}”? This removes it from the featured list and resets its access to ${target}.`
  }

  return (
    <div className="games-page">
      {editingDef && (
        <GameDefinitionEditor
          mode="tabs"
          title="Edit Game"
          initialForm={editingDef.form}
          onSubmit={request => void handleSaveDefinition(request)}
          onCancel={() => { setEditingDef(null); setActionError(null) }}
          hasScores={editingDef.hasScores}
          onReshuffle={() => reshuffleGameDefinition(token!, editingDef.id).then(d => d.seed)}
          onPreview={config => launchDefinitionPreview(config, true)}
        />
      )}
      {editingCol && (
        <GameCollectionFormModal
          title="Edit Collection"
          confirmLabel="Save"
          initialName={editingCol.collection.name}
          initialDescription={editingCol.collection.description ?? ''}
          collectionId={editingCol.collection.id}
          isLoading={editingCol.busy}
          error={editingCol.error}
          onSubmit={(name, description, memberIds) => void handleSaveCollection(name, description, memberIds)}
          onCancel={() => setEditingCol(null)}
        />
      )}
      {unfeaturing && (
        <ConfirmModal
          title="Unfeature"
          message={unfeatureMessage}
          confirmLabel="Unfeature"
          isLoading={unfeaturing.busy}
          error={unfeaturing.error}
          onConfirm={() => void handleConfirmUnfeature()}
          onCancel={() => setUnfeaturing(null)}
        />
      )}
      {viewingBoard && (
        <GameLeaderboardModal
          token={token!}
          gameId={viewingBoard.id}
          name={viewingBoard.name}
          currentUserId={profile?.id}
          onClose={() => setViewingBoard(null)}
        />
      )}

      <AppHeader title="Manage Features">
        <button
          className="btn-icon"
          onClick={refresh}
          aria-label="Refresh"
          title="Refresh"
        >
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <main className="maze-list-page">
        {actionError && <p className="error-msg" role="alert">{actionError}</p>}
        {list.isLoading && <p aria-label="Loading">Loading…</p>}
        {!list.isLoading && list.error && <p className="error-msg" role="alert">{list.error}</p>}
        {!list.isLoading && !list.error && items.length === 0 && <p>No featured items yet. Set a game or collection's access to Featured to add it here.</p>}
        {!list.isLoading && !list.error && items.length > 0 && (
          <>
            <ul className="game-list">
              {items.map((item, index) => {
                const e = featuredEntity(item)
                const isDef = item.kind === 'definition'
                return (
                  <li key={featuredKey(item)} className="game-list-item" style={{ cursor: 'default' }}>
                    <div className="featured-reorder">
                      <button type="button" className="btn-icon" aria-label={`Move ${e.name} up`} disabled={index === 0 || actionBusy} onClick={() => void moveFeatured(item, -1)}>
                        <img src="/images/icons/icon_move_up.svg" alt="" aria-hidden="true" />
                      </button>
                      <button type="button" className="btn-icon" aria-label={`Move ${e.name} down`} disabled={(index === items.length - 1 && !list.hasMore) || actionBusy} onClick={() => void moveFeatured(item, 1)}>
                        <img src="/images/icons/icon_move_down.svg" alt="" aria-hidden="true" />
                      </button>
                    </div>
                    <WorkshopThumbnail baseSrc={isDef ? '/images/workshop/workshop-game.svg' : '/images/workshop/workshop-game-collection.svg'} visibility={e.visibility} />
                    <div className="maze-item-text">
                      <span className="maze-item-name" title={e.name}>{e.name}</span>
                      <span className="maze-item-subtitle">{featuredSummary(item)}</span>
                    </div>
                    <div className="game-item-actions">
                      <button type="button" className="maze-item-action btn-secondary" aria-label={`Edit ${e.name}`} onClick={() => (isDef ? void handleEditDefinition(e.id) : setEditingCol({ collection: item.collection!, busy: false, error: null }))}>
                        <img src="/images/icons/icon_rename.png" alt="" aria-hidden="true" />
                        <span className="maze-item-action-label">Edit</span>
                      </button>
                      {isDef && (
                        <button type="button" className="maze-item-action btn-secondary" aria-label={`Play ${e.name}`} onClick={() => launchDefinition(e.id)}>
                          <img src="/images/icons/icon_play_3d.png" alt="" aria-hidden="true" />
                          <span className="maze-item-action-label">Play</span>
                        </button>
                      )}
                      {isDef && (
                        <button type="button" className="maze-item-action btn-secondary" aria-label={`Leaderboard for ${e.name}`} onClick={() => setViewingBoard(item.definition!)}>
                          <img src="/images/icons/icon_leaderboard.svg" alt="" aria-hidden="true" />
                          <span className="maze-item-action-label">Leaderboard</span>
                        </button>
                      )}
                      <button type="button" className="maze-item-action btn-secondary" aria-label={`Unfeature ${e.name}`} onClick={() => setUnfeaturing({ item, busy: false, error: null })}>
                        <img src="/images/icons/icon_unfeature.svg" alt="" aria-hidden="true" />
                        <span className="maze-item-action-label">Unfeature</span>
                      </button>
                    </div>
                  </li>
                )
              })}
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
