import { useCallback, useEffect, useRef, useState } from 'react'
import type { ScoreBoardResponse, ScoreEntry } from '../types/api'

const PAGE_SIZE = 20

export interface UseScoreBoard {
  rows: ScoreEntry[]
  isLoading: boolean
  isLoadingMore: boolean
  error: string | null
  hasMore: boolean
  loadMore: () => void
}

interface LoadedBoard {
  key: string
  rows: ScoreEntry[]
  hasMore: boolean
}

// Accumulates a paged leaderboard / history. `key` identifies the current board
// (subject + ordering); when it changes the view resets and the first page
// loads. `loadMore` appends the next page. State is keyed by `key`, so a key
// change resets the view via derivation (not synchronous setState in an
// effect); results from a superseded board are discarded by key. `fetchPage`'s
// identity may churn between renders, so it's read through a ref and only `key`
// drives reloads.
export function useScoreBoard(
  key: string | null,
  fetchPage: (limit: number, offset: number) => Promise<ScoreBoardResponse>,
): UseScoreBoard {
  const [loaded, setLoaded] = useState<LoadedBoard | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: string; message: string } | null>(null)
  const [loadingMoreKey, setLoadingMoreKey] = useState<string | null>(null)

  const fetchPageRef = useRef(fetchPage)
  useEffect(() => { fetchPageRef.current = fetchPage }, [fetchPage])
  const loadingMoreRef = useRef(false)

  const message = (err: unknown) => (err as Error).message || 'Failed to load scores'

  // Derive the view from the keyed state, so switching boards resets it without
  // calling setState during render or synchronously in an effect.
  const error = errorFor != null && errorFor.key === key ? errorFor.message : null
  const current = loaded != null && loaded.key === key ? loaded : null
  const rows = current?.rows ?? []
  const hasMore = current?.hasMore ?? false
  const isLoading = key != null && current == null && error == null
  const isLoadingMore = key != null && loadingMoreKey === key

  // Latest loaded board, so loadMore can read the current row count (offset)
  // without depending on the derived `rows` array's churning identity.
  const loadedRef = useRef<LoadedBoard | null>(null)
  useEffect(() => { loadedRef.current = loaded }, [loaded])

  // First page — only sets state in async callbacks.
  useEffect(() => {
    if (key == null) return
    let cancelled = false
    fetchPageRef.current(PAGE_SIZE, 0)
      .then(page => { if (!cancelled) setLoaded({ key, rows: page.scores, hasMore: page.has_more }) })
      .catch(err => { if (!cancelled) setErrorFor({ key, message: message(err) }) })
    return () => { cancelled = true }
  }, [key])

  const loadMore = useCallback(() => {
    if (key == null || loadingMoreRef.current) return
    loadingMoreRef.current = true
    setLoadingMoreKey(key)
    const offset =
      loadedRef.current != null && loadedRef.current.key === key ? loadedRef.current.rows.length : 0
    fetchPageRef.current(PAGE_SIZE, offset)
      .then(page => {
        setLoaded(prev =>
          prev != null && prev.key === key
            ? { key, rows: [...prev.rows, ...page.scores], hasMore: page.has_more }
            : prev,
        )
      })
      .catch(err => setErrorFor({ key, message: message(err) }))
      .finally(() => {
        loadingMoreRef.current = false
        setLoadingMoreKey(curr => (curr === key ? null : curr))
      })
  }, [key])

  return { rows, isLoading, isLoadingMore, error, hasMore, loadMore }
}
