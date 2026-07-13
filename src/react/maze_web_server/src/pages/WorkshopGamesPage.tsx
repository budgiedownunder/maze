import { useRef, useState } from 'react'
import { WorkshopListPage, WorkshopThumbnail, type WorkshopListContext } from '../components/WorkshopListPage'
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
import { accessLabel, reshuffleConfirmMessage, type Visibility } from '../utils/gameDefinitions'
import type { GameDefinition, GameDefinitionRequest, GamePlayResponse } from '../types/api'

// A one-line game summary — level count, rotation, and access tier — shown under
// the name. Level count lives in the opaque config; missing/≤1 reads as single.
function gameSummary(d: GameDefinition): string {
  const count = Number((d.config.levels as { count?: number } | undefined)?.count) || 1
  const levels = count <= 1 ? 'Single level' : `${count} levels`
  const rotation = d.rotation === 'daily' ? 'Daily' : 'Static'
  return `${levels} · ${rotation} · ${accessLabel(d.visibility)}`
}

// The workshop's Games area: the caller's own game definitions, each with the
// full lifecycle of actions (play, leaderboard, edit, reshuffle, duplicate,
// manage sharing, delete) plus a New game create flow. The list endpoint merges
// own + shared + public + curated, so we filter to the caller's own here.
export function WorkshopGamesPage() {
  const token = useToken()
  const { profile } = useAuth()

  // The shared list's context (refresh / patch), captured once it is ready.
  const listRef = useRef<WorkshopListContext<GameDefinition> | null>(null)
  const refresh = () => listRef.current?.refresh()

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

  function closeEditor() {
    setIsCreating(false)
    setEditing(null)
    setActionError(null)
  }

  // Persist a game's access tier — a visibility-only change (the stored config is
  // sent unchanged, so the board is not reset). Driven by the access modal.
  async function setDefinitionVisibility(def: GameDefinition, visibility: Visibility): Promise<void> {
    await updateGameDefinition(token!, def.id, {
      name: def.name,
      description: def.description ?? null,
      visibility,
      rotation: def.rotation,
      config: def.config,
    })
  }

  // Re-read one game's authoritative visibility and patch just its row — used when
  // the access modal saves. Best-effort: on failure the row is left as-is and a
  // manual Refresh still corrects it.
  async function reloadRowVisibility(id: string) {
    try {
      const def = await getGameDefinition(token!, id)
      listRef.current?.patchItem(id, { visibility: def.visibility })
    } catch {
      // Ignore — the badge stays until the next load/refresh.
    }
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
      refresh()
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
      refresh()
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
      refresh()
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
      refresh()
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to delete game.'
      setDeleting(d => (d ? { ...d, busy: false, error: message } : d))
    }
  }

  // A copy can't reuse an existing name (the server enforces unique per-owner and
  // would 409); pre-check against the visible list for a friendlier message.
  function validateDuplicateName(name: string): string | null {
    const games = listRef.current?.getItems() ?? []
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
      refresh()
    } catch (ex: unknown) {
      const message = (ex as { message?: string }).message ?? 'Failed to duplicate game.'
      setDuplicating(d => (d ? { ...d, busy: false, error: message } : d))
    }
  }

  const overlays = (
    <>
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
          visibility={sharing.visibility}
          isAdmin={!!profile?.is_admin}
          onSetVisibility={v => setDefinitionVisibility(sharing, v)}
          onSaved={() => { const id = sharing.id; setSharing(null); void reloadRowVisibility(id) }}
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
    </>
  )

  return (
    <WorkshopListPage<GameDefinition>
      title="Manage Games"
      newLabel="+ New Game"
      onNew={() => setIsCreating(true)}
      load={t => listGameDefinitions(t).then(page => page.definitions)}
      // The caller's own definitions, minus any they've featured: a curated game
      // is managed from the admin Features area, not here — otherwise the default
      // admin's seeded curated games would leak into their Manage Games list.
      filter={d => d.ownerId === profile?.id && d.visibility !== 'curated'}
      getId={d => d.id}
      emptyText="No games yet."
      errorText="Failed to load games"
      onReady={ctx => { listRef.current = ctx }}
      banner={actionError ? <p className="error-msg" role="alert">{actionError}</p> : null}
      overlays={overlays}
      row={d => ({
        name: d.name,
        subtitle: gameSummary(d),
        thumbnail: <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={d.visibility} />,
        onOpen: () => void handleEdit(d.id),
        actions: [
          { key: 'edit', label: 'Edit', ariaLabel: `Edit ${d.name}`, icon: '/images/icons/icon_rename.png', onClick: () => void handleEdit(d.id) },
          { key: 'play', label: 'Play', ariaLabel: `Play ${d.name}`, icon: '/images/icons/icon_play_3d.png', onClick: () => launchDefinition(d.id) },
          { key: 'leaderboard', label: 'Leaderboard', ariaLabel: `Leaderboard for ${d.name}`, icon: '/images/icons/icon_leaderboard.svg', onClick: () => setViewingBoard(d) },
          { key: 'reshuffle', label: 'Reshuffle', ariaLabel: `Reshuffle ${d.name}`, icon: '/images/icons/icon_reshuffle.svg', onClick: () => void openReshuffle(d.id) },
          { key: 'duplicate', label: 'Duplicate', ariaLabel: `Duplicate ${d.name}`, icon: '/images/icons/icon_duplicate.png', onClick: () => setDuplicating({ source: d, error: null, busy: false }) },
          { key: 'access', label: 'Access', ariaLabel: `Access for ${d.name}`, icon: '/images/icons/icon_share.svg', onClick: () => setSharing(d) },
          { key: 'delete', label: 'Delete', ariaLabel: `Delete ${d.name}`, icon: '/images/icons/icon_delete.png', onClick: () => setDeleting({ def: d, busy: false, error: null }), variant: 'danger' },
        ],
      })}
    />
  )
}
