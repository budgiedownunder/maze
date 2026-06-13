import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ScoreList } from '../../src/components/ScoreList'
import type { ScoreEntry } from '../../src/types/api'

function row(over: Partial<ScoreEntry>): ScoreEntry {
  return {
    id: 'id',
    user_id: 'u',
    maze_id: null,
    challenge: 'easy:1',
    score: 5,
    elapsed_ms: 42137,
    recorded_at: '2025-04-01T12:00:00Z',
    ...over,
  }
}

const base = {
  isLoading: false,
  isLoadingMore: false,
  error: null,
  hasMore: false,
  onLoadMore: vi.fn(),
}

describe('ScoreList', () => {
  it('renders a player column with usernames and "You" for the caller', () => {
    render(
      <ScoreList
        {...base}
        showPlayer
        currentUserId="me"
        rows={[
          row({ id: '1', user_id: 'other', username: 'alice', elapsed_ms: 31204, score: 9 }),
          row({ id: '2', user_id: 'me', username: 'bob', elapsed_ms: 42137, score: 7 }),
        ]}
      />,
    )
    expect(screen.getByText('alice')).toBeInTheDocument()
    expect(screen.getByText('You')).toBeInTheDocument()
    // The caller's row is not labelled by their username.
    expect(screen.queryByText('bob')).not.toBeInTheDocument()
    // Formatted time appears.
    expect(screen.getByText('0:42.137')).toBeInTheDocument()
  })

  it('never renders a raw user_id', () => {
    const { container } = render(
      <ScoreList
        {...base}
        showPlayer
        currentUserId="me"
        rows={[row({ id: '1', user_id: 'super-secret-user-id', username: 'alice' })]}
      />,
    )
    expect(container.textContent).not.toContain('super-secret-user-id')
  })

  it('omits the player column when showPlayer is false', () => {
    render(
      <ScoreList
        {...base}
        showPlayer={false}
        currentUserId="me"
        rows={[row({ id: '1', user_id: 'me' })]}
      />,
    )
    expect(screen.queryByText('Player')).not.toBeInTheDocument()
    expect(screen.queryByText('You')).not.toBeInTheDocument()
  })

  it('labels the timestamp column "Completed" and shows a date-time', () => {
    render(<ScoreList {...base} showPlayer rows={[row({ recorded_at: '2025-04-01T12:00:00Z' })]} />)
    expect(screen.getByRole('columnheader', { name: 'Completed' })).toBeInTheDocument()
    // The cell renders the localised date+time of the recorded_at instant.
    const expected = new Date('2025-04-01T12:00:00Z').toLocaleString()
    expect(screen.getByText(expected)).toBeInTheDocument()
  })

  it('shows an empty state when there are no rows', () => {
    render(<ScoreList {...base} showPlayer rows={[]} />)
    expect(screen.getByText(/no win scores yet/i)).toBeInTheDocument()
  })

  it('shows Load more only when hasMore and fires the callback', async () => {
    const onLoadMore = vi.fn()
    const { rerender } = render(
      <ScoreList {...base} showPlayer rows={[row({ id: '1' })]} hasMore={false} onLoadMore={onLoadMore} />,
    )
    expect(screen.queryByRole('button', { name: /load more/i })).not.toBeInTheDocument()

    rerender(
      <ScoreList {...base} showPlayer rows={[row({ id: '1' })]} hasMore onLoadMore={onLoadMore} />,
    )
    await userEvent.click(screen.getByRole('button', { name: /load more/i }))
    expect(onLoadMore).toHaveBeenCalledOnce()
  })
})
