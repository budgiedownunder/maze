import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

// The dev:mock backend grants admin to this email (see src/mocks/handlers.ts), so
// the admin-only Manage Features surface is reachable in e2e.
async function loginAsAdmin(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('admin@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('Home 3D Game Workshop tile opens the hub', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /create, publish and share your own 3d games/i }).click()
  await expect(page).toHaveURL(/\/workshop$/)
  await expect(page.getByRole('heading', { name: /^manage games$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^manage game collections$/i })).toBeVisible()
})

test('the Features tile is hidden from a non-admin', async ({ page }) => {
  await login(page)
  await page.goto('/workshop')
  await expect(page.getByRole('heading', { name: /^manage game collections$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /features/i })).toBeHidden()
})

test('a non-admin visiting /workshop/features is bounced back to the hub', async ({ page }) => {
  await login(page)
  await page.goto('/workshop/features')
  await expect(page).toHaveURL(/\/workshop$/)
})

test('the hub Games tile opens the games surface', async ({ page }) => {
  await login(page)
  await page.goto('/workshop')
  await page.getByRole('button', { name: /create, edit, publish and share your 3d games/i }).click()
  await expect(page).toHaveURL(/\/workshop\/games$/)
  await expect(page.getByRole('button', { name: 'New game' })).toBeVisible()
})

test('the hamburger 3D Game Workshop item opens the hub', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /3d game workshop/i }).click()
  await expect(page).toHaveURL(/\/workshop$/)
})

test('the retired /games route redirects to the hub', async ({ page }) => {
  await login(page)
  await page.goto('/games')
  await expect(page).toHaveURL(/\/workshop$/)
})

test('an admin can open Manage Features and sees the empty state', async ({ page }) => {
  await loginAsAdmin(page)
  await page.goto('/workshop/features')
  await expect(page.getByRole('banner').getByText('Manage Features')).toBeVisible()
  await expect(page.getByText(/no featured items yet/i)).toBeVisible()
})

test('featuring a game via Access surfaces it on Manage Features', async ({ page }) => {
  await loginAsAdmin(page)

  // Create a game.
  await page.goto('/workshop/games')
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  const name = `Feat ${Date.now()}`
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeHidden()

  // Feature it via the Access modal (admin-only Featured tier).
  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const access = page.getByRole('dialog', { name: /^Access:/ })
  await access.getByRole('radio', { name: /Featured/ }).click()
  await access.getByRole('button', { name: 'Save' }).click()
  await expect(access).toBeHidden()

  // It now appears on Manage Features with its game actions + reorder controls,
  // and the summary names the owner.
  await page.goto('/workshop/features')
  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row).toBeVisible()
  await expect(row.getByText(/· by /)).toBeVisible()
  await expect(row.getByRole('button', { name: `Play ${name}` })).toBeVisible()
  await expect(row.getByRole('button', { name: `Leaderboard for ${name}` })).toBeVisible()
  await expect(row.getByRole('button', { name: `Unfeature ${name}` })).toBeVisible()
  // As the only featured row it is both first and last, so both arrows disable.
  await expect(row.getByRole('button', { name: `Move ${name} up` })).toBeDisabled()
  await expect(row.getByRole('button', { name: `Move ${name} down` })).toBeDisabled()

  // Unfeature it (the admin owns it → "Just me") and it drops off the list.
  await row.getByRole('button', { name: `Unfeature ${name}` }).click()
  const confirm = page.getByRole('dialog', { name: 'Unfeature' })
  await expect(confirm.getByText(/resets its access to Just me/)).toBeVisible()
  await confirm.getByRole('button', { name: 'Unfeature' }).click()
  await expect(page.locator('.game-list-item', { hasText: name })).toHaveCount(0)
})
