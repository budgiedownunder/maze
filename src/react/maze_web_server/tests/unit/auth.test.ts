import { describe, it, expect } from 'vitest'
import { shouldRenewToken, RENEWAL_FRACTION } from '../../src/context/AuthContext'

function makeTimestamps(lifetimeHours: number, elapsedFraction: number): { issuedAt: string; expiry: string; now: Date } {
  const lifetimeMs = lifetimeHours * 60 * 60 * 1000
  const issuedAt = new Date(0)
  const expiry = new Date(lifetimeMs)
  const now = new Date(elapsedFraction * lifetimeMs)
  return { issuedAt: issuedAt.toISOString(), expiry: expiry.toISOString(), now }
}

describe('shouldRenewToken', () => {
  it('returns false when well within the renewal window (24h token, 10% elapsed)', () => {
    const { issuedAt, expiry, now } = makeTimestamps(24, 0.1)
    expect(shouldRenewToken(issuedAt, expiry, now)).toBe(false)
  })

  it('returns false when exactly at the renewal threshold boundary', () => {
    // At exactly (1 - RENEWAL_FRACTION) elapsed, remaining/lifetime === RENEWAL_FRACTION — not less than
    const { issuedAt, expiry, now } = makeTimestamps(24, 1 - RENEWAL_FRACTION)
    expect(shouldRenewToken(issuedAt, expiry, now)).toBe(false)
  })

  it('returns true when just past the renewal threshold (24h token)', () => {
    const { issuedAt, expiry, now } = makeTimestamps(24, 1 - RENEWAL_FRACTION + 0.001)
    expect(shouldRenewToken(issuedAt, expiry, now)).toBe(true)
  })

  it('returns true when near expiry (24h token, 99% elapsed)', () => {
    const { issuedAt, expiry, now } = makeTimestamps(24, 0.99)
    expect(shouldRenewToken(issuedAt, expiry, now)).toBe(true)
  })

  it('returns false when already expired', () => {
    const { issuedAt, expiry, now } = makeTimestamps(24, 1.01)
    expect(shouldRenewToken(issuedAt, expiry, now)).toBe(false)
  })

  it('scales correctly for a short-lived 4h token', () => {
    const withinWindow = makeTimestamps(4, 0.5)
    expect(shouldRenewToken(withinWindow.issuedAt, withinWindow.expiry, withinWindow.now)).toBe(false)

    const pastThreshold = makeTimestamps(4, 1 - RENEWAL_FRACTION + 0.001)
    expect(shouldRenewToken(pastThreshold.issuedAt, pastThreshold.expiry, pastThreshold.now)).toBe(true)
  })

  it('scales correctly for a long-lived 48h token', () => {
    const withinWindow = makeTimestamps(48, 0.5)
    expect(shouldRenewToken(withinWindow.issuedAt, withinWindow.expiry, withinWindow.now)).toBe(false)

    const pastThreshold = makeTimestamps(48, 1 - RENEWAL_FRACTION + 0.001)
    expect(shouldRenewToken(pastThreshold.issuedAt, pastThreshold.expiry, pastThreshold.now)).toBe(true)
  })

  it('uses current time when now is not provided', () => {
    // Token expired in the past — should return false
    const issuedAt = new Date(Date.now() - 25 * 60 * 60 * 1000).toISOString()
    const expiry = new Date(Date.now() - 60 * 1000).toISOString()
    expect(shouldRenewToken(issuedAt, expiry)).toBe(false)
  })
})
