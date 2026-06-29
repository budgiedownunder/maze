import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('Leaderboards opens from the menu and shows the default board', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^leaderboards$/i }).click()
  await expect(page).toHaveURL(/\/leaderboards$/)
  // Default subject is the most-recently played maze (Alpha) → its time shows.
  await expect(page.getByText('0:42.137')).toBeVisible()
  // The Play button (launches the selected subject in 3D) is always present —
  // labelled "Play Again" here since the default subject is a played maze.
  await expect(page.getByRole('button', { name: /play( again)?/i })).toBeVisible()
})

test('switching to Play 3D shows a global board with usernames', async ({ page }) => {
  await login(page)
  await page.goto('/leaderboards')
  await expect(page.getByLabel('Game Type')).toBeVisible()
  await page.getByLabel('Game Type').selectOption('play3d')
  // The curated board resolves its seed + lists every player by username,
  // including the signed-in user (testuser). Scope to board cells — the
  // signed-in username also appears in the page header (the account link).
  await expect(page.getByRole('cell', { name: 'alice' })).toBeVisible()
  await expect(page.getByRole('cell', { name: 'testuser' })).toBeVisible()
})
