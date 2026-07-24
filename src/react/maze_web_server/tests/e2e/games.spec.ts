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

test('creating a game with a duplicate name is rejected', async ({ page }) => {
  await login(page)

  const name = `Dup ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  let wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeHidden()
  await expect(page.locator('.game-list-item', { hasText: name })).toHaveCount(1)

  // A second game with the same name is refused (the server / mock 409s), so the
  // create wizard stays open and no duplicate row is added.
  await page.getByRole('button', { name: 'New game' }).click()
  wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeVisible()
  await wizard.getByRole('button', { name: 'Cancel' }).click()
  await expect(page.locator('.game-list-item', { hasText: name })).toHaveCount(1)
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

  // Clicking the row (on its name, not an action button) opens the editor.
  await page.locator('.game-list-item', { hasText: name }).getByText(name).click()

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

test('Access stages and un-stages a grantee via the username people-picker', async ({ page }) => {
  await login(page)

  const name = `Shareable ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: /^Access:/ })
  await expect(dialog).toBeVisible()
  // A fresh game is private; the people-picker only appears once the "Specific
  // people" tier is chosen.
  await dialog.getByRole('radio', { name: /Specific people/ }).click()
  await expect(dialog.getByText('No one added yet.')).toBeVisible()

  // Search the username lookup ("bob" is not a prefix of any other mock user, so
  // the Add/Remove button names stay unambiguous) and stage.
  await dialog.getByLabel('Add user').fill('bob')
  await dialog.getByRole('button', { name: 'Add bob' }).click()
  await expect(dialog.getByRole('button', { name: 'Remove bob' })).toBeVisible()

  // Un-staging returns to the empty state (nothing persists until Save).
  await dialog.getByRole('button', { name: 'Remove bob' }).click()
  await expect(dialog.getByText('No one added yet.')).toBeVisible()

  await dialog.getByRole('button', { name: 'Cancel' }).click()
  await expect(dialog).toBeHidden()
})

test('Leaderboard opens the board modal showing the board', async ({ page }) => {
  await login(page)

  const name = `Boarded ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Leaderboard for ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: `Leaderboard: ${name}` })
  await expect(dialog).toBeVisible()
  // Every game has a board (a private game's is owner-only); the modal shows it.
  await expect(dialog.getByRole('tab', { name: /fastest time/i })).toBeVisible()

  await dialog.getByRole('button', { name: 'Close' }).click()
  await expect(dialog).toBeHidden()
})

test('Delete removes a game after confirmation', async ({ page }) => {
  await login(page)

  const name = `Doomed ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(name)).toBeVisible()

  await page.getByRole('button', { name: `Delete ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: 'Delete Game' })
  await expect(dialog).toBeVisible()
  await dialog.getByRole('button', { name: 'Delete' }).click()

  // Assert on the row's own action button (unambiguous — the name also appears
  // transiently in the confirm dialog's message text).
  await expect(page.getByRole('button', { name: `Delete ${name}` })).toBeHidden()
  await page.reload()
  await expect(page.getByRole('button', { name: `Delete ${name}` })).toBeHidden()
})

test('the access modal updates a game badge from Just me to Specific people', async ({ page }) => {
  await login(page)

  const name = `Badged ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()

  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row.getByText('Just me')).toBeVisible()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: `Access: ${name}` })
  await dialog.getByRole('radio', { name: /Specific people/ }).click()
  await dialog.getByLabel('Add user').fill('bob')
  await dialog.getByRole('button', { name: 'Add bob' }).click()
  await expect(dialog.getByRole('button', { name: 'Remove bob' })).toBeVisible()
  await dialog.getByRole('button', { name: 'Save' }).click()

  // Save committed the tier + list, and the row reloaded to the shared tier.
  await expect(row.getByText('Specific people')).toBeVisible()
  await expect(row.getByText('Just me')).toBeHidden()
})

test('New game wizard steps through General → Layout and blocks an invalid size', async ({ page }) => {
  await login(page)

  await page.getByRole('button', { name: 'New game' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Game' })
  await dialog.getByLabel('Name').fill('Invalid')
  // General → Layout (Layout is the second step, right after General).
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

  // Raise the count (top of Layout) → Finish Cell appears at the bottom of Objects.
  await dialog.getByRole('tab', { name: 'Layout' }).click()
  await dialog.getByRole('spinbutton', { name: 'Number of Levels' }).fill('3')
  await dialog.getByRole('tab', { name: 'Objects' }).click()
  await expect(dialog.getByLabel('Finish Cell')).toBeVisible()

  // Back to a single level hides it again.
  await dialog.getByRole('tab', { name: 'Layout' }).click()
  await dialog.getByRole('spinbutton', { name: 'Number of Levels' }).fill('1')
  await dialog.getByRole('tab', { name: 'Objects' }).click()
  await expect(dialog.getByLabel('Finish Cell')).toBeHidden()
})

test('editing a game lets you upload then remove its image', async ({ page }) => {
  await login(page)

  // Create a game to edit (a new game has no image control — no id yet).
  const name = `Imaged ${Date.now()}`
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeHidden()

  // Open Edit → the Details tab shows the image control: placeholder + Upload.
  await page.getByRole('button', { name: `Edit ${name}` }).click()
  const editor = page.getByRole('dialog', { name: 'Edit Game' })
  const image = editor.locator('.game-image-editor')
  const preview = image.locator('.game-thumb-base')
  await expect(preview).toHaveAttribute('src', /workshop-game\.svg$/)
  await expect(image.getByRole('button', { name: /^Remove$/ })).toHaveCount(0)

  // Upload a real PNG → the preview swaps to the fetched image, a status shows,
  // and Change/Remove appear.
  await image.locator('input[type="file"]').setInputFiles('public/images/avatar-placeholder.png')
  await expect(preview).toHaveAttribute('src', /^blob:/)
  await expect(image.getByText('Image updated')).toBeVisible()
  await expect(image.getByRole('button', { name: /^Remove$/ })).toBeVisible()

  // The image is a separate resource, so Cancelling the editor keeps it — the row
  // thumbnail now shows it.
  await expect(editor.getByRole('button', { name: 'Save' })).toHaveCount(0)
  await editor.getByRole('button', { name: 'Close' }).click()
  await expect(editor).toBeHidden()
  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row.locator('.game-thumb-base')).toHaveAttribute('src', /^blob:/)

  // Reopen and Remove → back to the placeholder.
  await page.getByRole('button', { name: `Edit ${name}` }).click()
  const image2 = page.getByRole('dialog', { name: 'Edit Game' }).locator('.game-image-editor')
  await image2.getByRole('button', { name: /^Remove$/ }).click()
  await expect(image2.getByText('Image removed')).toBeVisible()
  await expect(image2.locator('.game-thumb-base')).toHaveAttribute('src', /workshop-game\.svg$/)
})
