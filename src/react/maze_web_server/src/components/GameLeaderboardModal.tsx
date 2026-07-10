import { useEffect, useState } from 'react'
import { getGameDefinition } from '../api/client'
import { useBusyCursor } from '../hooks/useBusyCursor'
import { Leaderboard } from './Leaderboard'

interface Props {
  token: string
  // The game whose board is shown; `name` titles the modal.
  gameId: string
  name: string
  currentUserId?: string
  onClose: () => void
}

// Read-only quick-view of a single game's leaderboard, for the workshop games
// list. The board itself is the shared `<Leaderboard>` component; this only
// resolves the game's challenge key (the play-fetch computes it) and frames it.
// An unpublished (private) game has no tracked board yet, so we say so instead.
export function GameLeaderboardModal({ token, gameId, name, currentUserId, onClose }: Props) {
  const [state, setState] = useState<{ challenge: string; tracked: boolean } | null>(null)
  const [error, setError] = useState<string | null>(null)

  const isLoading = state == null && error == null
  useBusyCursor(isLoading)

  useEffect(() => {
    let cancelled = false
    getGameDefinition(token, gameId)
      .then(def => { if (!cancelled) setState({ challenge: def.challengeKey, tracked: def.leaderboardTracked }) })
      .catch((ex: unknown) => { if (!cancelled) setError((ex as Error).message || 'Failed to load leaderboard.') })
    return () => { cancelled = true }
  }, [token, gameId])

  return (
    <div role="dialog" aria-modal="true" aria-label={`Leaderboard: ${name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm">
        <h2 className="modal-title">Leaderboard: {name}</h2>

        {isLoading && <p aria-label="Loading">Loading…</p>}
        {error && <p role="alert" className="error-msg">{error}</p>}
        {state != null && !state.tracked && (
          <p>This game isn’t published, so it has no leaderboard yet.</p>
        )}
        {state != null && state.tracked && (
          <Leaderboard
            token={token}
            subject={{ challenge: state.challenge }}
            currentUserId={currentUserId}
            showPlayer
          />
        )}

        <div className="modal-actions-row" style={{ marginTop: '1.25rem' }}>
          <button type="button" className="btn-gray" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  )
}
