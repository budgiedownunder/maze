import { useEffect, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { GameDefinitionEditor } from '../components/GameDefinitionEditor'
import { PromptModal } from '../components/PromptModal'
import { ManageSharesModal } from '../components/ManageSharesModal'
import { ConfirmModal } from '../components/ConfirmModal'
import { GameLeaderboardModal } from '../components/GameLeaderboardModal'
import { useToken, useAuth } from '../context/AuthContext'
import { useBusyCursor } from '../hooks/useBusyCursor'
import {
  createGameDefinition,
  deleteGameDefinition,
  getGameDefinition,
  getLeaderboard,
  listGameDefinitions,
  reshuffleGameDefinition,
  updateGameDefinition,
} from '../api/client'
import { DEFINITION_DEFAULTS, parseDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import { launchDefinitionPreview } from '../utils/definitionPreview'
import { launchDefinition } from '../utils/play3dLaunch'
import { accessLabel, reshuffleConfirmMessage } from '../utils/gameDefinitions'
import type { GameDefinition, GameDefinitionRequest, GamePlayResponse } from '../types/api'

// The workshop's Games area: the caller's own game definitions, each with the
// full lifecycle of actions (play, leaderboard, edit, reshuffle, duplicate,
// manage sharing, delete) plus a New game create flow. The list endpoint merges
// own + shared + public + curated, so we filter to the caller's own here.
export function WorkshopGamesPage() {
  const token = useToken()
  const { profile } = useAuth()

  // The fetched list and any load failure are keyed by the refresh counter, so a
  // refresh resets the view by derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; definitions: GameDefinition[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const [isCreating, setIsCreating] = useState(false)
  const [editing, setEditing] = useState<{ id: string; form: DefinitionFormState; hasScores: boolean } | null>(null)
  const [duplicating, setDuplicating] = useState<{ source: GameDefinition; error: string | null; busy: boolean } | null>(null)
  const [sharing, setSharing] = useState<GameDefinition | null>(null)
  const [viewingBoard, setViewingBoard] = useState<GameDefinition | null>(null)
  const [reshuffling, setReshuffling] = useState<{ def: GameDefinition; hasScores: boolean; busy: boolean; error: string | null } | null>(null)
  const [deleting, setDeleting] = useState<{ def: GameDefinition; busy: boolean; error: string | null } | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  // True while a button action's request is in flight before its modal opens
  // (edit / reshuffle load) or while a create/save is committing — the confirm /
  // prompt / share modals show their own wait cursor once open. Drives the global
  // busy cursor so the page never looks frozen.
  const [actionBusy, setActionBusy] = useState(false)
  useBusyCursor(actionBusy)

  const error = errorFor != null && errorFor.key === refreshCount ? errorFor.message : null
  const current = loaded != null && loaded.key === refreshCount ? loaded : null
  // The caller's own definitions, minus any they've featured: a curated game is
  // managed from the admin Features area, not here — otherwise the default
  // admin's seeded curated games would leak into their My Games list.
  const games = (current?.definitions ?? []).filter(d => d.ownerId === profile?.id && d.visibility !== 'curated')
  const isLoading = current == null && error == null

  useEffect(() => {
    if (!token) return
    let cancelled = false
    const key = refreshCount
    listGameDefinitions(token)
      .then(page => { if (!cancelled) setLoaded({ key, definitions: page.definitions }) })
      .catch(ex => {
        if (!cancelled) setErrorFor({ key, message: (ex as Error).message || 'Failed to load games' })
      })
    return () => { cancelled = true }
  }, [token, refreshCount])

  function closeEditor() {
    setIsCreating(false)
    setEditing(null)
    setActionError(null)
  }

  // Whether a definition's board already has scores drives the stronger
  // reshuffle-confirm wording; a tracked board that is empty (or an untracked
  // draft) is "no scores". The play-fetch computes the challenge key.
  async function hasScores(def: GamePlayResponse): Promise<boolean> {
    const board = def.leaderboardTracked
      ? await getLeaderboard(token!, { challenge: def.challengeKey, limit: 1 })
      : null
    return (board?.scores.length ?? 0) > 0
  }

  async function handleCreate(request: GameDefinitionRequest) {
    setActionBusy(true)
    try {
      await createGameDefinition(token!, request)
      closeEditor()
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to create game.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleEdit(id: string) {
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
      // The play-fetch splices an *effective* seed into `config` (date-mixed for
      // a Daily game), so hydrate the seed from the record's own field instead —
      // otherwise a Save would bake one day's layout into the stored config.
      setEditing({ id, form: { ...form, seed: def.seed }, hasScores: await hasScores(def) })
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to load game.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleSave(request: GameDefinitionRequest) {
    if (!editing) return
    setActionBusy(true)
    try {
      await updateGameDefinition(token!, editing.id, request)
      closeEditor()
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to save game.')
    } finally {
      setActionBusy(false)
    }
  }

  // Open the reshuffle confirm, first resolving whether the board has scores so
  // the dialog can warn appropriately (a scored board is wiped).
  async function openReshuffle(id: string) {
    setActionError(null)
    setActionBusy(true)
    try {
      const def = await getGameDefinition(token!, id)
      setReshuffling({ def, hasScores: await hasScores(def), busy: false, error: null })
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to load game.')
    } finally {
      setActionBusy(false)
    }
  }

  async function handleConfirmReshuffle() {
    if (!reshuffling) return
    setReshuffling(r => (r ? { ...r, busy: true, error: null } : r))
    try {
      await reshuffleGameDefinition(token!, reshuffling.def.id)
      setReshuffling(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to reshuffle.'
      setReshuffling(r => (r ? { ...r, busy: false, error: message } : r))
    }
  }

  async function handleConfirmDelete() {
    if (!deleting) return
    setDeleting(d => (d ? { ...d, busy: true, error: null } : d))
    try {
      await deleteGameDefinition(token!, deleting.def.id)
      setDeleting(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to delete game.'
      setDeleting(d => (d ? { ...d, busy: false, error: message } : d))
    }
  }

  // A copy can't reuse an existing name (the server enforces unique per-owner and
  // would 409); pre-check against the visible list for a friendlier message.
  function validateDuplicateName(name: string): string | null {
    return games.some(d => d.name.toLowerCase() === name.toLowerCase())
      ? 'A game with that name already exists.'
      : null
  }

  async function handleConfirmDuplicate(name: string) {
    if (!duplicating) return
    setDuplicating(d => (d ? { ...d, busy: true, error: null } : d))
    try {
      const source = duplicating.source
      // Re-post the source's stored config verbatim under a new name; the server
      // mints a fresh id + seed on create, so the copy is an independent draft.
      // It is created Private (no leaderboard) whatever the source's tier, so an
      // author can iterate without touching the original's live board.
      await createGameDefinition(token!, {
        name,
        description: source.description ?? null,
        visibility: 'private',
        rotation: source.rotation,
        config: source.config,
      })
      setDuplicating(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to duplicate game.'
      setDuplicating(d => (d ? { ...d, busy: false, error: message } : d))
    }
  }

  return (
    <div className="games-page">
      {isCreating && (
        <GameDefinitionEditor
          mode="wizard"
          title="New Game"
          initialForm={DEFINITION_DEFAULTS}
          onSubmit={request => void handleCreate(request)}
          onCancel={closeEditor}
          // A new game has no server-minted seed yet → unseeded preview.
          onPreview={config => launchDefinitionPreview(config, false)}
        />
      )}
      {editing && (
        <GameDefinitionEditor
          mode="tabs"
          title="Edit Game"
          initialForm={editing.form}
          onSubmit={request => void handleSave(request)}
          onCancel={closeEditor}
          hasScores={editing.hasScores}
          onReshuffle={() => reshuffleGameDefinition(token!, editing.id).then(d => d.seed)}
          // A saved definition has a real seed → the preview is the actual layout.
          onPreview={config => launchDefinitionPreview(config, true)}
        />
      )}
      {duplicating && (
        <PromptModal
          title="Duplicate Game"
          label="Name"
          initialValue={`Copy of ${duplicating.source.name}`}
          confirmLabel="Duplicate"
          validate={validateDuplicateName}
          isLoading={duplicating.busy}
          error={duplicating.error}
          onConfirm={name => void handleConfirmDuplicate(name)}
          onCancel={() => setDuplicating(null)}
        />
      )}
      {sharing && (
        <ManageSharesModal
          subject={{ kind: 'definition', id: sharing.id, name: sharing.name, ownerId: sharing.ownerId }}
          onClose={() => setSharing(null)}
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
      {reshuffling && (
        <ConfirmModal
          title="Reshuffle Layout"
          message={reshuffleConfirmMessage(reshuffling.hasScores)}
          confirmLabel="Reshuffle"
          isDangerous={reshuffling.hasScores}
          isLoading={reshuffling.busy}
          error={reshuffling.error}
          onConfirm={() => void handleConfirmReshuffle()}
          onCancel={() => setReshuffling(null)}
        />
      )}
      {deleting && (
        <ConfirmModal
          title="Delete Game"
          message={`Delete “${deleting.def.name}”? This permanently removes the game${deleting.def.visibility !== 'private' ? ' and its leaderboard' : ''}. This cannot be undone.`}
          confirmLabel="Delete"
          isDangerous
          isLoading={deleting.busy}
          error={deleting.error}
          onConfirm={() => void handleConfirmDelete()}
          onCancel={() => setDeleting(null)}
        />
      )}
      <AppHeader title="My Games">
        <button type="button" className="btn-primary" onClick={() => setIsCreating(true)}>
          + New game
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
        {actionError && <p className="error-msg" role="alert">{actionError}</p>}
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && <p className="error-msg" role="alert">{error}</p>}
        {!isLoading && !error && games.length === 0 && <p>No games yet.</p>}
        {!isLoading && !error && games.length > 0 && (
          <ul className="game-list">
            {games.map(d => (
              <li key={d.id} className="game-list-item">
                <div className="maze-item-text">
                  <span className="maze-item-name" title={d.name}>{d.name}</span>
                  <span className="maze-item-subtitle">
                    <span className="access-badge">{accessLabel(d.visibility)}</span>
                  </span>
                </div>
                <div className="game-item-actions">
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => launchDefinition(d.id)} aria-label={`Play ${d.name}`}>
                    <img src="/images/icons/icon_play_3d.png" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Play</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => setViewingBoard(d)} aria-label={`Leaderboard for ${d.name}`}>
                    <img src="/images/icons/icon_leaderboard.svg" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Leaderboard</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => void handleEdit(d.id)} aria-label={`Edit ${d.name}`}>
                    <img src="/images/icons/icon_rename.png" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Edit</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => void openReshuffle(d.id)} aria-label={`Reshuffle ${d.name}`}>
                    <img src="/images/icons/icon_reshuffle.svg" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Reshuffle</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => setDuplicating({ source: d, error: null, busy: false })} aria-label={`Duplicate ${d.name}`}>
                    <img src="/images/icons/icon_duplicate.png" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Duplicate</span>
                  </button>
                  <button type="button" className="maze-item-action btn-secondary" onClick={() => setSharing(d)} aria-label={`Share ${d.name}`}>
                    <img src="/images/icons/icon_share.svg" alt="" aria-hidden="true" />
                    <span className="maze-item-action-label">Share</span>
                  </button>
                  <button type="button" className="maze-item-action btn-danger-outline" onClick={() => setDeleting({ def: d, busy: false, error: null })} aria-label={`Delete ${d.name}`}>
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
