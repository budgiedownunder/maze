import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

async function openAccountPage(page: Page) {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)
  await expect(page.getByRole('heading', { name: /my account/i })).toBeVisible()
}

test('add then make primary then remove email round-trip through the account page', async ({ page }) => {
  await login(page)
  await openAccountPage(page)
  const main = page.locator('main')

  // Initial state: one row, the seeded primary email.
  const emailList = main.locator('.email-list')
  await expect(emailList.locator('li')).toHaveCount(1)
  const seededRow = emailList.locator('li').filter({ hasText: 'test@example.com' })
  await expect(seededRow.getByText('Primary')).toBeVisible()
  await expect(seededRow.getByText('Verified')).toBeVisible()

  // Add a new email.
  await main.getByPlaceholder(/add another email/i).fill('second@example.com')
  await main.getByRole('button', { name: /^Add$/ }).click()

  // List grows to two rows; the new row is verified but not primary.
  await expect(emailList.locator('li')).toHaveCount(2)
  const newRow = emailList.locator('li').filter({ hasText: 'second@example.com' })
  await expect(newRow.getByText('Verified')).toBeVisible()
  // `Make Primary` button also contains "Primary" — scope to the badge class.
  await expect(newRow.locator('.badge-primary')).toHaveCount(0)

  // Promote the new row to primary.
  await newRow.getByRole('button', { name: /Make Primary/ }).click()
  await expect(newRow.locator('.badge-primary')).toBeVisible()
  await expect(seededRow.locator('.badge-primary')).toHaveCount(0)

  // Now the previously primary row is removable. Remove it (via confirm modal).
  await seededRow.getByRole('button', { name: /^Remove$/ }).click()
  const confirmDialog = page.getByRole('dialog', { name: /remove email address/i })
  await expect(confirmDialog).toBeVisible()
  await confirmDialog.getByRole('button', { name: /^Remove$/ }).click()
  await expect(emailList.locator('li')).toHaveCount(1)
  await expect(emailList.locator('li').filter({ hasText: 'second@example.com' })).toBeVisible()
})
