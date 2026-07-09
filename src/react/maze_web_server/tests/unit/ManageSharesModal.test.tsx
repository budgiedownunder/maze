import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { ManageSharesModal } from '../../src/components/ManageSharesModal'
import { resetMockShares } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return { ...actual, useToken: () => 'test-token' }
})

beforeEach(() => {
  vi.clearAllMocks()
  resetMockShares()
})

const defSubject = { kind: 'definition' as const, id: 'd1', name: 'Tower' }

describe('ManageSharesModal', () => {
  it('shows the empty state, then a searched user can be added and is no longer offered', async () => {
    render(<ManageSharesModal subject={defSubject} onClose={vi.fn()} />)

    // The title names the subject; the grantee group starts empty.
    expect(screen.getByRole('heading', { name: /^Share:/ })).toHaveTextContent('Tower')
    await waitFor(() => expect(screen.getByText('No one has access yet.')).toBeInTheDocument())

    // Typing a prefix searches the lookup; "an" matches ann + anna.
    await userEvent.type(screen.getByLabelText('Add user'), 'an')
    expect(await screen.findByRole('button', { name: 'Add ann' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Add anna' })).toBeInTheDocument()

    // Grant ann → it appears in the grantee list and the picker input clears.
    await userEvent.click(screen.getByRole('button', { name: 'Add ann' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Remove ann' })).toBeInTheDocument())
    expect(screen.getByLabelText('Add user')).toHaveValue('')
    expect(screen.queryByText('No one has access yet.')).toBeNull()

    // Searching "an" again offers anna but not the already-granted ann.
    await userEvent.type(screen.getByLabelText('Add user'), 'an')
    expect(await screen.findByRole('button', { name: 'Add anna' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Add ann' })).toBeNull()
  })

  it('revokes a grantee', async () => {
    render(<ManageSharesModal subject={defSubject} onClose={vi.fn()} />)
    await waitFor(() => expect(screen.getByText('No one has access yet.')).toBeInTheDocument())

    await userEvent.type(screen.getByLabelText('Add user'), 'bob')
    await userEvent.click(await screen.findByRole('button', { name: 'Add bob' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Remove bob' })).toBeInTheDocument())

    await userEvent.click(screen.getByRole('button', { name: 'Remove bob' }))
    await waitFor(() => expect(screen.getByText('No one has access yet.')).toBeInTheDocument())
  })

  it('drives the collection share endpoints for a collection subject', async () => {
    render(<ManageSharesModal subject={{ kind: 'collection', id: 'c1', name: 'Campaign' }} onClose={vi.fn()} />)
    await waitFor(() => expect(screen.getByText('No one has access yet.')).toBeInTheDocument())

    await userEvent.type(screen.getByLabelText('Add user'), 'cleo')
    await userEvent.click(await screen.findByRole('button', { name: 'Add cleo' }))
    await waitFor(() => expect(screen.getByRole('button', { name: 'Remove cleo' })).toBeInTheDocument())
  })

  it('hints to keep typing when the lookup reports more matches than the page', async () => {
    server.use(
      http.get('/api/v1/users/lookup', () =>
        HttpResponse.json({ users: [{ id: 'u1', username: 'aaa' }], limit: 8, offset: 0, has_more: true }),
      ),
    )
    render(<ManageSharesModal subject={defSubject} onClose={vi.fn()} />)
    await userEvent.type(screen.getByLabelText('Add user'), 'a')
    expect(await screen.findByText('More matches — keep typing to narrow.')).toBeInTheDocument()
  })

  it('surfaces a grant failure and keeps the modal open', async () => {
    server.use(
      http.put('/api/v1/game-definitions/:id/shares', () => new HttpResponse('nope', { status: 500 })),
    )
    render(<ManageSharesModal subject={defSubject} onClose={vi.fn()} />)
    await waitFor(() => expect(screen.getByText('No one has access yet.')).toBeInTheDocument())

    await userEvent.type(screen.getByLabelText('Add user'), 'ann')
    await userEvent.click(await screen.findByRole('button', { name: 'Add ann' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('nope'))
    expect(screen.queryByRole('button', { name: 'Remove ann' })).toBeNull()
  })

  it('Close dismisses the modal', async () => {
    const onClose = vi.fn()
    render(<ManageSharesModal subject={defSubject} onClose={onClose} />)
    await userEvent.click(screen.getByRole('button', { name: 'Close' }))
    expect(onClose).toHaveBeenCalled()
  })
})
