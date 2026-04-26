import { describe, it, expect } from 'vitest'
import { parseCallbackHash } from '../utils/oauth'

describe('parseCallbackHash', () => {
  it('extracts token and expires_at from a hash with leading #', () => {
    const result = parseCallbackHash('#token=abc-123&expires_at=2026-04-26T12:00:00Z')
    expect(result).toEqual({ token: 'abc-123', expiresAt: '2026-04-26T12:00:00Z' })
  })

  it('accepts a hash without leading #', () => {
    const result = parseCallbackHash('token=abc-123&expires_at=2026-04-26T12:00:00Z')
    expect(result).toEqual({ token: 'abc-123', expiresAt: '2026-04-26T12:00:00Z' })
  })

  it('decodes percent-encoded expires_at', () => {
    // The server URL-encodes expires_at; URLSearchParams handles the decode.
    const result = parseCallbackHash('#token=abc&expires_at=2026-04-26T12%3A00%3A00Z')
    expect(result?.expiresAt).toBe('2026-04-26T12:00:00Z')
  })

  it('returns null when token is missing', () => {
    expect(parseCallbackHash('#expires_at=2026-04-26T12:00:00Z')).toBeNull()
  })

  it('returns null when expires_at is missing', () => {
    expect(parseCallbackHash('#token=abc')).toBeNull()
  })

  it('returns null for an empty hash', () => {
    expect(parseCallbackHash('')).toBeNull()
  })
})
