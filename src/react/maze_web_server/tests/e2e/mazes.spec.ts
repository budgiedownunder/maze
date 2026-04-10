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

  // Wait for loading to complete — either the list or the empty state must be visible
  const items = page.locator('.maze-list-item')
  const emptyState = page.getByText(/no mazes yet/i)
  await expect(items.first().or(emptyState)).toBeVisible()

  if (await items.count() > 0) {
    // Each item should have a subtitle matching "N rows × M columns"
    const subtitle = items.first().locator('.maze-item-subtitle')
    await expect(subtitle).toHaveText(/\d+ rows × \d+ columns/)
  } else {
    await expect(emptyState).toBeVisible()
  }
})

test('Duplicate → "Copy of X" pre-filled → confirm → new item in list', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()
  const initialCount = await items.count()

  await items.first().getByRole('button', { name: /duplicate/i }).click()
  const dialog = page.getByRole('dialog', { name: 'Duplicate Maze' })
  await expect(dialog).toBeVisible()
  await expect(dialog.getByRole('textbox')).toHaveValue(`Copy of ${firstName}`)

  await dialog.getByRole('button', { name: /^duplicate$/i }).click()

  await expect(dialog).not.toBeVisible()
  await expect(items).toHaveCount(initialCount + 1)
  await expect(page.locator('.maze-item-name', { hasText: `Copy of ${firstName}` })).toBeVisible()
})

test('Duplicate → cancel → list unchanged', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const initialCount = await items.count()

  await items.first().getByRole('button', { name: /duplicate/i }).click()
  await expect(page.getByRole('dialog', { name: 'Duplicate Maze' })).toBeVisible()

  await page.getByRole('button', { name: /cancel/i }).click()

  await expect(page.getByRole('dialog')).not.toBeVisible()
  await expect(items).toHaveCount(initialCount)
})

test('Rename → modal pre-filled → enter new name → list updates', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()

  await items.first().getByRole('button', { name: /rename/i }).click()
  const dialog = page.getByRole('dialog', { name: 'Rename Maze' })
  await expect(dialog).toBeVisible()
  await expect(dialog.getByRole('textbox')).toHaveValue(firstName!)

  await dialog.getByRole('textbox').fill('RenamedMaze')
  await dialog.getByRole('button', { name: /^rename$/i }).click()

  await expect(dialog).not.toBeVisible()
  await expect(page.locator('.maze-item-name', { hasText: 'RenamedMaze' })).toBeVisible()
  await expect(page.locator('.maze-item-name', { hasText: firstName! })).not.toBeVisible()
})

test('Rename → cancel → name unchanged', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()

  await items.first().getByRole('button', { name: /rename/i }).click()
  await expect(page.getByRole('dialog', { name: 'Rename Maze' })).toBeVisible()

  await page.getByRole('button', { name: /cancel/i }).click()

  await expect(page.getByRole('dialog')).not.toBeVisible()
  await expect(page.locator('.maze-item-name', { hasText: firstName! })).toBeVisible()
})

test('Delete → confirm → item removed from list', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()
  const initialCount = await items.count()

  await items.first().getByRole('button', { name: /delete/i }).click()
  await expect(page.getByRole('dialog', { name: 'Delete Maze' })).toBeVisible()
  await expect(page.getByRole('dialog')).toContainText(firstName!)

  await page.getByRole('dialog').getByRole('button', { name: /^delete$/i }).click()

  await expect(items).toHaveCount(initialCount - 1)
  await expect(page.locator('.maze-item-name', { hasText: firstName! })).not.toBeVisible()
})

test('Delete → cancel → item stays in list', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()
  const initialCount = await items.count()

  await items.first().getByRole('button', { name: /delete/i }).click()
  await expect(page.getByRole('dialog', { name: 'Delete Maze' })).toBeVisible()

  await page.getByRole('button', { name: /cancel/i }).click()

  await expect(page.getByRole('dialog')).not.toBeVisible()
  await expect(items).toHaveCount(initialCount)
  await expect(page.locator('.maze-item-name', { hasText: firstName! })).toBeVisible()
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
