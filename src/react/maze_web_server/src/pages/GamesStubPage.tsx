import { useEffect, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { GameDefinitionEditor } from '../components/GameDefinitionEditor'
import { useToken } from '../context/AuthContext'
import { createGameDefinition, getGameDefinition, getLeaderboard, listGameDefinitions, reshuffleGameDefinition, updateGameDefinition } from '../api/client'
import { DEFINITION_DEFAULTS, parseDefinitionConfig, type DefinitionFormState } from '../utils/definitionConfig'
import { launchDefinitionPreview } from '../utils/definitionPreview'
import type { GameDefinition, GameDefinitionRequest } from '../types/api'

// A minimal 3D-games surface: the definitions the caller can see, a New game
// button that opens the definition editor as a create wizard, and a per-row Edit
// that opens the same editor in tabs mode over the loaded definition.
export function GamesStubPage() {
  const token = useToken()

  // The fetched list and any load failure are keyed by the refresh counter, so a
  // refresh resets the view by derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; definitions: GameDefinition[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const [isCreating, setIsCreating] = useState(false)
  const [editing, setEditing] = useState<{ id: string; form: DefinitionFormState; hasScores: boolean } | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const error = errorFor != null && errorFor.key === refreshCount ? errorFor.message : null
  const current = loaded != null && loaded.key === refreshCount ? loaded : null
  const definitions = current?.definitions ?? []
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

  async function handleCreate(request: GameDefinitionRequest) {
    try {
      await createGameDefinition(token!, request)
      closeEditor()
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to create game.')
    }
  }

  async function handleEdit(id: string) {
    setActionError(null)
    try {
      const def = await getGameDefinition(token!, id)
      const form = parseDefinitionConfig(def.config, {
        name: def.name,
        description: def.description,
        visibility: def.visibility,
        rotation: def.rotation,
      })
      // Whether the board already has scores drives the stronger reshuffle-confirm
      // wording; a tracked board that is empty (or an untracked draft) is "no scores".
      const board = def.leaderboardTracked
        ? await getLeaderboard(token!, { challenge: def.challengeKey, limit: 1 })
        : null
      const hasScores = (board?.scores.length ?? 0) > 0
      // The play-fetch splices an *effective* seed into `config` (date-mixed for
      // a Daily game), so hydrate the seed from the record's own field instead —
      // otherwise a Save would bake one day's layout into the stored config.
      setEditing({ id, form: { ...form, seed: def.seed }, hasScores })
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to load game.')
    }
  }

  async function handleSave(request: GameDefinitionRequest) {
    if (!editing) return
    try {
      await updateGameDefinition(token!, editing.id, request)
      closeEditor()
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setActionError((ex as { message?: string }).message ?? 'Failed to save game.')
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
      <AppHeader title="Games">
        <button type="button" className="btn-primary" onClick={() => setIsCreating(true)}>
          New game
        </button>
      </AppHeader>
      <main>
        {actionError && <p className="error-msg" role="alert">{actionError}</p>}
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && <p className="error-msg" role="alert">{error}</p>}
        {!isLoading && !error && definitions.length === 0 && <p>No games yet.</p>}
        {!isLoading && !error && definitions.length > 0 && (
          <ul>
            {definitions.map(d => (
              <li key={d.id}>
                <span>{d.name}</span>
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() => void handleEdit(d.id)}
                  aria-label={`Edit ${d.name}`}
                >
                  Edit
                </button>
              </li>
            ))}
          </ul>
        )}
      </main>
    </div>
  )
}
