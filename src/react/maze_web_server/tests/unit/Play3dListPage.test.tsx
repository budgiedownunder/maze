import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dListPage, type Play3dCard } from '../../src/components/Play3dListPage'

// Isolation tests for the reusable Play-3D browse shell: paging, the client-side
// filter, empty/error states, and card action wiring — driven by a stub loader,
// no routing or real endpoints.

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: 'u1', username: 'testuser', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

interface Item { id: string; name: string; description?: string }

function cardOf(actions: Play3dCard['actions'] = [{ key: 'play', label: 'Play', ariaLabel: 'Play', variant: 'primary', onClick: () => {} }]) {
  return (i: Item): Play3dCard => ({ name: i.name, description: i.description, actions: actions.map(a => ({ ...a, ariaLabel: `${a.label} ${i.name}` })) })
}

interface ListOverrides {
  title?: string
  fetchPage?: (token: string, limit: number, offset: number) => Promise<{ items: Item[]; hasMore: boolean }>
  getId?: (i: Item) => string
  card?: (i: Item) => Play3dCard
  searchText?: (i: Item) => string
  emptyText?: string
  errorText?: string
}

function renderList(overrides: ListOverrides) {
  const props = {
    title: 'Featured',
    fetchPage: () => Promise.resolve({ items: [] as Item[], hasMore: false }),
    getId: (i: Item) => i.id,
    card: cardOf(),
    emptyText: 'Nothing here yet.',
    errorText: 'Failed to load',
    ...overrides,
  }
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <Play3dListPage<Item> {...props} />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

describe('Play3dListPage', () => {
  it('renders a gallery card per item with name, description and actions', async () => {
    renderList({
      fetchPage: () => Promise.resolve({ items: [
        { id: 'g1', name: 'Night Climb', description: 'A 3-level ascent' },
        { id: 'g2', name: 'Quick Picks' },
      ], hasMore: false }),
    })
    await waitFor(() => expect(screen.getByText('Night Climb')).toBeInTheDocument())
    expect(screen.getByText('A 3-level ascent')).toBeInTheDocument()
    expect(screen.getByText('Quick Picks')).toBeInTheDocument()
    // Each card has its Play action.
    expect(screen.getByRole('button', { name: 'Play Night Climb' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Play Quick Picks' })).toBeInTheDocument()
  })

  it('fires a card action’s onClick', async () => {
    const onPlay = vi.fn()
    renderList({
      fetchPage: () => Promise.resolve({ items: [{ id: 'g1', name: 'Night Climb' }], hasMore: false }),
      card: cardOf([{ key: 'play', label: 'Play', ariaLabel: 'Play', variant: 'primary', onClick: onPlay }]),
    })
    await userEvent.click(await screen.findByRole('button', { name: 'Play Night Climb' }))
    expect(onPlay).toHaveBeenCalledTimes(1)
  })

  it('pages with a Load more button', async () => {
    const all: Item[] = Array.from({ length: 25 }, (_, i) => ({ id: `g${i}`, name: `Game ${i}` }))
    renderList({
      fetchPage: (_t, limit, offset) => Promise.resolve({ items: all.slice(offset, offset + limit), hasMore: offset + limit < all.length }),
    })
    await waitFor(() => expect(screen.getByText('Game 0')).toBeInTheDocument())
    // First page = 20 cards, Load more present.
    expect(screen.getAllByRole('listitem')).toHaveLength(20)
    await userEvent.click(screen.getByRole('button', { name: 'Load more' }))
    await waitFor(() => expect(screen.getAllByRole('listitem')).toHaveLength(25))
    expect(screen.queryByRole('button', { name: 'Load more' })).toBeNull()
  })

  it('filters the loaded items client-side when searchText is given', async () => {
    renderList({
      fetchPage: () => Promise.resolve({ items: [
        { id: 'g1', name: 'Alpha' },
        { id: 'g2', name: 'Beta' },
      ], hasMore: false }),
      searchText: (i: Item) => i.name,
    })
    await waitFor(() => expect(screen.getByText('Alpha')).toBeInTheDocument())
    await userEvent.type(screen.getByLabelText('Filter…'), 'bet')
    expect(screen.getByText('Beta')).toBeInTheDocument()
    expect(screen.queryByText('Alpha')).toBeNull()
    // A non-matching query shows the no-matches message, not the empty state.
    await userEvent.clear(screen.getByLabelText('Filter…'))
    await userEvent.type(screen.getByLabelText('Filter…'), 'zzz')
    expect(screen.getByText('No matches.')).toBeInTheDocument()
  })

  it('shows the empty state when there are no items', async () => {
    renderList({ fetchPage: () => Promise.resolve({ items: [], hasMore: false }) })
    await waitFor(() => expect(screen.getByText('Nothing here yet.')).toBeInTheDocument())
  })

  it('surfaces a load error', async () => {
    renderList({ fetchPage: () => Promise.reject(new Error('boom')) })
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('boom'))
  })

  it('renders a disabled action (the coming-soon state) that does not fire', async () => {
    const onPlay = vi.fn()
    renderList({
      fetchPage: () => Promise.resolve({ items: [{ id: 'c1', name: 'Difficulty' }], hasMore: false }),
      card: cardOf([{ key: 'play', label: 'Play', ariaLabel: 'Play', variant: 'primary', onClick: onPlay, disabled: true, title: 'Coming soon' }]),
    })
    const play = await screen.findByRole('button', { name: 'Play Difficulty' })
    expect(play).toBeDisabled()
    expect(play).toHaveAttribute('title', 'Coming soon')
    await userEvent.click(play, { pointerEventsCheck: 0 })
    expect(onPlay).not.toHaveBeenCalled()
  })
})
