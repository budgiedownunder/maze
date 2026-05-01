function validateNewPasswordComplexity(password: string): string | null {
  if (password.length < 8) {
    return 'Password must be at least 8 characters'
  }
  if (!/[A-Z]/.test(password)) return 'Password must contain an uppercase letter'
  if (!/[a-z]/.test(password)) return 'Password must contain a lowercase letter'
  if (!/[0-9]/.test(password)) return 'Password must contain a digit'
  if (!/[^A-Za-z0-9]/.test(password)) return 'Password must contain a special character'
  return null
}

export function validateChangePasswordForm(fields: {
  currentPassword: string
  newPassword: string
  confirmPassword: string
}): string | null {
  if (!fields.currentPassword || !fields.newPassword || !fields.confirmPassword) {
    return 'All fields are required'
  }
  if (fields.newPassword !== fields.confirmPassword) {
    return 'New passwords do not match'
  }
  return validateNewPasswordComplexity(fields.newPassword)
}

export function validateSetPasswordForm(fields: {
  newPassword: string
  confirmPassword: string
}): string | null {
  if (!fields.newPassword || !fields.confirmPassword) {
    return 'All fields are required'
  }
  if (fields.newPassword !== fields.confirmPassword) {
    return 'New passwords do not match'
  }
  return validateNewPasswordComplexity(fields.newPassword)
}
