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

  it('explains email_collision distinct from generic store errors', () => {
    const collision = getOAuthErrorMessage('email_collision')!
    expect(collision).toMatch(/account already exists/i)
    // The actionable cause: the email exists but is unverified. Both
    // password-sign-in and OAuth-auto-link gate on a verified email, so
    // the user must verify before either path will work.
    expect(collision).toMatch(/verif/i)
    // Must not blur into the catch-all "server error" message.
    expect(collision).not.toMatch(/server error/i)
    expect(getOAuthErrorMessage('store_error')).toMatch(/server error/i)
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
    expect(result).toEqual({ token: 'abc-123', expiresAt: '2026-04-26T12:00:00Z', newUser: false, firstSignIn: false })
  })

  it('accepts a hash without leading #', () => {
    const result = parseCallbackHash('token=abc-123&expires_at=2026-04-26T12:00:00Z')
    expect(result).toEqual({ token: 'abc-123', expiresAt: '2026-04-26T12:00:00Z', newUser: false, firstSignIn: false })
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

  it('flags newUser=true when the server emits new_user=true', () => {
    // Set by the Rust callback handler when account::resolve returned `Created`
    // (User row was just created during this OAuth flow). Distinct from
    // firstSignIn, which is the welcome-banner trigger.
    const result = parseCallbackHash('#token=abc&expires_at=2026-04-26T12:00:00Z&new_user=true')
    expect(result?.newUser).toBe(true)
  })

  it('returns newUser=false when the server omits new_user (returning user)', () => {
    const result = parseCallbackHash('#token=abc&expires_at=2026-04-26T12:00:00Z')
    expect(result?.newUser).toBe(false)
  })

  it('returns newUser=false for any value other than the literal string "true"', () => {
    expect(parseCallbackHash('#token=a&expires_at=z&new_user=1')?.newUser).toBe(false)
    expect(parseCallbackHash('#token=a&expires_at=z&new_user=yes')?.newUser).toBe(false)
    expect(parseCallbackHash('#token=a&expires_at=z&new_user=false')?.newUser).toBe(false)
  })

  it('flags firstSignIn=true when the server emits first_sign_in=true', () => {
    const result = parseCallbackHash('#token=abc&expires_at=2026-04-26T12:00:00Z&first_sign_in=true')
    expect(result?.firstSignIn).toBe(true)
  })

  it('returns firstSignIn=false when the server omits first_sign_in', () => {
    const result = parseCallbackHash('#token=abc&expires_at=2026-04-26T12:00:00Z')
    expect(result?.firstSignIn).toBe(false)
  })

  it('treats newUser and firstSignIn as independent flags', () => {
    const both = parseCallbackHash('#token=a&expires_at=z&new_user=true&first_sign_in=true')
    expect(both).toMatchObject({ newUser: true, firstSignIn: true })
    const onlyFirst = parseCallbackHash('#token=a&expires_at=z&first_sign_in=true')
    expect(onlyFirst).toMatchObject({ newUser: false, firstSignIn: true })
    const onlyNew = parseCallbackHash('#token=a&expires_at=z&new_user=true')
    expect(onlyNew).toMatchObject({ newUser: true, firstSignIn: false })
  })
})
