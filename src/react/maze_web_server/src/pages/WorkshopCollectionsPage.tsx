import { useEffect, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { GameCollectionFormModal } from '../components/GameCollectionFormModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { ManageSharesModal } from '../components/ManageSharesModal'
import { useToken, useAuth } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { createGameCollection, deleteGameCollection, getGameCollection, listGameCollections, updateGameCollection } from '../api/client'
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
// edit / access / delete plus a New collection create flow. The list endpoint
// merges own + shared + public + curated, so we filter to the caller's own here.
// Membership editing (open/reorder) lands in the following step.
export function WorkshopCollectionsPage() {
  const token = useToken()
  const { profile } = useAuth()

  // Load state keyed by the refresh counter, so a refresh resets the view by
  // derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; collections: GameCollection[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const [creating, setCreating] = useState<{ busy: boolean; error: string | null } | null>(null)
  const [editing, setEditing] = useState<{ collection: GameCollection; busy: boolean; error: string | null } | null>(null)
  const [deleting, setDeleting] = useState<{ collection: GameCollection; busy: boolean; error: string | null } | null>(null)
  const [sharing, setSharing] = useState<GameCollection | null>(null)

  useEffect(() => {
    if (!token) return
    let cancelled = false
    const key = refreshCount
    listGameCollections(token)
      .then(page => { if (!cancelled) setLoaded({ key, collections: page.collections }) })
      .catch(ex => {
        if (!cancelled) setErrorFor({ key, message: (ex as Error).message || 'Failed to load collections' })
      })
    return () => { cancelled = true }
  }, [token, refreshCount])

  const error = errorFor != null && errorFor.key === refreshCount ? errorFor.message : null
  const current = loaded != null && loaded.key === refreshCount ? loaded : null
  const collections = (current?.collections ?? []).filter(c => c.ownerId === profile?.id)
  const isLoading = current == null && error == null

  useBusyCursor(!!creating?.busy || !!editing?.busy || !!deleting?.busy)

  async function handleCreate(name: string, description: string | null) {
    setCreating({ busy: true, error: null })
    try {
      await createGameCollection(token!, { name, description })
      setCreating(null)
      setRefreshCount(c => c + 1)
    } catch (ex) {
      setCreating({ busy: false, error: (ex as Error).message || 'Failed to create collection' })
    }
  }

  async function handleEdit(collection: GameCollection, name: string, description: string | null) {
    setEditing({ collection, busy: true, error: null })
    try {
      // Membership + visibility are managed elsewhere; the edit form only touches
      // the name/description, so the stored visibility is preserved.
      await updateGameCollection(token!, collection.id, { name, description, visibility: collection.visibility })
      setEditing(null)
      setRefreshCount(c => c + 1)
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
      setRefreshCount(c => c + 1)
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
      setLoaded(prev =>
        prev == null
          ? prev
          : { ...prev, collections: prev.collections.map(c => (c.id === id ? { ...c, visibility: detail.visibility } : c)) })
    } catch {
      // Ignore — the summary stays until the next load/refresh.
    }
  }

  return (
    <div className="games-page">
      {creating && (
        <GameCollectionFormModal
          title="New Collection"
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
          isLoading={editing.busy}
          error={editing.error}
          onSubmit={(name, description) => void handleEdit(editing.collection, name, description)}
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
      <AppHeader title="Manage Game Collections">
        <button type="button" className="btn-primary" onClick={() => setCreating({ busy: false, error: null })}>
          + New collection
        </button>
        <button
          className="btn-icon"
          onClick={() => setRefreshCount(c => c + 1)}
          aria-label="Refresh"
          title="Refresh"
        >
          <img src="/images/maze/refresh.png" alt="Refresh" style={{ width: '1.1rem', height: '1.1rem' }} />
        </button>
      </AppHeader>
      <main className="maze-list-page">
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && <p className="error-msg" role="alert">{error}</p>}
        {!isLoading && !error && collections.length === 0 && <p>No collections yet.</p>}
        {!isLoading && !error && collections.length > 0 && (
          <ul className="game-list">
            {collections.map(c => (
              <li
                key={c.id}
                className="game-list-item"
                onClick={() => setEditing({ collection: c, busy: false, error: null })}
              >
                <div className="maze-item-text">
                  <span className="maze-item-name" title={c.name}>{c.name}</span>
                  <span className="maze-item-subtitle">{collectionSummary(c)}</span>
                </div>
                <div className="game-item-actions">
                  <button type="button" className="maze-item-action btn-secondary" onClick={e => { e.stopPropagation(); setEditing({ collection: c, busy: false, error: null }) }} aria-label={`Edit ${c.name}`}>
                    <img src="/images/icons/icon_rename.png" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Edit</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={e => { e.stopPropagation(); setSharing(c) }} aria-label={`Access for ${c.name}`}>
                    <img src="/images/icons/icon_share.svg" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Access</span>
                  </button>
                  <button type="button" className="maze-item-action btn-danger-outline" onClick={e => { e.stopPropagation(); setDeleting({ collection: c, busy: false, error: null }) }} aria-label={`Delete ${c.name}`}>
                    <img src="/images/icons/icon_delete.png" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Delete</span>
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </main>
    </div>
  )
}
