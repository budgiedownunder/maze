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
// Every game has a board — a private game's is simply owner-only — so it shows
// the board directly (empty when no one has scored yet).
export function GameLeaderboardModal({ token, gameId, name, currentUserId, onClose }: Props) {
  const [challenge, setChallenge] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const isLoading = challenge == null && error == null
  useBusyCursor(isLoading)

  useEffect(() => {
    let cancelled = false
    getGameDefinition(token, gameId)
      .then(def => { if (!cancelled) setChallenge(def.challengeKey) })
      .catch((ex: unknown) => { if (!cancelled) setError((ex as Error).message || 'Failed to load leaderboard.') })
    return () => { cancelled = true }
  }, [token, gameId])

  return (
    <div role="dialog" aria-modal="true" aria-label={`Leaderboard: ${name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-sm modal-capped">
        <h2 className="modal-title">Leaderboard: {name}</h2>

        <div className="leaderboard-modal-body">
          {isLoading && <p aria-label="Loading">Loading…</p>}
          {error && <p role="alert" className="error-msg">{error}</p>}
          {challenge != null && (
            <Leaderboard
              token={token}
              subject={{ challenge }}
              currentUserId={currentUserId}
              showPlayer
            />
          )}
        </div>

        <div className="modal-actions-row" style={{ marginTop: '1.25rem' }}>
          <button type="button" className="btn-gray" onClick={onClose}>Close</button>
        </div>
      </div>
    </div>
  )
}
