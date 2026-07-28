import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { http, HttpResponse } from 'msw'
import { GameLeaderboardModal } from '../../src/components/GameLeaderboardModal'
import { resetMockGameDefinitions } from '../../src/mocks/handlers'
import { server } from '../../src/mocks/server'

function renderModal() {
  return render(
    <GameLeaderboardModal token="test-token" gameId="d1" name="Tower" currentUserId="me" onClose={() => {}} />,
  )
}

beforeEach(() => {
  resetMockGameDefinitions()
})

describe('GameLeaderboardModal', () => {
  it('renders the shared board for a published (tracked) game', async () => {
    server.use(
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'public', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', challengeKey: 'def:d1', leaderboardTracked: true })),
    )
    renderModal()
    // The default /scores handler returns a couple of named rows.
    await waitFor(() => expect(screen.getByText('alice')).toBeInTheDocument())
    expect(screen.getByRole('tab', { name: /fastest time/i })).toBeInTheDocument()
  })

  it('shows the owner-only board for a private game', async () => {
    server.use(
      http.get('/api/v1/game-definitions/d1', () =>
        HttpResponse.json({ id: 'd1', ownerId: 'o1', name: 'Tower', visibility: 'private', seed: 1, rotation: 'static', config: {}, createdAt: 'x', updatedAt: 'x', challengeKey: 'def:d1', leaderboardTracked: true })),
    )
    renderModal()
    await waitFor(() => expect(screen.getByText('alice')).toBeInTheDocument())
  })

  it('surfaces a load failure', async () => {
    server.use(http.get('/api/v1/game-definitions/d1', () => new HttpResponse('gone', { status: 404 })))
    renderModal()
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('gone'))
  })
})
