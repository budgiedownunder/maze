import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { ManageSharesModal, type ShareSubject } from '../../src/components/ManageSharesModal'
import { resetMockShares } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'
import type { Visibility } from '../../src/utils/gameDefinitions'

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return { ...actual, useToken: () => 'test-token' }
})

// Stub the avatar to expose its props (the real fetch/object-URL behaviour is
// covered by Avatar.test); each grantee row's avatar becomes queryable by id.
vi.mock('../../src/components/Avatar', () => ({
  Avatar: ({ userId, avatarUpdatedAt }: { userId: string; avatarUpdatedAt?: string | null }) => (
    <span data-testid={`avatar-${userId}`} data-marker={avatarUpdatedAt ?? ''} />
  ),
}))

beforeEach(() => {
  vi.clearAllMocks()
  resetMockShares()
})

const defSubject: ShareSubject = { kind: 'definition', id: 'd1', name: 'Tower', ownerId: 'owner-1' }

function renderModal(over: { subject?: ShareSubject; visibility?: Visibility; isAdmin?: boolean } = {}) {
  const onSetVisibility = vi.fn<(v: Visibility) => Promise<void>>().mockResolvedValue(undefined)
  const onSaved = vi.fn()
  const onClose = vi.fn()
  render(
    <ManageSharesModal
      subject={over.subject ?? defSubject}
      visibility={over.visibility ?? 'shared'}
      isAdmin={over.isAdmin ?? false}
      onSetVisibility={onSetVisibility}
      onSaved={onSaved}
      onClose={onClose}
    />,
  )
  return { onSetVisibility, onSaved, onClose }
}

