import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

test('navigating to / redirects to /login', async ({ page }) => {
  await page.goto('/')
  await expect(page).toHaveURL(/\/login/)
})

test('navigating to /mazes without auth redirects to /login', async ({ page }) => {
  await page.goto('/mazes')
  await expect(page).toHaveURL(/\/login/)
})

test('successful login navigates to /mazes', async ({ page }) => {
  await login(page)
})

test('sign out returns to /login', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /sign out/i }).click()
  await expect(page).toHaveURL(/\/login/)
})

test('navigating to /mazes after sign out redirects to /login', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /open menu/i }).click()
  await page.getByRole('menuitem', { name: /sign out/i }).click()
  await expect(page).toHaveURL(/\/login/)
  await page.goto('/mazes')
  await expect(page).toHaveURL(/\/login/)
})
