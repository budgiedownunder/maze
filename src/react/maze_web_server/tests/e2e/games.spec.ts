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

test('Edit opens the tabs editor over an existing game and Save persists the change', async ({ page }) => {
  await login(page)

  const name = `Spire ${Date.now()}`
  const renamed = `${name} v2`

  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Edit ${name}` }).click()

  // Tabs mode: no wizard navigation, Save instead of Finish, name hydrated.
  const editor = page.getByRole('dialog', { name: 'Edit game' })
  await expect(editor).toBeVisible()
  await expect(editor.getByRole('button', { name: 'Next' })).toBeHidden()
  await expect(editor.getByRole('button', { name: 'Back' })).toBeHidden()
  await expect(editor.getByLabel('Name')).toHaveValue(name)

  await editor.getByLabel('Name').fill(renamed)
  await editor.getByRole('button', { name: 'Save' }).click()

  await expect(editor).toBeHidden()
  await expect(page.getByText(renamed)).toBeVisible()

  await page.reload()
  await expect(page.getByText(renamed)).toBeVisible()
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