describe('ManageSharesModal', () => {
  it('shows the access tiers (current one checked); Featured only for admins', () => {
    renderModal({ visibility: 'private' })
    expect(screen.getByRole('heading', { name: /^Access:/ })).toHaveTextContent('Tower')
    expect(screen.getByRole('radio', { name: /Just me/ })).toBeChecked()
    expect(screen.getByRole('radio', { name: /Specific people/ })).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: /Everyone/ })).toBeInTheDocument()
    expect(screen.queryByRole('radio', { name: /Featured/ })).toBeNull()
  })

  it('offers the Featured tier to an admin', () => {
    renderModal({ isAdmin: true })
    expect(screen.getByRole('radio', { name: /Featured/ })).toBeInTheDocument()
  })

  it('stages an added user and drops them from the picker; removing un-stages', async () => {
    renderModal() // Specific people
    await waitFor(() => expect(screen.getByText('No one added yet.')).toBeInTheDocument())

    await userEvent.type(screen.getByLabelText('Add user'), 'an')
    expect(await screen.findByRole('button', { name: 'Add ann' })).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Add ann' }))
    expect(screen.getByRole('button', { name: 'Remove ann' })).toBeInTheDocument()
    expect(screen.getByLabelText('Add user')).toHaveValue('')

    // Searching again offers anna but not the already-staged ann.
    await userEvent.type(screen.getByLabelText('Add user'), 'an')
    expect(await screen.findByRole('button', { name: 'Add anna' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Add ann' })).toBeNull()

    await userEvent.click(screen.getByRole('button', { name: 'Remove ann' }))
    await waitFor(() => expect(screen.getByText('No one added yet.')).toBeInTheDocument())
  })

  it("excludes the game's owner from the picker", async () => {
    renderModal({ subject: { kind: 'definition', id: 'd1', name: 'Tower', ownerId: 'user-ann' } })
    await userEvent.type(screen.getByLabelText('Add user'), 'an')
    expect(await screen.findByRole('button', { name: 'Add anna' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Add ann' })).toBeNull()
  })

  it('Save commits the staged share list and the tier, then calls onSaved', async () => {
    let putBody: unknown
    server.use(
      http.put('/api/v1/game-definitions/:id/shares', async ({ request }) => {
        putBody = await request.json()
        return HttpResponse.json({ grantees: [] })
      }),
    )
    const { onSetVisibility, onSaved } = renderModal()
    await waitFor(() => expect(screen.getByText('No one added yet.')).toBeInTheDocument())
    await userEvent.type(screen.getByLabelText('Add user'), 'bob')
    await userEvent.click(await screen.findByRole('button', { name: 'Add bob' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(onSaved).toHaveBeenCalled())
    expect(putBody).toEqual({ userIds: ['user-bob'] })
    expect(onSetVisibility).toHaveBeenCalledWith('shared')
  })

  it('switching to Everyone hides the picker; Save clears the list and sets public', async () => {
    let putBody: unknown
    server.use(
      http.put('/api/v1/game-definitions/:id/shares', async ({ request }) => {
        putBody = await request.json()
        return HttpResponse.json({ grantees: [] })
      }),
    )
    const { onSetVisibility } = renderModal({ visibility: 'private' })
    await userEvent.click(screen.getByRole('radio', { name: /Everyone/ }))
    expect(screen.queryByLabelText('Add user')).toBeNull()

    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(onSetVisibility).toHaveBeenCalledWith('public'))
    expect(putBody).toEqual({ userIds: [] })
  })

  it('saving Specific people with no one staged persists it as private', async () => {
    let putBody: unknown
    server.use(
      http.get('/api/v1/game-definitions/:id/shares', () =>
        HttpResponse.json({ grantees: [{ id: 'user-bob', username: 'bob' }] }),
      ),
      http.put('/api/v1/game-definitions/:id/shares', async ({ request }) => {
        putBody = await request.json()
        return HttpResponse.json({ grantees: [] })
      }),
    )
    const { onSetVisibility } = renderModal()
    await userEvent.click(await screen.findByRole('button', { name: 'Remove bob' }))
    await waitFor(() => expect(screen.getByText('No one added yet.')).toBeInTheDocument())
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(onSetVisibility).toHaveBeenCalledWith('private'))
    expect(putBody).toEqual({ userIds: [] })
  })

  it('Cancel dismisses without saving', async () => {
    const { onClose, onSetVisibility } = renderModal()
    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(onClose).toHaveBeenCalled()
    expect(onSetVisibility).not.toHaveBeenCalled()
  })

  it('surfaces a save failure and keeps the modal open', async () => {
    server.use(
      http.put('/api/v1/game-definitions/:id/shares', () => new HttpResponse('nope', { status: 500 })),
    )
    const { onSaved } = renderModal()
    await userEvent.type(screen.getByLabelText('Add user'), 'ann')
    await userEvent.click(await screen.findByRole('button', { name: 'Add ann' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('nope'))
    expect(onSaved).not.toHaveBeenCalled()
  })

  it('drives the collection share endpoint for a collection subject', async () => {
    let putPath: string | null = null
    server.use(
      http.put('/api/v1/game-collections/:id/shares', async ({ request }) => {
        putPath = new URL(request.url).pathname
        return HttpResponse.json({ grantees: [] })
      }),
    )
    renderModal({ subject: { kind: 'collection', id: 'c1', name: 'Campaign', ownerId: 'owner-1' } })
    await userEvent.type(screen.getByLabelText('Add user'), 'cleo')
    await userEvent.click(await screen.findByRole('button', { name: 'Add cleo' }))
    await userEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(putPath).toBe('/api/v1/game-collections/c1/shares'))
  })

  it('renders an avatar per loaded grantee, carrying the marker only when present', async () => {
    server.use(
      http.get('/api/v1/game-definitions/:id/shares', () =>
        HttpResponse.json({ grantees: [
          { id: 'user-bob', username: 'bob', avatar_updated_at: '2026-01-01T00:00:00Z' },
          { id: 'user-ann', username: 'ann' },
        ] }),
      ),
    )
    renderModal()
    await waitFor(() => expect(screen.getByRole('button', { name: 'Remove bob' })).toBeInTheDocument())
    expect(screen.getByTestId('avatar-user-bob').getAttribute('data-marker')).not.toBe('')
    expect(screen.getByTestId('avatar-user-ann').getAttribute('data-marker')).toBe('')
  })

  it('hints to keep typing when the lookup reports more matches than the page', async () => {
    server.use(
      http.get('/api/v1/users/lookup', () =>
        HttpResponse.json({ users: [{ id: 'u1', username: 'aaa' }], limit: 8, offset: 0, has_more: true }),
      ),
    )
    renderModal()
    await userEvent.type(screen.getByLabelText('Add user'), 'a')
    expect(await screen.findByText('More matches — keep typing to narrow.')).toBeInTheDocument()
  })
})
