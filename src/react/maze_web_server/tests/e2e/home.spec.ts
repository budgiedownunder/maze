import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('successful sign-in lands on the Home page with the tiles visible', async ({ page }) => {
  await login(page)
  await expect(page.getByRole('heading', { name: /^3d games$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^mazes$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^leaderboards$/i })).toBeVisible()
})

test('clicking the 3D Games tile navigates to /play-3d', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /browse and play 3d games/i }).click()
  await expect(page).toHaveURL(/\/play-3d$/)
})

test('clicking Mazes tile navigates to /mazes', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /mazes/i }).click()
  await expect(page).toHaveURL(/\/mazes$/)
})

test('clicking Leaderboards tile navigates to /leaderboards', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /your times and how you rank/i }).click()
  await expect(page).toHaveURL(/\/leaderboards$/)
})
