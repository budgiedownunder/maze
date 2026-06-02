import { describe, it, expect } from 'vitest'
import {
  countKeysAndDoors,
  exceedsGenerateFeatureCap,
  exceedsKeyDoorCap,
  MAX_TOTAL_FEATURES,
  validateSignupForm,
} from '../../src/utils/validation'
import {
  validateChangePasswordForm,
  validateSetPasswordForm,
} from '../../src/utils/passwordValidation'

describe('validateSignupForm', () => {
  const valid = {
    email: 'test@example.com',
    password: 'Password1!',
    confirmPassword: 'Password1!',
  }

  it('returns null for a valid form', () => {
    expect(validateSignupForm(valid)).toBeNull()
  })

  it('requires all fields', () => {
    expect(validateSignupForm({ ...valid, email: '' })).not.toBeNull()
    expect(validateSignupForm({ ...valid, password: '' })).not.toBeNull()
    expect(validateSignupForm({ ...valid, confirmPassword: '' })).not.toBeNull()
  })

  it('requires a valid email address', () => {
    expect(validateSignupForm({ ...valid, email: 'mytest@x' })).toMatch(/valid email/i)
  })

  it('requires passwords to match', () => {
    expect(validateSignupForm({ ...valid, confirmPassword: 'Different1!' })).toMatch(/match/)
  })

  it('requires password of at least 8 characters', () => {
    expect(validateSignupForm({ ...valid, password: 'P1!aaaa', confirmPassword: 'P1!aaaa' })).toMatch(/8 characters/)
  })

  it('requires an uppercase letter', () => {
    expect(validateSignupForm({ ...valid, password: 'password1!', confirmPassword: 'password1!' })).toMatch(/uppercase/)
  })

  it('requires a lowercase letter', () => {
    expect(validateSignupForm({ ...valid, password: 'PASSWORD1!', confirmPassword: 'PASSWORD1!' })).toMatch(/lowercase/)
  })

  it('requires a digit', () => {
    expect(validateSignupForm({ ...valid, password: 'Password!!', confirmPassword: 'Password!!' })).toMatch(/digit/)
  })

  it('requires a special character', () => {
    expect(validateSignupForm({ ...valid, password: 'Password1a', confirmPassword: 'Password1a' })).toMatch(/special/)
  })
})

describe('validateChangePasswordForm', () => {
  const valid = {
    currentPassword: 'OldPass1!',
    newPassword: 'NewPass1!',
    confirmPassword: 'NewPass1!',
  }

  it('returns null for a valid form', () => {
    expect(validateChangePasswordForm(valid)).toBeNull()
  })

  it('requires all fields', () => {
    expect(validateChangePasswordForm({ ...valid, currentPassword: '' })).not.toBeNull()
    expect(validateChangePasswordForm({ ...valid, newPassword: '' })).not.toBeNull()
    expect(validateChangePasswordForm({ ...valid, confirmPassword: '' })).not.toBeNull()
  })

  it('requires new passwords to match', () => {
    expect(validateChangePasswordForm({ ...valid, confirmPassword: 'Different1!' })).toMatch(/match/)
  })

  it('requires new password of at least 8 characters', () => {
    expect(validateChangePasswordForm({ ...valid, newPassword: 'P1!aaaa', confirmPassword: 'P1!aaaa' })).toMatch(/8 characters/)
  })

  it('requires an uppercase letter', () => {
    expect(validateChangePasswordForm({ ...valid, newPassword: 'password1!', confirmPassword: 'password1!' })).toMatch(/uppercase/)
  })

  it('requires a lowercase letter', () => {
    expect(validateChangePasswordForm({ ...valid, newPassword: 'PASSWORD1!', confirmPassword: 'PASSWORD1!' })).toMatch(/lowercase/)
  })

  it('requires a digit', () => {
    expect(validateChangePasswordForm({ ...valid, newPassword: 'Password!!', confirmPassword: 'Password!!' })).toMatch(/digit/)
  })

  it('requires a special character', () => {
    expect(validateChangePasswordForm({ ...valid, newPassword: 'Password1a', confirmPassword: 'Password1a' })).toMatch(/special/)
  })
})

describe('validateSetPasswordForm', () => {
  const valid = {
    newPassword: 'NewPass1!',
    confirmPassword: 'NewPass1!',
  }

  it('returns null for a valid form', () => {
    expect(validateSetPasswordForm(valid)).toBeNull()
  })

  it('requires both fields', () => {
    expect(validateSetPasswordForm({ ...valid, newPassword: '' })).not.toBeNull()
    expect(validateSetPasswordForm({ ...valid, confirmPassword: '' })).not.toBeNull()
  })

  it('requires new passwords to match', () => {
    expect(validateSetPasswordForm({ ...valid, confirmPassword: 'Different1!' })).toMatch(/match/)
  })

  it('requires new password of at least 8 characters', () => {
    expect(validateSetPasswordForm({ newPassword: 'P1!aaaa', confirmPassword: 'P1!aaaa' })).toMatch(/8 characters/)
  })

  it('requires an uppercase letter', () => {
    expect(validateSetPasswordForm({ newPassword: 'password1!', confirmPassword: 'password1!' })).toMatch(/uppercase/)
  })

  it('requires a lowercase letter', () => {
    expect(validateSetPasswordForm({ newPassword: 'PASSWORD1!', confirmPassword: 'PASSWORD1!' })).toMatch(/lowercase/)
  })

  it('requires a digit', () => {
    expect(validateSetPasswordForm({ newPassword: 'Password!!', confirmPassword: 'Password!!' })).toMatch(/digit/)
  })

  it('requires a special character', () => {
    expect(validateSetPasswordForm({ newPassword: 'Password1a', confirmPassword: 'Password1a' })).toMatch(/special/)
  })
})

describe('countKeysAndDoors', () => {
  it('counts K and D cells, ignoring everything else', () => {
    const grid = [
      ['S', 'K', 'D', 'F'],
      ['W', 'K', ' ', 'D'],
    ]
    expect(countKeysAndDoors(grid)).toEqual({ keys: 2, doors: 2 })
  })

  it('returns zero on an empty grid', () => {
    expect(countKeysAndDoors([])).toEqual({ keys: 0, doors: 0 })
  })
})

describe('exceedsKeyDoorCap', () => {
  it('returns false when K + D is at the cap', () => {
    const grid: string[][] = [[]]
    for (let i = 0; i < 8; i++) grid[0].push('K')
    for (let i = 0; i < 8; i++) grid[0].push('D')
    expect(exceedsKeyDoorCap(grid)).toBe(false)
  })

  it('returns true when K + D is over the cap', () => {
    const grid: string[][] = [[]]
    for (let i = 0; i < 9; i++) grid[0].push('K')
    for (let i = 0; i < 8; i++) grid[0].push('D')
    expect(exceedsKeyDoorCap(grid)).toBe(true)
  })
})

describe('exceedsGenerateFeatureCap', () => {
  it('counts each real door twice (key + door pair)', () => {
    expect(exceedsGenerateFeatureCap(8, 0, 0)).toBe(false) // 2*8 = 16 at cap
    expect(exceedsGenerateFeatureCap(8, 1, 0)).toBe(true)  // 17 > 16
    expect(exceedsGenerateFeatureCap(4, 4, 4)).toBe(false) // 8 + 4 + 4 = 16
    expect(exceedsGenerateFeatureCap(4, 4, 5)).toBe(true)  // 17 > 16
  })

  it('pins MAX_TOTAL_FEATURES to 16 so the Rust side stays in sync', () => {
    expect(MAX_TOTAL_FEATURES).toBe(16)
  })
})
