import { useEffect, useState } from 'react'
import { AppHeader } from '../components/AppHeader'
import { GameDefinitionEditor } from '../components/GameDefinitionEditor'
import { useToken } from '../context/AuthContext'
import { createGameDefinition, listGameDefinitions } from '../api/client'
import { DEFINITION_DEFAULTS } from '../utils/definitionConfig'
import type { GameDefinition, GameDefinitionRequest } from '../types/api'

// A minimal 3D-games surface: the definitions the caller can see, and a New game
// button that opens the definition editor as a create wizard.
export function GamesStubPage() {
  const token = useToken()

  // The fetched list and any load failure are keyed by the refresh counter, so a
  // refresh resets the view by derivation rather than by setState in an effect.
  const [refreshCount, setRefreshCount] = useState(0)
  const [loaded, setLoaded] = useState<{ key: number; definitions: GameDefinition[] } | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: number; message: string } | null>(null)

  const [isCreating, setIsCreating] = useState(false)
  const [createError, setCreateError] = useState<string | null>(null)

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

  async function handleCreate(request: GameDefinitionRequest) {
    try {
      await createGameDefinition(token!, request)
      setIsCreating(false)
      setCreateError(null)
      setRefreshCount(c => c + 1)
    } catch (ex: unknown) {
      setCreateError((ex as { message?: string }).message ?? 'Failed to create game.')
    }
  }

  return (
    <div className="games-page">
      {isCreating && (
        <GameDefinitionEditor
          mode="wizard"
          title="New game"
          initialForm={DEFINITION_DEFAULTS}
          onSubmit={request => void handleCreate(request)}
          onCancel={() => { setIsCreating(false); setCreateError(null) }}
        />
      )}
      <AppHeader title="Games">
        <button type="button" className="btn-primary" onClick={() => setIsCreating(true)}>
          New game
        </button>
      </AppHeader>
      <main>
        {createError && <p className="error-msg" role="alert">{createError}</p>}
        {isLoading && <p aria-label="Loading">Loading…</p>}
        {!isLoading && error && <p className="error-msg" role="alert">{error}</p>}
        {!isLoading && !error && definitions.length === 0 && <p>No games yet.</p>}
        {!isLoading && !error && definitions.length > 0 && (
          <ul>
            {definitions.map(d => (
              <li key={d.id}>{d.name}</li>
            ))}
          </ul>
        )}
      </main>
    </div>
  )
}
