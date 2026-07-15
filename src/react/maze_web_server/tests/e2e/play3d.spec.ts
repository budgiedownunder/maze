import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

// The dev:mock backend grants admin to this email, so a game can be featured.
async function loginAsAdmin(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('admin@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('the 3D Games hub shows the four browse tiles and Featured opens empty', async ({ page }) => {
  await login(page)
  await page.goto('/play-3d')
  await expect(page.getByRole('heading', { name: /^featured$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^my games$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^shared with me$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^community$/i })).toBeVisible()

  await page.getByRole('button', { name: /featured/i }).click()
  await expect(page).toHaveURL(/\/play-3d\/featured$/)
  await expect(page.getByText(/no featured games or collections yet/i)).toBeVisible()
})

test('the not-yet-built scopes show a coming-soon placeholder', async ({ page }) => {
  await login(page)
  await page.goto('/play-3d/my-games')
  await expect(page.getByRole('banner').getByText('My Games')).toBeVisible()
  await expect(page.getByText('Coming soon.')).toBeVisible()

  await page.goto('/play-3d/community')
  await expect(page.getByText('Coming soon.')).toBeVisible()
})

test('a featured game appears as a card on Featured with Play and a Leaderboard modal', async ({ page }) => {
  await loginAsAdmin(page)

  // Create a game and feature it via the Access modal (admin-only Featured tier).
  await page.goto('/workshop/games')
  await page.getByRole('button', { name: 'New Game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  const name = `Feat ${Date.now()}`
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeHidden()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const access = page.getByRole('dialog', { name: /^Access:/ })
  await access.getByRole('radio', { name: /Featured/ }).click()
  await access.getByRole('button', { name: 'Save' }).click()
  await expect(access).toBeHidden()

  // It now shows as a gallery card on the Featured browse page.
  await page.goto('/play-3d/featured')
  const card = page.locator('.play3d-card', { hasText: name })
  await expect(card).toBeVisible()
  await expect(card.getByRole('button', { name: `Play ${name}` })).toBeVisible()

  // Leaderboard opens the board modal for that game.
  await card.getByRole('button', { name: `Leaderboard for ${name}` }).click()
  await expect(page.getByRole('dialog', { name: `Leaderboard: ${name}` })).toBeVisible()
})
