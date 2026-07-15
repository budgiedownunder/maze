import { useState } from 'react'
import { launchDefinition } from '../utils/play3dLaunch'
import { WorkshopThumbnail } from './WorkshopListPage'
import type { GameDefinition } from '../types/api'

interface Props {
  // The collection's name, for the modal title.
  name: string
  // The member games the viewer may play, already hydrated + access-filtered +
  // ordered by the caller (from the collection detail). Empty ⇒ nothing to play.
  definitions: GameDefinition[]
  onClose: () => void
}

// The Arcade collection picker: an Arcade collection is a free-choice set, so
// this lists its accessible member games (image / name / description) as radio
// options — default the first — and launches the chosen one via `/game/?def=<id>`.
// The parent resolves the accessible members before opening this, so an empty
// list is guarded here (Play disabled) rather than launching a game the viewer
// can't access.
export function ArcadeCollectionModal({ name, definitions, onClose }: Props) {
  const [selectedId, setSelectedId] = useState<string | null>(definitions[0]?.id ?? null)
  const isEmpty = definitions.length === 0

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (selectedId) launchDefinition(selectedId)
  }

  return (
    <div role="dialog" aria-modal="true" aria-label={`Play: ${name}`} className="modal-overlay" style={{ zIndex: 1200 }}>
      <div className="modal modal-md modal-capped arcade-modal">
        <h2 className="modal-title">Play: {name}</h2>
        <form className="modal-form" onSubmit={handleSubmit}>
          <div className="modal-scroll-body">
            {isEmpty && <p>This collection has no games you can play.</p>}
            {!isEmpty && (
              <div role="radiogroup" aria-label="Choose a game" className="modal-radio-group">
                {definitions.map(d => (
                  <label key={d.id} className={`modal-radio arcade-pick${selectedId === d.id ? ' selected' : ''}`}>
                    <input
                      type="radio"
                      name="game"
                      value={d.id}
                      checked={selectedId === d.id}
                      onChange={() => setSelectedId(d.id)}
                    />
                    <WorkshopThumbnail baseSrc="/images/workshop/workshop-game.svg" visibility={d.visibility} showMarker={false} />
                    <span className="arcade-pick-text">
                      <span className="arcade-pick-name" title={d.name}>{d.name}</span>
                      {d.description && <span className="arcade-pick-desc">{d.description}</span>}
                    </span>
                  </label>
                ))}
              </div>
            )}
          </div>
          <div className="modal-actions-row">
            <button type="button" onClick={onClose} className="btn-gray">Cancel</button>
            <button type="submit" className="btn-primary" disabled={selectedId == null}>Play</button>
          </div>
        </form>
      </div>
    </div>
  )
}
