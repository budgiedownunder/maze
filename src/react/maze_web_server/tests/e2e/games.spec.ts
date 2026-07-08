import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
  await page.goto('/games')
}

test('New game wizard creates a definition that survives a reload', async ({ page }) => {
  await login(page)

  // A unique name keeps the assertion independent of anything a prior test in
  // this file left in the mock store (which persists across reloads).
  const name = `Tower ${Date.now()}`

  await page.getByRole('button', { name: 'New game' }).click()

  const dialog = page.getByRole('dialog', { name: 'New game' })
  await expect(dialog).toBeVisible()
  // Early Finish: offered on the first step, disabled until the name is entered.
  await expect(dialog.getByRole('button', { name: 'Finish' })).toBeDisabled()

  await dialog.getByLabel('Name').fill(name)
  await expect(dialog.getByRole('button', { name: 'Finish' })).toBeEnabled()
  await dialog.getByRole('button', { name: 'Finish' }).click()

  await expect(dialog).toBeHidden()
  await expect(page.getByText(name)).toBeVisible()

  await page.reload()
  await expect(page.getByText(name)).toBeVisible()
})

test('New game wizard steps through Details → Generation and blocks an invalid size', async ({ page }) => {
  await login(page)

  await page.getByRole('button', { name: 'New game' }).click()
  const dialog = page.getByRole('dialog', { name: 'New game' })
  await dialog.getByLabel('Name').fill('Invalid')
  await dialog.getByRole('button', { name: 'Next' }).click()

  await expect(dialog.getByLabel('Rows')).toBeVisible()
  await dialog.getByLabel('Rows').fill('2')
  await expect(dialog.getByRole('alert')).toHaveText('Rows must be a whole number of 3 or more.')
  await expect(dialog.getByRole('button', { name: 'Finish' })).toBeDisabled()

  await dialog.getByRole('button', { name: 'Cancel' }).click()
  await expect(dialog).toBeHidden()
})
