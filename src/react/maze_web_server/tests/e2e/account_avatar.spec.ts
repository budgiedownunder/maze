import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

async function openAccountPage(page: Page) {
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)
  await expect(page.getByRole('heading', { name: /my account/i })).toBeVisible()
}

// Upload a real PNG from the app's own assets (valid image/png, well under the
// size cap). Its bytes don't matter to the mock — which serves its own image
// and just stamps the marker — only that the client sees a PNG it accepts.
// Path is relative to the project root (the cwd Playwright runs from).
const AVATAR_FIXTURE = 'public/images/avatar-placeholder.png'

test('upload then remove an avatar updates the account avatar', async ({ page }) => {
  await login(page)
  await openAccountPage(page)

  const section = page.locator('.account-avatar')
  const avatarImg = section.locator('img')
  const removeButton = section.getByRole('button', { name: /^Remove$/ })

  // Initial state: placeholder, no Remove.
  await expect(avatarImg).toHaveAttribute('src', /avatar-placeholder\.png$/)
  await expect(removeButton).toHaveCount(0)

  // Upload a PNG → the avatar swaps to the fetched image (a blob: URL) and the
  // Remove button appears.
  await section.locator('input[type="file"]').setInputFiles(AVATAR_FIXTURE)
  await expect(removeButton).toBeVisible()
  await expect(avatarImg).toHaveAttribute('src', /^blob:/)

  // Remove → back to the placeholder, Remove button gone.
  await removeButton.click()
  await expect(removeButton).toHaveCount(0)
  await expect(avatarImg).toHaveAttribute('src', /avatar-placeholder\.png$/)
})
