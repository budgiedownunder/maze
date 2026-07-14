import { useRef, useState } from 'react'
import { WorkshopListPage, WorkshopThumbnail, type WorkshopListContext } from '../components/WorkshopListPage'
import { GameCollectionFormModal } from '../components/GameCollectionFormModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { ManageSharesModal } from '../components/ManageSharesModal'
import { useToken, useAuth } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { createGameCollection, deleteGameCollection, getGameCollection, listGameCollections, setGameCollectionItems, updateGameCollection } from '../api/client'
import { accessLabel, type Visibility } from '../utils/gameDefinitions'
import type { GameCollection } from '../types/api'

// A one-line collection summary — game count and access tier — shown under the
// name.
function collectionSummary(c: GameCollection): string {
  const count = c.items.length
  const games = count === 1 ? '1 game' : `${count} games`
  return `${games} · ${accessLabel(c.visibility)}`
}

// The workshop's Collections area: the caller's own game collections, each with
// edit (name/description + membership) / access / delete plus a New collection
// create flow, rendered through the shared WorkshopListPage shell.
export function WorkshopCollectionsPage() {
  const token = useToken()
  const { profile } = useAuth()

  // The shared list's context (refresh / patch), captured once it is ready.
  const listRef = useRef<WorkshopListContext<GameCollection> | null>(null)
  const refresh = () => listRef.current?.refresh()

  const [creating, setCreating] = useState<{ busy: boolean; error: string | null } | null>(null)
  const [editing, setEditing] = useState<{ collection: GameCollection; busy: boolean; error: string | null } | null>(null)
  const [deleting, setDeleting] = useState<{ collection: GameCollection; busy: boolean; error: string | null } | null>(null)
  const [sharing, setSharing] = useState<GameCollection | null>(null)

  useBusyCursor(!!creating?.busy || !!editing?.busy || !!deleting?.busy)

  async function handleCreate(name: string, description: string | null) {
    setCreating({ busy: true, error: null })
    try {
      await createGameCollection(token!, { name, description })
      setCreating(null)
      refresh()
    } catch (ex) {
      setCreating({ busy: false, error: (ex as Error).message || 'Failed to create collection' })
    }
  }

  async function handleEdit(collection: GameCollection, name: string, description: string | null, memberIds?: string[]) {
    setEditing({ collection, busy: true, error: null })
    try {
      // Commit only what changed: metadata (visibility preserved) and, when the
      // membership was edited, reconcile it in one call.
      const metaDirty = name !== collection.name || (description ?? null) !== (collection.description ?? null)
      if (metaDirty) {
        await updateGameCollection(token!, collection.id, { name, description, visibility: collection.visibility })
      }
      if (memberIds) {
        await setGameCollectionItems(token!, collection.id, memberIds)
      }
      setEditing(null)
      refresh()
    } catch (ex) {
      setEditing({ collection, busy: false, error: (ex as Error).message || 'Failed to save collection' })
    }
  }

  async function handleDelete() {
    if (!deleting) return
    setDeleting({ ...deleting, busy: true, error: null })
    try {
      await deleteGameCollection(token!, deleting.collection.id)
      setDeleting(null)
      refresh()
    } catch (ex) {
      setDeleting({ ...deleting, busy: false, error: (ex as Error).message || 'Failed to delete collection' })
    }
  }

  // Persist a collection's access tier — a visibility-only change (name +
  // description sent unchanged). Driven by the access modal.
  async function setCollectionVisibility(collection: GameCollection, visibility: Visibility): Promise<void> {
    await updateGameCollection(token!, collection.id, {
      name: collection.name,
      description: collection.description ?? null,
      visibility,
    })
  }

  // Re-read one collection's authoritative visibility and patch just its row —
  // used when the access modal saves. Best-effort: on failure the row is left
  // as-is and a manual Refresh still corrects it.
  async function reloadRowVisibility(id: string) {
    try {
      const detail = await getGameCollection(token!, id)
      listRef.current?.patchItem(id, { visibility: detail.visibility })
    } catch {
      // Ignore — the summary stays until the next load/refresh.
    }
  }

  const overlays = (
    <>
      {creating && (
        <GameCollectionFormModal
          title="New Game Collection"
          confirmLabel="Create"
          isLoading={creating.busy}
          error={creating.error}
          onSubmit={(name, description) => void handleCreate(name, description)}
          onCancel={() => setCreating(null)}
        />
      )}
      {editing && (
        <GameCollectionFormModal
          title="Edit Collection"
          confirmLabel="Save"
          initialName={editing.collection.name}
          initialDescription={editing.collection.description ?? ''}
          collectionId={editing.collection.id}
          isLoading={editing.busy}
          error={editing.error}
          onSubmit={(name, description, memberIds) => void handleEdit(editing.collection, name, description, memberIds)}
          onCancel={() => setEditing(null)}
        />
      )}
      {deleting && (
        <ConfirmModal
          title="Delete Collection"
          message={`Delete “${deleting.collection.name}”? This removes the collection only — the games in it are not deleted. This cannot be undone.`}
          confirmLabel="Delete"
          isDangerous
          isLoading={deleting.busy}
          error={deleting.error}
          onConfirm={() => void handleDelete()}
          onCancel={() => setDeleting(null)}
        />
      )}
      {sharing && (
        <ManageSharesModal
          subject={{ kind: 'collection', id: sharing.id, name: sharing.name, ownerId: sharing.ownerId }}
          visibility={sharing.visibility}
          isAdmin={!!profile?.is_admin}
          onSetVisibility={v => setCollectionVisibility(sharing, v)}
          onSaved={() => { const id = sharing.id; setSharing(null); void reloadRowVisibility(id) }}
          onClose={() => setSharing(null)}
        />
      )}
    </>
  )

  return (
    <WorkshopListPage<GameCollection>
      title="Manage Game Collections"
      newLabel="+ New Game Collection"
      onNew={() => setCreating({ busy: false, error: null })}
      // Own collections only (server-scoped, paged).
      fetchPage={(t, limit, offset) => listGameCollections(t, { scope: 'mine', limit, offset }).then(p => ({ items: p.collections, hasMore: p.hasMore }))}
      getId={c => c.id}
      emptyText="No collections yet."
      errorText="Failed to load collections"
      onReady={ctx => { listRef.current = ctx }}
      overlays={overlays}
      // The whole row edits on mouse-click as a convenience; keyboard /
      // screen-reader users use the explicit Edit action (the row is not a button
      // containing buttons).
      row={c => ({
        name: c.name,
        subtitle: collectionSummary(c),
        thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game-collection.svg" visibility={c.visibility} />,
        onOpen: () => setEditing({ collection: c, busy: false, error: null }),
        actions: [
          { key: 'edit', label: 'Edit', ariaLabel: `Edit ${c.name}`, icon: '/images/icons/icon_rename.png', onClick: () => setEditing({ collection: c, busy: false, error: null }) },
          { key: 'access', label: 'Access', ariaLabel: `Access for ${c.name}`, icon: '/images/icons/icon_share.svg', onClick: () => setSharing(c) },
          { key: 'delete', label: 'Delete', ariaLabel: `Delete ${c.name}`, icon: '/images/icons/icon_delete.png', onClick: () => setDeleting({ collection: c, busy: false, error: null }), variant: 'danger' },
        ],
      })}
    />
  )
}
