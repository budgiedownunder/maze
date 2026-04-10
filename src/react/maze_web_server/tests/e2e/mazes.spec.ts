import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Username').fill('testuser')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

test('maze list page loads after login', async ({ page }) => {
  await login(page)

  // Page should show either a list of mazes or the empty state — not the old placeholder
  await expect(page.getByRole('main')).toBeVisible()
  await expect(page.locator('text=coming soon')).not.toBeVisible()

  // Refresh button should be present in the header
  await expect(page.getByRole('button', { name: /refresh/i })).toBeVisible()
})

test('maze list shows dimensions subtitle in correct format', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  const count = await items.count()

  if (count > 0) {
    // Each item should have a subtitle matching "N rows × M columns"
    const subtitle = items.first().locator('.maze-item-subtitle')
    await expect(subtitle).toHaveText(/\d+ rows × \d+ columns/)
  } else {
    await expect(page.getByText(/no mazes yet/i)).toBeVisible()
  }
})

test('refresh button reloads the maze list', async ({ page }) => {
  await login(page)

  // Wait for initial load to complete (list or empty state visible)
  await expect(page.locator('main')).toBeVisible()
  const hasItems = await page.locator('.maze-list-item').count() > 0
  if (hasItems) {
    await expect(page.locator('.maze-list')).toBeVisible()
  } else {
    await expect(page.getByText(/no mazes yet/i)).toBeVisible()
  }

  // Click refresh — page should still show the list or empty state afterwards
  await page.getByRole('button', { name: /refresh/i }).click()

  if (hasItems) {
    await expect(page.locator('.maze-list')).toBeVisible()
  } else {
    await expect(page.getByText(/no mazes yet/i)).toBeVisible()
  }
})
