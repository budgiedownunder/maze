import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Username').fill('testuser')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

test('clicking a maze from the list navigates to the editor and loads the grid', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()

  // Click the item row (not an action button)
  await items.first().locator('.maze-item-text').click()

  // Should navigate to /mazes/:id
  await expect(page).toHaveURL(/\/mazes\/[^/]+$/)

  // Maze name should appear in the header
  await expect(page.locator('.app-header-title')).toHaveText(firstName!)

  // Grid container should be visible
  await expect(page.locator('.maze-grid-container')).toBeVisible()
})

test('maze grid contains expected cell images', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  await items.first().locator('.maze-item-text').click()
  await expect(page).toHaveURL(/\/mazes\/[^/]+$/)

  // Wait for the grid to appear
  await expect(page.locator('.maze-grid-container')).toBeVisible()

  // At least a Start image should be present (all mock mazes have S)
  await expect(page.locator('img[alt="Start"]')).toBeVisible()
  // And a Finish image
  await expect(page.locator('img[alt="Finish"]')).toBeVisible()
})
