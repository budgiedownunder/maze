import { useRef, useState } from 'react'
import { useToken } from '../context/AuthContext'
import { uploadGameImage, deleteGameImage, type GameImageKind } from '../api/client'
import { WorkshopThumbnail } from './WorkshopListPage'

// Mirrors the account avatar's accepted formats + 2 MiB cap, so an oversize /
// wrong-type file is rejected before a pointless upload round-trip.
const ACCEPTED_TYPES = ['image/png', 'image/jpeg']
const MAX_BYTES = 2 * 1024 * 1024

const PLACEHOLDER: Record<GameImageKind, string> = {
  definition: '/images/workshop/workshop-game.svg',
  collection: '/images/workshop/workshop-game-collection.svg',
}

interface Props {
  kind: GameImageKind
  /** The saved entity's id — the control only makes sense once it exists. */
  id: string
  /** Current image marker (`imageUpdatedAt`); drives the preview + Change/Remove. */
  imageUpdatedAt?: string | null
  /** Reports the new marker (or `null` when removed) so the surrounding list row
   *  can refresh its thumbnail. */
  onChange: (imageUpdatedAt: string | null) => void
}

/**
 * Inline image control for a saved game / collection, mirroring the account-page
 * avatar: a preview (the entity's own image, or the same placeholder art the row
 * shows when it has none) plus Change / Remove. The image is a **separate** server
 * resource — Change/Remove take effect immediately via the image endpoints,
 * independent of the enclosing editor's Save/Cancel — so it is never part of the
 * form and never affects the Save button's enabled state.
 */
export function GameImageEditor({ kind, id, imageUpdatedAt, onChange }: Props) {
  const token = useToken() ?? ''
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [status, setStatus] = useState<string | null>(null)

  async function handleFile(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    // Reset the input so picking the *same* file again re-fires `change`.
    e.target.value = ''
    if (!file) return
    setError(null)
    setStatus(null)
    if (!ACCEPTED_TYPES.includes(file.type)) {
      setError('Please choose a PNG or JPEG image.')
      return
    }
    if (file.size > MAX_BYTES) {
      setError('Image must be 2 MB or smaller.')
      return
    }
    setBusy(true)
    try {
      const { imageUpdatedAt: marker } = await uploadGameImage(token, kind, id, file)
      onChange(marker)
      setStatus('Image updated')
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to upload image.')
    } finally {
      setBusy(false)
    }
  }

  async function handleRemove() {
    setError(null)
    setStatus(null)
    setBusy(true)
    try {
      await deleteGameImage(token, kind, id)
      onChange(null)
      setStatus('Image removed')
    } catch (ex: unknown) {
      setError((ex as { message?: string }).message ?? 'Failed to remove image.')
    } finally {
      setBusy(false)
    }
  }

  return (
    <section className="game-image-editor" aria-label="Image">
      <button
        type="button"
        className="game-image-editor-preview"
        disabled={busy}
        onClick={() => fileInputRef.current?.click()}
        aria-label={imageUpdatedAt ? 'Change image' : 'Upload image'}
        title={imageUpdatedAt ? 'Change image' : 'Upload image'}
      >
        <WorkshopThumbnail baseSrc={PLACEHOLDER[kind]} visibility="private" showMarker={false} imageSubject={{ kind, id, imageUpdatedAt }} />
      </button>
      <div className="game-image-editor-actions">
        <div className="game-image-editor-buttons">
          <button type="button" className="btn-gray" disabled={busy} onClick={() => fileInputRef.current?.click()}>
            {busy ? 'Working…' : imageUpdatedAt ? 'Change' : 'Upload'}
          </button>
          {imageUpdatedAt && (
            <button type="button" className="btn-link" disabled={busy} onClick={() => void handleRemove()}>
              Remove
            </button>
          )}
        </div>
        <input ref={fileInputRef} type="file" accept="image/png,image/jpeg" onChange={handleFile} hidden />
        {error && <p role="alert" className="error-msg">{error}</p>}
        {status && !error && <p role="status" className="game-image-editor-status">{status}</p>}
      </div>
    </section>
  )
}
