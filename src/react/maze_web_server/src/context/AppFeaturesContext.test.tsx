import { renderHook, waitFor } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '../mocks/server'
import { useAppFeatures } from './AppFeaturesContext'
import { AppFeaturesProvider } from './AppFeaturesProvider'

describe('AppFeaturesContext', () => {
  it('fetches features on mount', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.json({
        allow_signup: false,
        oauth_providers: [{ name: 'google', display_name: 'Google' }],
      }))
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.allow_signup).toBe(false)
    })
    expect(result.current.oauth_providers).toEqual([{ name: 'google', display_name: 'Google' }])
  })

  it('treats a missing oauth_providers field as empty (compat with older servers)', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.json({ allow_signup: true }))
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.allow_signup).toBe(true)
    })
    expect(result.current.oauth_providers).toEqual([])
  })

  it('treats a missing email_enabled field as false (compat with older servers)', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.json({ allow_signup: true }))
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.allow_signup).toBe(true)
    })
    // Fail-closed: an older server that doesn't return email_enabled is
    // assumed to lack working email — hides verification banners etc.
    expect(result.current.email_enabled).toBe(false)
  })

  it('reflects server-supplied email_enabled value', async () => {
    server.use(
      http.get('/api/v1/features', () => HttpResponse.json({
        allow_signup: true,
        oauth_providers: [],
        email_enabled: true,
      }))
    )
    const { result } = renderHook(() => useAppFeatures(), {
      wrapper: AppFeaturesProvider,
    })
    await waitFor(() => {
      expect(result.current.email_enabled).toBe(true)
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
    // OAuth fails closed: defaults already set oauth_providers to [] and a
    // failed fetch leaves them that way (no display names to render). Email
    // fails closed for the same reason — better silent than promise
    // behaviour we can't deliver.
    expect(result.current.oauth_providers).toEqual([])
    expect(result.current.email_enabled).toBe(false)
  })
})
