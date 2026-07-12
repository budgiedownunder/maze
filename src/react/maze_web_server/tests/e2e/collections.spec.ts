import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
  await page.goto('/workshop/game-collections')
}

test('the collections page shows the empty state before any collection exists', async ({ page }) => {
  await login(page)
  await expect(page.getByRole('button', { name: '+ New collection' })).toBeVisible()
  await expect(page.getByText('No collections yet.')).toBeVisible()
})

test('New collection creates a collection that appears in the list', async ({ page }) => {
  await login(page)

  const name = `Campaign ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Collection' })
  await expect(dialog).toBeVisible()
  await dialog.getByLabel('Name').fill(name)
  await dialog.getByLabel('Description (optional)').fill('My best levels')
  await dialog.getByRole('button', { name: 'Create' }).click()

  await expect(dialog).toBeHidden()
  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row).toBeVisible()
  // A fresh collection is empty and private.
  await expect(row.getByText('0 games · Just me')).toBeVisible()

  // It survives a reload (the mock store is sessionStorage-backed).
  await page.reload()
  await expect(page.locator('.game-list-item', { hasText: name })).toBeVisible()
})
