import { test, expect } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  // Log in to reach the mazes page
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
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

test('Home menu item navigates back to /mazes from /account', async ({ page }) => {
  // First go to /account so the back-to-Home navigation has somewhere to come from.
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /my account/i }).click()
  await expect(page).toHaveURL(/\/account/)

  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /^home$/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
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
