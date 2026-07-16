import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { http, HttpResponse } from 'msw'
import { ThemeProvider } from '../../src/context/ThemeProvider'
import { Play3dFeaturedPage } from '../../src/pages/Play3dFeaturedPage'
import { server } from '../../src/mocks/server'
import type { FeaturedGameItem, GameCollection, GameDefinition } from '../../src/types/api'
import { launchDefinition } from '../../src/utils/play3dLaunch'

vi.mock('../../src/utils/play3dLaunch', () => ({ launchDefinition: vi.fn() }))

vi.mock('../../src/context/AuthContext', async () => {
  const actual = await vi.importActual('../../src/context/AuthContext')
  return {
    ...actual,
    useToken: () => 'test-token',
    useAuth: () => ({ isLoading: false, isAuthenticated: true, profile: { id: 'me', username: 'me', is_admin: false }, login: vi.fn(), logout: vi.fn() }),
  }
})

function def(over: Partial<GameDefinition> & { id: string; name: string }): GameDefinition {
  return { ownerId: 'admin', visibility: 'curated', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', ...over }
}
function col(over: Partial<GameCollection> & { id: string; name: string }): GameCollection {
  return { ownerId: 'admin', visibility: 'curated', playMode: 'arcade', items: [], createdAt: 'x', updatedAt: 'x', ...over }
}
function defItem(d: GameDefinition): FeaturedGameItem { return { kind: 'definition', ownerUsername: 'admin', definition: d } }
function colItem(c: GameCollection): FeaturedGameItem { return { kind: 'collection', ownerUsername: 'admin', collection: c } }

function featuredOf(...items: FeaturedGameItem[]) {
  return http.get('/api/v1/featured-game-items', ({ request }) => {
    const url = new URL(request.url)
    const limit = Number(url.searchParams.get('limit') ?? '20')
    const offset = Number(url.searchParams.get('offset') ?? '0')
    return HttpResponse.json({ items: items.slice(offset, offset + limit), limit, offset, hasMore: offset + limit < items.length })
  })
}

function renderPage() {
  return render(
    <MemoryRouter>
      <ThemeProvider>
        <Play3dFeaturedPage />
      </ThemeProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => vi.clearAllMocks())

describe('Play3dFeaturedPage', () => {
  it('renders a card per featured item with Play (and Leaderboard for games)', async () => {
    server.use(featuredOf(
      defItem(def({ id: 'd1', name: 'Night Climb', description: 'A 3-level ascent' })),
      colItem(col({ id: 'c1', name: 'Difficulty', items: [
        { definitionId: 'e', sortOrder: 0 }, { definitionId: 't', sortOrder: 1 },
      ] })),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Night Climb')).toBeInTheDocument())
    expect(screen.getByText('A 3-level ascent')).toBeInTheDocument()
    expect(screen.getByText('Difficulty')).toBeInTheDocument()

    // A game card has Play + Leaderboard, each with its icon.
    const play = screen.getByRole('button', { name: 'Play Night Climb' })
    expect(play.querySelector('img')).toHaveAttribute('src', '/images/icons/icon_play_3d.png')
    const board = screen.getByRole('button', { name: 'Leaderboard for Night Climb' })
    expect(board.querySelector('img')).toHaveAttribute('src', '/images/icons/icon_leaderboard.svg')
    // A collection card has no Leaderboard.
    expect(screen.queryByRole('button', { name: 'Leaderboard for Difficulty' })).toBeNull()
  })

  it('Play launches a game definition', async () => {
    server.use(featuredOf(defItem(def({ id: 'd1', name: 'Night Climb' }))))
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Night Climb' }))
    expect(launchDefinition).toHaveBeenCalledWith('d1')
  })

  it('Play on a single-game collection launches its sole accessible member', async () => {
    server.use(
      featuredOf(colItem(col({ id: 'c1', name: 'Solo Set', items: [{ definitionId: 'only', sortOrder: 0 }] }))),
      // The detail resolves the accessible members (here, the one game).
      http.get('/api/v1/game-collections/c1', () =>
        HttpResponse.json({ ...col({ id: 'c1', name: 'Solo Set' }), definitions: [def({ id: 'only', name: 'Only' })] })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Solo Set' }))
    await waitFor(() => expect(launchDefinition).toHaveBeenCalledWith('only'))
  })

  it('a single-game collection whose only member is inaccessible guards instead of launching', async () => {
    server.use(
      featuredOf(colItem(col({ id: 'c1', name: 'Gated Set', items: [{ definitionId: 'secret', sortOrder: 0 }] }))),
      // The sole member is another user's non-public game → filtered out of the
      // detail, so there is nothing the viewer can play.
      http.get('/api/v1/game-collections/c1', () =>
        HttpResponse.json({ ...col({ id: 'c1', name: 'Gated Set' }), definitions: [] })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Gated Set' }))
    const dialog = await screen.findByRole('dialog', { name: 'Play: Gated Set' })
    expect(within(dialog).getByText(/no games you can play/i)).toBeInTheDocument()
    expect(within(dialog).getByRole('button', { name: 'Play' })).toBeDisabled()
    expect(launchDefinition).not.toHaveBeenCalled()
  })

  it('Play on a multi-game Arcade collection opens a picker (default first) and launches the chosen game', async () => {
    const alpha = def({ id: 'a', name: 'Alpha', description: 'the first one' })
    const beta = def({ id: 'b', name: 'Beta' })
    server.use(
      featuredOf(colItem(col({ id: 'c1', name: 'Arcade Set', playMode: 'arcade', items: [
        { definitionId: 'a', sortOrder: 0 }, { definitionId: 'b', sortOrder: 1 },
      ] }))),
      http.get('/api/v1/game-collections/c1', () =>
        HttpResponse.json({ ...col({ id: 'c1', name: 'Arcade Set' }), definitions: [alpha, beta] })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Arcade Set' }))

    const dialog = await screen.findByRole('dialog', { name: 'Play: Arcade Set' })
    expect(within(dialog).getByText('Alpha')).toBeInTheDocument()
    expect(within(dialog).getByText('the first one')).toBeInTheDocument()
    expect(within(dialog).getByText('Beta')).toBeInTheDocument()
    // The first game is the default selection.
    expect(within(dialog).getByRole('radio', { name: /Alpha/ })).toBeChecked()

    // Choosing Beta then Play launches it.
    await userEvent.click(within(dialog).getByRole('radio', { name: /Beta/ }))
    await userEvent.click(within(dialog).getByRole('button', { name: 'Play' }))
    expect(launchDefinition).toHaveBeenCalledWith('b')
  })

  it('the Arcade picker guards an empty (no accessible games) collection', async () => {
    server.use(
      featuredOf(colItem(col({ id: 'c1', name: 'Locked Set', playMode: 'arcade', items: [
        { definitionId: 'x', sortOrder: 0 }, { definitionId: 'y', sortOrder: 1 },
      ] }))),
      // The viewer can access none of the members → detail returns no definitions.
      http.get('/api/v1/game-collections/c1', () =>
        HttpResponse.json({ ...col({ id: 'c1', name: 'Locked Set' }), definitions: [] })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Locked Set' }))
    const dialog = await screen.findByRole('dialog', { name: 'Play: Locked Set' })
    expect(await within(dialog).findByText(/no games you can play/i)).toBeInTheDocument()
    expect(within(dialog).getByRole('button', { name: 'Play' })).toBeDisabled()
  })

  function campaignSetup(completed: string[]) {
    const g1 = def({ id: 'g1', name: 'Alpha' })
    const g2 = def({ id: 'g2', name: 'Beta' })
    const g3 = def({ id: 'g3', name: 'Gamma' })
    server.use(
      featuredOf(colItem(col({ id: 'c1', name: 'Campaign', playMode: 'campaign', items: [
        { definitionId: 'g1', sortOrder: 0 }, { definitionId: 'g2', sortOrder: 1 }, { definitionId: 'g3', sortOrder: 2 },
      ] }))),
      http.get('/api/v1/game-collections/c1', () =>
        HttpResponse.json({ ...col({ id: 'c1', name: 'Campaign' }), definitions: [g1, g2, g3] })),
      http.post('/api/v1/scores/me/completed', () => HttpResponse.json({ completed })),
    )
  }

  it('a multi-game Campaign collection opens an ordered modal with complete/current/locked progress', async () => {
    // Alpha (def:g1) is completed → Beta is current → Gamma is locked.
    campaignSetup(['def:g1'])
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Play: Campaign' })

    expect(within(dialog).getByText('✓ Completed')).toBeInTheDocument()
    const alpha = within(dialog).getByRole('button', { name: 'Replay Alpha' })
    const beta = within(dialog).getByRole('button', { name: 'Play Beta' })
    const gamma = within(dialog).getByRole('button', { name: 'Locked: Gamma' })
    expect(alpha).toBeEnabled()
    expect(beta).toBeEnabled()
    expect(gamma).toBeDisabled()

    // A completed level replays; Continue plays the current (first-incomplete) level.
    await userEvent.click(alpha)
    expect(launchDefinition).toHaveBeenCalledWith('g1')
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }))
    expect(launchDefinition).toHaveBeenCalledWith('g2')
  })

  it('an all-complete campaign shows the done state with Continue disabled', async () => {
    campaignSetup(['def:g1', 'def:g2', 'def:g3'])
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Play Campaign' }))
    const dialog = await screen.findByRole('dialog', { name: 'Play: Campaign' })
    expect(within(dialog).getByText(/completed this campaign/i)).toBeInTheDocument()
    expect(within(dialog).getByRole('button', { name: 'Completed' })).toBeDisabled()
    // Every level is replayable (none locked).
    expect(within(dialog).getByRole('button', { name: 'Replay Alpha' })).toBeEnabled()
    expect(within(dialog).getByRole('button', { name: 'Replay Gamma' })).toBeEnabled()
  })

  it('Leaderboard opens the game leaderboard modal', async () => {
    server.use(
      featuredOf(defItem(def({ id: 'd1', name: 'Night Climb' }))),
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ ...def({ id: 'd1', name: 'Night Climb' }), challengeKey: 'def:d1', leaderboardTracked: true })),
      http.get('/api/v1/scores', () => HttpResponse.json({ scores: [], limit: 20, offset: 0, has_more: false })),
    )
    renderPage()
    await userEvent.click(await screen.findByRole('button', { name: 'Leaderboard for Night Climb' }))
    expect(await screen.findByRole('dialog', { name: 'Leaderboard: Night Climb' })).toBeInTheDocument()
  })

  it('filters featured items client-side', async () => {
    server.use(featuredOf(
      defItem(def({ id: 'd1', name: 'Alpha' })),
      defItem(def({ id: 'd2', name: 'Beta' })),
    ))
    renderPage()
    await waitFor(() => expect(screen.getByText('Alpha')).toBeInTheDocument())
    await userEvent.type(screen.getByLabelText('Filter featured…'), 'bet')
    expect(screen.getByText('Beta')).toBeInTheDocument()
    expect(screen.queryByText('Alpha')).toBeNull()
  })

  it('shows the empty state', async () => {
    server.use(featuredOf())
    renderPage()
    await waitFor(() => expect(screen.getByText('No featured games or collections yet.')).toBeInTheDocument())
  })
})
