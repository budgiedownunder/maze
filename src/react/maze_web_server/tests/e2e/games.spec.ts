import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
  await page.goto('/workshop/games')
}

test('Preview stashes the config and opens the game host in a new tab', async ({ page, context }) => {
  await login(page)

  await page.getByRole('button', { name: 'New game' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Game' })
  await dialog.getByLabel('Name').fill('Previewable')

  // Preview is offered on the footer and enabled with a valid (default) config.
  const preview = dialog.getByRole('button', { name: 'Preview' })
  await expect(preview).toBeEnabled()

  const popupPromise = context.waitForEvent('page')
  await preview.click()
  const popup = await popupPromise
  // Opened the host in preview mode (new tab, so the editor survives).
  expect(popup.url()).toContain('/game/?preview=1')
  await popup.close()

  // The handoff payload is stashed for the host to read — unseeded (a new game).
  const payload = await page.evaluate(() => JSON.parse(localStorage.getItem('gameDefinitionPreview') || 'null'))
  expect(payload.seeded).toBe(false)
  expect(payload.config).toMatchObject({ rows: 8, cols: 8, title: 'Previewable' })

  await dialog.getByRole('button', { name: 'Cancel' }).click()
})

test('New game wizard creates a definition that survives a reload', async ({ page }) => {
  await login(page)

  // A unique name keeps the assertion independent of anything a prior test in
  // this file left in the mock store (which persists across reloads).
  const name = `Tower ${Date.now()}`

  await page.getByRole('button', { name: 'New game' }).click()

  const dialog = page.getByRole('dialog', { name: 'New Game' })
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
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Edit ${name}` }).click()

  // Tabs mode: no wizard navigation, Save instead of Finish, name hydrated.
  const editor = page.getByRole('dialog', { name: 'Edit Game' })
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

test('Share adds and removes a grantee via the username people-picker', async ({ page }) => {
  await login(page)

  const name = `Shareable ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Share ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: /^Share:/ })
  await expect(dialog).toBeVisible()
  await expect(dialog.getByText('No one has access yet.')).toBeVisible()

  // Search the username lookup ("bob" is not a prefix of any other mock user, so
  // the Add/Remove button names stay unambiguous) and grant.
  await dialog.getByLabel('Add user').fill('bob')
  await dialog.getByRole('button', { name: 'Add bob' }).click()
  await expect(dialog.getByRole('button', { name: 'Remove bob' })).toBeVisible()

  // Revoke returns to the empty state.
  await dialog.getByRole('button', { name: 'Remove bob' }).click()
  await expect(dialog.getByText('No one has access yet.')).toBeVisible()

  await dialog.getByRole('button', { name: 'Close' }).click()
  await expect(dialog).toBeHidden()
})

test('New game wizard steps through General → Layout and blocks an invalid size', async ({ page }) => {
  await login(page)

  await page.getByRole('button', { name: 'New game' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Game' })
  await dialog.getByLabel('Name').fill('Invalid')
  // General → Scene → Layout (Scene now precedes Layout).
  await dialog.getByRole('button', { name: 'Next' }).click()
  await dialog.getByRole('button', { name: 'Next' }).click()

  await expect(dialog.getByLabel('Rows')).toBeVisible()
  await dialog.getByLabel('Rows').fill('2')
  await expect(dialog.getByRole('alert')).toHaveText('Rows must be a whole number of 3 or more.')
  await expect(dialog.getByRole('button', { name: 'Finish' })).toBeDisabled()

  await dialog.getByRole('button', { name: 'Cancel' }).click()
  await expect(dialog).toBeHidden()
})

test('the multi-level controls appear only when the level count is raised above 1', async ({ page }) => {
  await login(page)

  await page.getByRole('button', { name: 'New game' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Game' })
  await dialog.getByLabel('Name').fill('Stacked')

  // There is no Levels tab; the settings are distributed across the other tabs.
  await expect(dialog.getByRole('tab', { name: 'Levels' })).toBeHidden()

  // Single-level: the Objects tab has no Finish Cell.
  await dialog.getByRole('tab', { name: 'Objects' }).click()
  await expect(dialog.getByLabel('Finish Cell')).toBeHidden()

  // Raise the count on General → Finish Cell appears at the bottom of Objects.
  await dialog.getByRole('tab', { name: 'General' }).click()
  await dialog.getByRole('spinbutton', { name: 'Number of Levels' }).fill('3')
  await dialog.getByRole('tab', { name: 'Objects' }).click()
  await expect(dialog.getByLabel('Finish Cell')).toBeVisible()

  // Back to a single level hides it again.
  await dialog.getByRole('tab', { name: 'General' }).click()
  await dialog.getByRole('spinbutton', { name: 'Number of Levels' }).fill('1')
  await dialog.getByRole('tab', { name: 'Objects' }).click()
  await expect(dialog.getByLabel('Finish Cell')).toBeHidden()
})
