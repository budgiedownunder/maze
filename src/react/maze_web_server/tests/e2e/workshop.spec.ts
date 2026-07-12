import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
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
