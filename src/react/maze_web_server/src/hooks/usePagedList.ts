import { useCallback, useEffect, useRef, useState } from 'react'

const PAGE_SIZE = 20

export interface PagedList<T> {
  items: T[]
  isLoading: boolean
  isLoadingMore: boolean
  error: string | null
  hasMore: boolean
  loadMore: () => void
  // Patch one already-loaded item in place (by id) without a refetch.
  patchItem: (id: string, patch: Partial<T>) => void
}

interface LoadedPage<T> {
  key: string
  items: T[]
  hasMore: boolean
}

// Accumulates a server-paged list. `key` identifies the current query (e.g.
// scope + search term + a refresh counter); when it changes the view resets and
// the first page loads. `loadMore` appends the next page. State is keyed by
// `key`, so a key change resets the view via derivation (not synchronous
// setState in an effect) and results from a superseded query are discarded.
// Generalises `useLeaderboard` over the item type, adding `getId` for in-place
// patching. `fetchPage`'s identity may churn between renders, so it is read
// through a ref and only `key` drives reloads.
export function usePagedList<T>(
  key: string | null,
  fetchPage: (limit: number, offset: number) => Promise<{ items: T[]; hasMore: boolean }>,
  getId: (item: T) => string,
  errorText = 'Failed to load',
): PagedList<T> {
  const [loaded, setLoaded] = useState<LoadedPage<T> | null>(null)
  const [errorFor, setErrorFor] = useState<{ key: string; message: string } | null>(null)
  const [loadingMoreKey, setLoadingMoreKey] = useState<string | null>(null)

  const fetchPageRef = useRef(fetchPage)
  const getIdRef = useRef(getId)
  useEffect(() => { fetchPageRef.current = fetchPage; getIdRef.current = getId })
  const loadingMoreRef = useRef(false)

  // Derive the view from the keyed state, so a query change resets it without
  // setState during render or synchronously in an effect.
  const error = errorFor != null && errorFor.key === key ? errorFor.message : null
  const current = loaded != null && loaded.key === key ? loaded : null
  const items = current?.items ?? []
  const hasMore = current?.hasMore ?? false
  const isLoading = key != null && current == null && error == null
  const isLoadingMore = key != null && loadingMoreKey === key

  // Latest loaded page, so loadMore can read the current count (offset) without
  // depending on the derived `items` array's churning identity.
  const loadedRef = useRef<LoadedPage<T> | null>(null)
  useEffect(() => { loadedRef.current = loaded }, [loaded])

  // First page — only sets state in async callbacks.
  useEffect(() => {
    if (key == null) return
    let cancelled = false
    fetchPageRef.current(PAGE_SIZE, 0)
      .then(page => { if (!cancelled) setLoaded({ key, items: page.items, hasMore: page.hasMore }) })
      .catch(err => { if (!cancelled) setErrorFor({ key, message: (err as Error).message || errorText }) })
    return () => { cancelled = true }
  }, [key, errorText])

  const loadMore = useCallback(() => {
    if (key == null || loadingMoreRef.current) return
    loadingMoreRef.current = true
    setLoadingMoreKey(key)
    const offset =
      loadedRef.current != null && loadedRef.current.key === key ? loadedRef.current.items.length : 0
    fetchPageRef.current(PAGE_SIZE, offset)
      .then(page => {
        setLoaded(prev =>
          prev != null && prev.key === key
            ? { key, items: [...prev.items, ...page.items], hasMore: page.hasMore }
            : prev,
        )
      })
      .catch(err => setErrorFor({ key, message: (err as Error).message || errorText }))
      .finally(() => {
        loadingMoreRef.current = false
        setLoadingMoreKey(curr => (curr === key ? null : curr))
      })
  }, [key, errorText])

  const patchItem = useCallback((id: string, patch: Partial<T>) => {
    setLoaded(prev =>
      prev == null
        ? prev
        : { ...prev, items: prev.items.map(it => (getIdRef.current(it) === id ? { ...it, ...patch } : it)) },
    )
  }, [])

  return { items, isLoading, isLoadingMore, error, hasMore, loadMore, patchItem }
}
