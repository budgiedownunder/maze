import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GameImageEditor } from '../../src/components/GameImageEditor'
import { uploadGameImage, deleteGameImage, fetchGameImage } from '../../src/api/client'

vi.mock('../../src/api/client', () => ({
  uploadGameImage: vi.fn(),
  deleteGameImage: vi.fn(),
  fetchGameImage: vi.fn(), // used by the preview thumbnail's cache
}))
vi.mock('../../src/context/AuthContext', () => ({ useToken: () => 'test-token' }))

const mockUpload = vi.mocked(uploadGameImage)
const mockDelete = vi.mocked(deleteGameImage)
const mockFetch = vi.mocked(fetchGameImage)

function fileInput(container: HTMLElement): HTMLInputElement {
  return container.querySelector('input[type="file"]') as HTMLInputElement
}

beforeEach(async () => {
  vi.clearAllMocks()
  // The preview thumbnail always resolves through fetchGameImage; give it a
  // default so a test with an existing image doesn't crash the cache.
  mockFetch.mockResolvedValue(new Blob([new Uint8Array([1])], { type: 'image/png' }))
  globalThis.URL.createObjectURL = vi.fn(() => 'blob:mock-url')
  globalThis.URL.revokeObjectURL = vi.fn()
  const { resetImageCache } = await import('../../src/utils/imageCache')
  resetImageCache()
})

describe('GameImageEditor', () => {
  it('shows an Upload button (no Remove) and the placeholder when there is no image', () => {
    const { container } = render(<GameImageEditor kind="definition" id="g1" onChange={vi.fn()} />)
    expect(screen.getByRole('button', { name: 'Upload' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /remove/i })).not.toBeInTheDocument()
    expect(container.querySelector('.game-thumb-base')?.getAttribute('src')).toBe('/images/workshop/workshop-game.svg')
  })

  it('uploads a picked file, reports the new marker, and shows a status message', async () => {
    mockUpload.mockResolvedValue({ imageUpdatedAt: '2026-03-05T00:00:00Z' })
    const onChange = vi.fn()
    const { container } = render(<GameImageEditor kind="definition" id="g1" onChange={onChange} />)

    const file = new File([new Uint8Array([1, 2, 3])], 'pic.png', { type: 'image/png' })
    await userEvent.upload(fileInput(container), file)

    await waitFor(() => expect(mockUpload).toHaveBeenCalledWith('test-token', 'definition', 'g1', file))
    expect(onChange).toHaveBeenCalledWith('2026-03-05T00:00:00Z')
    expect(await screen.findByText('Image updated')).toBeInTheDocument()
  })

  it('rejects a non-image file with an error and no upload', async () => {
    const onChange = vi.fn()
    const { container } = render(<GameImageEditor kind="collection" id="c1" onChange={onChange} />)

    const bad = new File(['x'], 'notes.txt', { type: 'text/plain' })
    // Bypass the input's `accept` filter so the component's own type guard runs.
    await userEvent.upload(fileInput(container), bad, { applyAccept: false })

    expect(await screen.findByText(/PNG or JPEG/i)).toBeInTheDocument()
    expect(mockUpload).not.toHaveBeenCalled()
    expect(onChange).not.toHaveBeenCalled()
  })

  it('shows Change + Remove when an image exists, and removing reports null', async () => {
    mockDelete.mockResolvedValue()
    const onChange = vi.fn()
    render(<GameImageEditor kind="collection" id="c1" imageUpdatedAt="2026-03-01T00:00:00Z" onChange={onChange} />)

    expect(screen.getByRole('button', { name: 'Change' })).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: /remove/i }))

    await waitFor(() => expect(mockDelete).toHaveBeenCalledWith('test-token', 'collection', 'c1'))
    expect(onChange).toHaveBeenCalledWith(null)
    expect(await screen.findByText('Image removed')).toBeInTheDocument()
  })

  it('opens the file picker when the preview image is clicked', async () => {
    const clickSpy = vi.spyOn(HTMLInputElement.prototype, 'click')
    render(<GameImageEditor kind="definition" id="g1" onChange={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: 'Upload image' }))
    expect(clickSpy).toHaveBeenCalled()
    clickSpy.mockRestore()
  })

  it('renders the existing image in the preview via the cache', async () => {
    mockFetch.mockResolvedValue(new Blob([new Uint8Array([1])], { type: 'image/png' }))
    const { container } = render(<GameImageEditor kind="definition" id="g1" imageUpdatedAt="2026-03-01T00:00:00Z" onChange={vi.fn()} />)
    await waitFor(() => expect(container.querySelector('.game-thumb-base')?.getAttribute('src')).toBe('blob:mock-url'))
    expect(mockFetch).toHaveBeenCalledWith('test-token', 'definition', 'g1', '2026-03-01T00:00:00Z')
  })
})
