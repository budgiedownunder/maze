import { launchDefinition } from '../utils/play3dLaunch'
import { gameChallengeKey } from '../utils/gameDefinitions'
import { WorkshopThumbnail } from './WorkshopListPage'
import type { GameDefinition } from '../types/api'

interface Props {
  // The collection's name, for the modal title.
  name: string
  // The member games the viewer may play, ordered + access-filtered (from the
  // collection detail). At least two (a single-game collection launches directly).
  definitions: GameDefinition[]
  // The challenge keys the caller has completed (from /scores/me/completed), used
  // to derive each level's complete / current / locked state.
  completed: string[]
  onClose: () => void
}

// The Campaign collection modal: an ordered progression through the member games.
// A game is `complete` when the caller has any score on its board (globally —
// completion is shared across every collection the game is in, so a game beaten
// elsewhere shows completed here too); `current` is the first game with no score;
// everything after the current with no score is `locked`. Play launches the
// current level; completed levels are replayable; locked levels are disabled.
export function CampaignCollectionModal({ name, definitions, completed, onClose }: Props) {
  const done = definitions.map(d => new Set(completed).has(gameChallengeKey(d.id)))
  const currentIndex = done.findIndex(isDone => !isDone)
  const allComplete = currentIndex === -1

  return (
    <div role="dialog" aria-modal="true" aria-label={`Play: ${name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-md modal-capped campaign-modal">
        <h2 className="modal-title">Play: {name}</h2>
        <div className="modal-scroll-body">
          {allComplete && <p className="campaign-done">You’ve completed this campaign!</p>}
          <ol className="campaign-levels">
            {definitions.map((d, i) => {
              const isDone = done[i]
              const isCurrent = i === currentIndex
              const locked = !isDone && !isCurrent
              const state = isDone ? 'complete' : isCurrent ? 'current' : 'locked'
              const verb = locked ? 'Locked:' : isDone ? 'Replay' : 'Play'
              return (
                <li key={d.id} className={`campaign-level ${state}`}>
                  <button
                    type="button"
                    className="campaign-level-btn"
                    disabled={locked}
                    onClick={() => launchDefinition(d.id)}
                    aria-label={`${verb} ${d.name}`}
                  >
                    <span className="campaign-level-num" aria-hidden="true">{i + 1}</span>
                    <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={d.visibility} showMarker={false} />
                    <span className="campaign-level-text">
                      <span className="campaign-level-name" title={d.name}>{d.name}</span>
                      {d.description && <span className="campaign-level-desc">{d.description}</span>}
                    </span>
                    <span className={`campaign-level-status ${state}`}>
                      {isDone ? '✓ Completed' : isCurrent ? 'Play' : 'Locked'}
                    </span>
                  </button>
                </li>
              )
            })}
          </ol>
        </div>
        <div className="modal-actions-row">
          <button type="button" onClick={onClose} className="btn-gray">Cancel</button>
          <button
            type="button"
            className="btn-primary"
            disabled={allComplete}
            onClick={() => { if (currentIndex >= 0) launchDefinition(definitions[currentIndex].id) }}
          >
            {allComplete ? 'Completed' : 'Continue'}
          </button>
        </div>
      </div>
    </div>
  )
}
