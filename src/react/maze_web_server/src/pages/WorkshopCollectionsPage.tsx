import { useEffect, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { GameCollectionFormModal } from '../components/GameCollectionFormModal'
import { useToken, useAuth } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { createGameCollection, listGameCollections } from '../api/client'
import { accessLabel } from '../utils/gameDefinitions'
import type { GameCollection } from '../types/api'

// A one-line collection summary — game count and access tier — shown under the
// name.
function collectionSummary(c: GameCollection): string {
  const count = c.items.length
  const games = count === 1 ? '1 game' : `${count} games`
  return `${games} · ${accessLabel(c.visibility)}`
}

// The workshop's Collections area: the caller's own game collections. The list
// endpoint merges own + shared + public + curated, so we filter to the caller's
// own here. Membership editing (open/reorder) and the row actions
// (edit/access/delete) land in the following steps.
export function WorkshopCollectionsPage() {
  const token = useToken()
  const { profile } = useAuth()

  // Load state keyed by the refresh counter, so a refresh resets the view by
  // derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; collections: GameCollection[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const [creating, setCreating] = useState<{ busy: boolean; error: string | null } | null>(null)

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

  useBusyCursor(!!creating?.busy)

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
              <li key={c.id} className="game-list-item" style={{ cursor: 'default' }}>
                <div className="maze-item-text">
                  <span className="maze-item-name" title={c.name}>{c.name}</span>
                  <span className="maze-item-subtitle">{collectionSummary(c)}</span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </main>
    </div>
  )
}
