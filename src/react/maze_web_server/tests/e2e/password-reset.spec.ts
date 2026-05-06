import { test, expect } from '@playwright/test'

test('navigating from sign-in to Forgot Password and submitting shows the anti-enumeration success copy', async ({ page }) => {
  await page.goto('/login')

  await page.getByRole('button', { name: /forgot password/i }).click()
  await expect(page).toHaveURL(/\/forgot-password/)

  await page.getByLabel('Email').fill('test@example.com')
  await page.getByRole('button', { name: /send reset link/i }).click()

  await expect(page.getByRole('status')).toContainText(/if that email is registered/i)
})

test('Forgot Password shows the same success copy for an unknown email', async ({ page }) => {
  await page.goto('/forgot-password')

  await page.getByLabel('Email').fill('stranger@example.com')
  await page.getByRole('button', { name: /send reset link/i }).click()

  await expect(page.getByRole('status')).toContainText(/if that email is registered/i)
})

test('Forgot Password Back to sign in returns to /login', async ({ page }) => {
  await page.goto('/forgot-password')
  await page.getByRole('button', { name: /back to sign in/i }).click()
  await expect(page).toHaveURL(/\/login/)
})

test('Reset Password without a token shows the invalid-link state', async ({ page }) => {
  await page.goto('/reset-password')
  await expect(page.getByRole('alert')).toContainText(/invalid/i)
})

test('Reset Password with an unknown token surfaces an inline error', async ({ page }) => {
  await page.goto('/reset-password?token=not-a-real-token')
  await page.getByLabel('New password', { exact: true }).fill('Password1!')
  await page.getByLabel('Confirm new password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /set new password/i }).click()
  await expect(page.getByRole('alert')).toContainText(/invalid or has expired/i)
})

test('Reset Password mismatched confirmation blocks the submit before any network call', async ({ page }) => {
  await page.goto('/reset-password?token=irrelevant')
  await page.getByLabel('New password', { exact: true }).fill('Password1!')
  await page.getByLabel('Confirm new password', { exact: true }).fill('Different1!')
  await page.getByRole('button', { name: /set new password/i }).click()
  await expect(page.getByRole('alert')).toContainText(/match/i)
})
