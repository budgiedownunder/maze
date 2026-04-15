import { renderHook, waitFor } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../mocks/server'
import { AppFeaturesProvider, useAppFeatures } from './AppFeaturesContext'

describe('AppFeaturesContext', () => {
  it('fetches features on mount', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.json({ allow_signup: false }))
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.allow_signup).toBe(false)
    })
  })

  it('fails open when fetch fails', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.error())
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.allow_signup).toBe(true)
    })
  })
})
