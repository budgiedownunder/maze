import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

async function openAccountModal(page: Page) {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page.getByRole('dialog', { name: /my account/i })).toBeVisible()
}

test('add then make primary then remove email round-trip through the account modal', async ({ page }) => {
  await login(page)
  await openAccountModal(page)
  const dialog = page.getByRole('dialog', { name: /my account/i })

  // Initial state: one row, the seeded primary email.
  const emailList = dialog.locator('.email-list')
  await expect(emailList.locator('li')).toHaveCount(1)
  const seededRow = emailList.locator('li').filter({ hasText: 'test@example.com' })
  await expect(seededRow.getByText('Primary')).toBeVisible()
  await expect(seededRow.getByText('Verified')).toBeVisible()

  // Add a new email.
  await dialog.getByPlaceholder(/add another email/i).fill('second@example.com')
  await dialog.getByRole('button', { name: /^Add Email$/ }).click()

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

  // Now the previously primary row is removable. Remove it.
  await seededRow.getByRole('button', { name: /^Remove$/ }).click()
  await expect(emailList.locator('li')).toHaveCount(1)
  await expect(emailList.locator('li').filter({ hasText: 'second@example.com' })).toBeVisible()
})
