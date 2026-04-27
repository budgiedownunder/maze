import { describe, it, expect } from 'vitest'
import { validateSignupForm } from '../../src/utils/validation'
import { validateChangePasswordForm } from '../../src/components/ChangePasswordModal'

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
