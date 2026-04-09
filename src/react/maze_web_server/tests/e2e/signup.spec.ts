import { test, expect } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.goto('/signup')
})

test('mismatched passwords shows error and does not navigate', async ({ page }) => {
  await page.getByLabel('Username').fill('newuser')
  await page.getByLabel('Full Name').fill('New User')
  await page.getByLabel('Email').fill('new@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByLabel('Confirm Password', { exact: true }).fill('Different1!')
  await page.getByRole('button', { name: /sign up/i }).click()
  await expect(page.getByRole('alert')).toContainText(/match/)
  await expect(page).toHaveURL(/\/signup/)
})

test('weak password shows validation error', async ({ page }) => {
  await page.getByLabel('Username').fill('newuser')
  await page.getByLabel('Full Name').fill('New User')
  await page.getByLabel('Email').fill('new@example.com')
  await page.getByLabel('Password', { exact: true }).fill('weak')
  await page.getByLabel('Confirm Password', { exact: true }).fill('weak')
  await page.getByRole('button', { name: /sign up/i }).click()
  await expect(page.getByRole('alert')).toBeVisible()
  await expect(page).toHaveURL(/\/signup/)
})

test('valid signup navigates to /login', async ({ page }) => {
  await page.getByLabel('Username').fill('newuser')
  await page.getByLabel('Full Name').fill('New User')
  await page.getByLabel('Email').fill('new@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByLabel('Confirm Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign up/i }).click()
  await expect(page).toHaveURL(/\/login/)
})

test('Back button returns to /login', async ({ page }) => {
  await page.getByRole('button', { name: /back/i }).click()
  await expect(page).toHaveURL(/\/login/)
})
