import { test, expect } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  // Log in to reach the Home page (post-sign-in landing).
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
})

test('hamburger menu opens and closes on outside click', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await expect(page.getByRole('menu')).toBeVisible()
  await page.mouse.click(10, 10)
  await expect(page.getByRole('menu')).not.toBeVisible()
})

test('About modal opens and closes', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /about/i }).click()
  await expect(page.getByRole('dialog', { name: /about/i })).toBeVisible()
  await expect(page.getByRole('dialog')).toContainText('Maze')
  await expect(page.getByRole('dialog')).toContainText('© BudgieDownUnder, 2026')
  await page.getByRole('button', { name: /close/i }).click()
  await expect(page.getByRole('dialog')).not.toBeVisible()
})

test('Home menu item navigates back to home from /account', async ({ page }) => {
  // First go to /account so the back-to-Home navigation has somewhere to come from.
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)
  // Wait for the (lazily-loaded) Account page to finish rendering before
  // re-opening the menu, so the interaction doesn't race the Suspense fallback.
  await expect(page.getByRole('heading', { name: /^my account$/i })).toBeVisible()

  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^home$/i }).click()
  await expect(page).toHaveURL(/\/$/)
})

test('Mazes menu item navigates to /mazes', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^mazes$/i }).click()
  await expect(page).toHaveURL(/\/mazes$/)
})

test("hamburger menu Today's Challenge launches the seeded daily game", async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /today's challenge/i }).click()
  // Resolves the curated "Daily Challenges" collection (dev:mock seeds `def-daily`)
  // and launches its daily member via the host page.
  await expect(page).toHaveURL(/\/game\/\?def=def-daily/)
})

test('hamburger menu 3D Games item navigates to the Play-3D hub', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^3d games$/i }).click()
  await expect(page).toHaveURL(/\/play-3d$/)
  await expect(page.getByRole('heading', { name: /^featured$/i })).toBeVisible()
})

test('hamburger menu Featured sub-item navigates to the Featured page', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^featured$/i }).click()
  await expect(page).toHaveURL(/\/play-3d\/featured$/)
})

test('hamburger menu 3D Games sub-items navigate to each scope page', async ({ page }) => {
  for (const [name, url, heading] of [
    [/^my games$/i, /\/play-3d\/my-games$/, 'My Games'],
    [/^shared with me$/i, /\/play-3d\/shared$/, 'Shared with me'],
    [/^community$/i, /\/play-3d\/community$/, 'Community'],
  ] as const) {
    await page.getByRole('button', { name: /open menu/i }).click()
    await page.getByRole('menuitem', { name }).click()
    await expect(page).toHaveURL(url)
    // Wait for the lazily-loaded destination to render before the next iteration
    // re-opens the menu, so it doesn't race the Suspense fallback (which briefly
    // drops the header/menu).
    await expect(page.getByRole('banner').getByText(heading)).toBeVisible()
  }
})

test('My Account page opens and shows profile fields', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)
  await expect(page.getByRole('heading', { name: /my account/i })).toBeVisible()
  await expect(page.getByLabel('Username')).toHaveValue('testuser')
  await expect(page.getByText('test@example.com')).toBeVisible()
})

test('Change Password modal opens from Account page and Back returns', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)

  await page.getByRole('button', { name: /change password/i }).click()
  await expect(page.getByRole('dialog', { name: /change password/i })).toBeVisible()

  await page.getByRole('button', { name: /back/i }).click()
  await expect(page.getByRole('dialog', { name: /change password/i })).not.toBeVisible()
  await expect(page).toHaveURL(/\/account/)
})

test('Delete Account shows confirmation and Cancel returns to account page', async ({ page }) => {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)

  await page.getByRole('button', { name: /delete account/i }).click()
  await expect(page.getByText(/cannot be undone/i)).toBeVisible()

  await page.getByRole('button', { name: /cancel/i }).click()
  await expect(page.getByText(/cannot be undone/i)).not.toBeVisible()
  await expect(page).toHaveURL(/\/account/)
})
