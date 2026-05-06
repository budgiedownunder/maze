import { describe, it, expect, beforeEach } from 'vitest'
import {
  addMyEmail,
  confirmEmailVerification,
  confirmPasswordReset,
  getMyEmails,
  removeMyEmail,
  requestEmailVerification,
  requestPasswordReset,
  setPrimaryEmail,
} from '../../src/api/client'
import { mockResetTokens, mockVerificationTokens, resetMockEmails, resetMockTokens } from '../../src/mocks/handlers'

const TOKEN = 'test-token'

beforeEach(() => {
  resetMockEmails()
  resetMockTokens()
})

describe('getMyEmails', () => {
  it('returns the seeded primary email', async () => {
    const result = await getMyEmails(TOKEN)

    expect(result.emails).toHaveLength(1)
    expect(result.emails[0]).toMatchObject({
      email: 'test@example.com',
      is_primary: true,
      verified: true,
    })
  })
})

describe('addMyEmail', () => {
  it('adds a new non-primary verified email', async () => {
    const result = await addMyEmail(TOKEN, 'second@example.com')

    expect(result.emails).toHaveLength(2)
    const added = result.emails.find(e => e.email === 'second@example.com')
    expect(added?.is_primary).toBe(false)
    expect(added?.verified).toBe(true)
  })

  it('throws 409 when adding a duplicate', async () => {
    await expect(addMyEmail(TOKEN, 'test@example.com')).rejects.toMatchObject({ status: 409 })
  })
})

describe('removeMyEmail', () => {
  it('removes a non-primary email', async () => {
    await addMyEmail(TOKEN, 'second@example.com')

    const result = await removeMyEmail(TOKEN, 'second@example.com')

    expect(result.emails).toHaveLength(1)
    expect(result.emails[0].email).toBe('test@example.com')
  })

  it('throws 409 when removing the only email (which is also primary)', async () => {
    await expect(removeMyEmail(TOKEN, 'test@example.com')).rejects.toMatchObject({ status: 409 })
  })

  it('throws 409 when removing the primary while another email exists', async () => {
    await addMyEmail(TOKEN, 'second@example.com')

    await expect(removeMyEmail(TOKEN, 'test@example.com')).rejects.toMatchObject({ status: 409 })
  })

  it('throws 404 for an unknown email', async () => {
    await expect(removeMyEmail(TOKEN, 'unknown@example.com')).rejects.toMatchObject({ status: 404 })
  })

  it('URL-encodes the email path segment', async () => {
    // Plus signs in addresses (`user+tag@example.com`) must not be misread as
    // spaces in the path; `encodeURIComponent` is the contract verified here.
    await addMyEmail(TOKEN, 'user+tag@example.com')

    const result = await removeMyEmail(TOKEN, 'user+tag@example.com')

    expect(result.emails.some(e => e.email === 'user+tag@example.com')).toBe(false)
  })
})

describe('setPrimaryEmail', () => {
  it('promotes a verified email to primary and clears the previous primary', async () => {
    await addMyEmail(TOKEN, 'second@example.com')

    const result = await setPrimaryEmail(TOKEN, 'second@example.com')

    expect(result.emails.find(e => e.email === 'second@example.com')?.is_primary).toBe(true)
    expect(result.emails.find(e => e.email === 'test@example.com')?.is_primary).toBe(false)
  })

  it('throws 404 for an unknown email', async () => {
    await expect(setPrimaryEmail(TOKEN, 'unknown@example.com')).rejects.toMatchObject({ status: 404 })
  })
})

describe('requestPasswordReset', () => {
  it('always returns 200 (anti-enumeration) and mints a token for known emails', async () => {
    await requestPasswordReset('test@example.com')

    expect([...mockResetTokens.values()]).toContain('test@example.com')
  })

  it('returns 200 even for unknown emails (no enumeration)', async () => {
    await expect(requestPasswordReset('unknown@example.com')).resolves.toBeUndefined()
    expect([...mockResetTokens.values()]).not.toContain('unknown@example.com')
  })
})

describe('confirmPasswordReset', () => {
  it('consumes the minted token on success', async () => {
    await requestPasswordReset('test@example.com')
    const token = [...mockResetTokens.keys()][0]

    await confirmPasswordReset(token, 'NewPassword1!')

    expect(mockResetTokens.has(token)).toBe(false)
  })

  it('throws 400 for an invalid or expired token', async () => {
    await expect(confirmPasswordReset('not-a-real-token', 'NewPassword1!'))
      .rejects.toMatchObject({ status: 400 })
  })
})

describe('requestEmailVerification', () => {
  it('mints a token for an email on the current user', async () => {
    await requestEmailVerification(TOKEN, 'test@example.com')

    expect([...mockVerificationTokens.values()]).toContain('test@example.com')
  })

  it('returns 200 even for an email not on the user (anti-enumeration)', async () => {
    await expect(requestEmailVerification(TOKEN, 'stranger@example.com')).resolves.toBeUndefined()
    expect([...mockVerificationTokens.values()]).not.toContain('stranger@example.com')
  })
})

describe('confirmEmailVerification', () => {
  it('consumes the minted verification token on success', async () => {
    await requestEmailVerification(TOKEN, 'test@example.com')
    const verificationToken = [...mockVerificationTokens.keys()][0]

    await confirmEmailVerification(verificationToken)

    expect(mockVerificationTokens.has(verificationToken)).toBe(false)
  })

  it('throws 400 for an invalid or already-used token', async () => {
    await expect(confirmEmailVerification('not-a-real-token'))
      .rejects.toMatchObject({ status: 400 })
  })
})
