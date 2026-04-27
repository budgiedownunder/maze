import { describe, it, expect } from 'vitest'
import { parseCallbackHash, getOAuthErrorMessage } from '../utils/oauth'

describe('getOAuthErrorMessage', () => {
  it('returns null for null / empty input', () => {
    expect(getOAuthErrorMessage(null)).toBeNull()
    expect(getOAuthErrorMessage('')).toBeNull()
  })

  it('explains signup_disabled in user-friendly terms', () => {
    // The original bug this fixes: with allow_signup=false, the server
    // redirects to /login?error=signup_disabled but the user saw nothing.
    const msg = getOAuthErrorMessage('signup_disabled')
    expect(msg).toMatch(/sign-up is disabled/i)
    expect(msg).toMatch(/existing users/i)
  })

  it('explains email_not_verified', () => {
    expect(getOAuthErrorMessage('email_not_verified')).toMatch(/verified email/i)
  })

  it('coalesces all state-related codes into one message', () => {
    const message = getOAuthErrorMessage('invalid_state')
    expect(getOAuthErrorMessage('missing_state')).toBe(message)
    expect(getOAuthErrorMessage('state_mismatch')).toBe(message)
    expect(getOAuthErrorMessage('state_expired')).toBe(message)
    expect(getOAuthErrorMessage('provider_mismatch')).toBe(message)
    expect(message).toMatch(/expired|invalid/i)
  })

  it('handles provider_error:access_denied as a clean cancellation message', () => {
    expect(getOAuthErrorMessage('provider_error:access_denied'))
      .toMatch(/cancel/i)
  })

  it('echoes other provider_error codes back in a friendly wrapper', () => {
    const msg = getOAuthErrorMessage('provider_error:something_unexpected')!
    expect(msg).toMatch(/something_unexpected/)
  })

  it('falls back to a generic message for unknown codes', () => {
    const msg = getOAuthErrorMessage('completely_made_up_code')!
    // Must NOT echo the raw code back to the user.
    expect(msg).not.toMatch(/completely_made_up_code/)
    expect(msg).toMatch(/could not sign you in/i)
  })
})

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
