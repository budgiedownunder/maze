import { describe, it, expect, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useScoreBoard } from '../../src/hooks/useScoreBoard'
import type { ScoreBoardResponse, ScoreEntry } from '../../src/types/api'

function entry(id: string): ScoreEntry {
  return { id, user_id: 'u', maze_id: null, challenge: 'c', score: 1, elapsed_ms: 1, recorded_at: '2025-01-01T00:00:00Z' }
}
function page(ids: string[], hasMore: boolean): ScoreBoardResponse {
  return { scores: ids.map(entry), limit: 20, offset: 0, has_more: hasMore }
}

describe('useScoreBoard', () => {
  it('loads the first page', async () => {
    const fetchPage = vi.fn().mockResolvedValueOnce(page(['1', '2'], true))
    const { result } = renderHook(() => useScoreBoard('k1', fetchPage))

    expect(result.current.isLoading).toBe(true)
    await waitFor(() => expect(result.current.isLoading).toBe(false))
    expect(result.current.rows.map(r => r.id)).toEqual(['1', '2'])
    expect(result.current.hasMore).toBe(true)
    expect(fetchPage).toHaveBeenCalledWith(20, 0)
  })

  it('appends the next page on loadMore (offset = current row count)', async () => {
    const fetchPage = vi.fn()
      .mockResolvedValueOnce(page(['1'], true))
      .mockResolvedValueOnce(page(['2'], false))
    const { result } = renderHook(() => useScoreBoard('k1', fetchPage))
    await waitFor(() => expect(result.current.rows).toHaveLength(1))

    act(() => { result.current.loadMore() })
    await waitFor(() => expect(result.current.rows.map(r => r.id)).toEqual(['1', '2']))
    expect(result.current.hasMore).toBe(false)
    expect(fetchPage).toHaveBeenNthCalledWith(2, 20, 1)
  })

  it('resets the view when the key changes', async () => {
    const fetchPage = vi.fn()
      .mockResolvedValueOnce(page(['a'], false))
      .mockResolvedValueOnce(page(['b'], false))
    const { result, rerender } = renderHook(({ k }) => useScoreBoard(k, fetchPage), {
      initialProps: { k: 'k1' },
    })
    await waitFor(() => expect(result.current.rows.map(r => r.id)).toEqual(['a']))

    rerender({ k: 'k2' })
    // Stale rows are dropped immediately (derived from the new key).
    expect(result.current.rows).toEqual([])
    expect(result.current.isLoading).toBe(true)
    await waitFor(() => expect(result.current.rows.map(r => r.id)).toEqual(['b']))
  })

  it('does not fetch when the key is null', () => {
    const fetchPage = vi.fn()
    const { result } = renderHook(() => useScoreBoard(null, fetchPage))
    expect(result.current.isLoading).toBe(false)
    expect(result.current.rows).toEqual([])
    expect(fetchPage).not.toHaveBeenCalled()
  })

  it('surfaces a fetch error', async () => {
    const fetchPage = vi.fn().mockRejectedValueOnce(new Error('boom'))
    const { result } = renderHook(() => useScoreBoard('k1', fetchPage))
    await waitFor(() => expect(result.current.error).toBe('boom'))
    expect(result.current.isLoading).toBe(false)
  })
})
