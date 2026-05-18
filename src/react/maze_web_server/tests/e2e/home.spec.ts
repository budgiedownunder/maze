import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('successful sign-in lands on the Home page with both tiles visible', async ({ page }) => {
  await login(page)
  await expect(page.getByRole('heading', { name: /play 3d/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /design & play/i })).toBeVisible()
})

test('clicking Design & Play tile navigates to /mazes', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: /design & play/i }).click()
  await expect(page).toHaveURL(/\/mazes$/)
})
