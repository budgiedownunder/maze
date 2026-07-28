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

test('switching to 3D Games prompts for a game until one is picked', async ({ page }) => {
  await login(page)
  await page.goto('/leaderboards')
  await expect(page.getByLabel('Game Type')).toBeVisible()
  await page.getByLabel('Game Type').selectOption('play3d')
  await expect(page.getByText('Choose a game to see its leaderboard.')).toBeVisible()
  await expect(page.getByRole('button', { name: /^▶ Play$/ })).toBeDisabled()
})

test('picking a game in the picker shows its board with usernames', async ({ page }) => {
  await login(page)
  await page.goto('/leaderboards')
  await page.getByLabel('Game Type').selectOption('play3d')

  // Browse to a game another user published (the dev:mock Community fixture) and
  // pick it; the picker collapses to the selection.
  await page.getByRole('button', { name: 'Choose a game' }).click()
  await page.getByRole('tab', { name: 'Community' }).click()
  await page.getByRole('button', { name: 'Show leaderboard for Community Classic' }).click()
  await expect(page.getByRole('button', { name: 'Change' })).toBeVisible()

  // The game's board lists every player by username. Scope to board cells — the
  // signed-in username also appears in the page header (the account link).
  await expect(page.getByRole('cell', { name: 'alice' })).toBeVisible()
  await expect(page.getByRole('cell', { name: 'testuser' })).toBeVisible()
})

test('a daily game shows a date picker and browses past days', async ({ page }) => {
  await login(page)
  await page.goto('/leaderboards')
  await page.getByLabel('Game Type').selectOption('play3d')

  // The seeded daily game is curated → it shows on the picker's Featured tab.
  await page.getByRole('button', { name: 'Choose a game' }).click()
  await page.getByRole('button', { name: 'Show leaderboard for Daily Maze' }).click()

  // The date dropdown appears, defaulting to the most-recent day with runs
  // (2026-07-10) rather than today, which has none.
  const daySelect = page.getByRole('combobox', { name: 'Day' })
  await expect(daySelect).toBeVisible()
  await expect(daySelect).toHaveValue('2026-07-10')

  // Picking an earlier day with runs re-keys the board.
  await daySelect.selectOption('2026-07-05')
  await expect(daySelect).toHaveValue('2026-07-05')
  await expect(page.getByRole('table')).toBeVisible()
})
