import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
  await page.goto('/workshop/game-collections')
}

test('the collections page shows the empty state before any collection exists', async ({ page }) => {
  await login(page)
  await expect(page.getByRole('button', { name: '+ New collection' })).toBeVisible()
  await expect(page.getByText('No collections yet.')).toBeVisible()
})

test('New collection creates a collection that appears in the list', async ({ page }) => {
  await login(page)

  const name = `Campaign ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const dialog = page.getByRole('dialog', { name: 'New Collection' })
  await expect(dialog).toBeVisible()
  await dialog.getByLabel('Name').fill(name)
  await dialog.getByLabel('Description (optional)').fill('My best levels')
  await dialog.getByRole('button', { name: 'Create' }).click()

  await expect(dialog).toBeHidden()
  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row).toBeVisible()
  // A fresh collection is empty and private.
  await expect(row.getByText('0 games · Just me')).toBeVisible()

  // It survives a reload (the mock store is sessionStorage-backed).
  await page.reload()
  await expect(page.locator('.game-list-item', { hasText: name })).toBeVisible()
})

test('Edit renames a collection', async ({ page }) => {
  await login(page)

  const name = `Renamable ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(name)
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(page.locator('.game-list-item', { hasText: name })).toBeVisible()

  await page.getByRole('button', { name: `Edit ${name}` }).click()
  const edit = page.getByRole('dialog', { name: 'Edit Collection' })
  const renamed = `${name} v2`
  await edit.getByLabel('Name').fill(renamed)
  await edit.getByRole('button', { name: 'Save' }).click()

  await expect(page.locator('.game-list-item', { hasText: renamed })).toBeVisible()
  await page.reload()
  await expect(page.locator('.game-list-item', { hasText: renamed })).toBeVisible()
})

test('the access modal updates a collection summary from Just me to Everyone', async ({ page }) => {
  await login(page)

  const name = `Accessible ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(name)
  await create.getByRole('button', { name: 'Create' }).click()

  const row = page.locator('.game-list-item', { hasText: name })
  await expect(row.getByText(/Just me/)).toBeVisible()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: `Access: ${name}` })
  await dialog.getByRole('radio', { name: /Everyone/ }).click()
  await dialog.getByRole('button', { name: 'Save' }).click()

  // Save committed the tier; the row reloaded to the public tier.
  await expect(row.getByText(/Everyone/)).toBeVisible()
  await expect(row.getByText(/Just me/)).toBeHidden()
})

test('Edit adds a game to a collection and updates its count', async ({ page }) => {
  await login(page)

  // Create a game so the Edit modal's picker has something to offer.
  const gameName = `Tower ${Date.now()}`
  await page.goto('/workshop/games')
  await page.getByRole('button', { name: 'New game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  await wizard.getByLabel('Name').fill(gameName)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(page.getByText(gameName)).toBeVisible()

  // Create a collection (starts empty).
  await page.goto('/workshop/game-collections')
  const colName = `Campaign ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(colName)
  await create.getByRole('button', { name: 'Create' }).click()
  const row = page.locator('.game-list-item', { hasText: colName })
  await expect(row.getByText('0 games · Just me')).toBeVisible()

  // Edit → add the game via the picker → Save.
  await page.getByRole('button', { name: `Edit ${colName}` }).click()
  const edit = page.getByRole('dialog', { name: 'Edit Collection' })
  await edit.getByLabel('Add game').fill(gameName)
  await edit.getByRole('button', { name: `Add ${gameName}` }).click()
  await edit.getByRole('button', { name: 'Save' }).click()
  await expect(edit).toBeHidden()

  // The row's game count reflects the added member.
  await expect(row.getByText('1 game · Just me')).toBeVisible()
})

test('the Edit modal keeps Cancel/Save on-screen on a short window', async ({ page }) => {
  await page.setViewportSize({ width: 520, height: 340 })
  await login(page)

  const colName = `Short ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(colName)
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(page.locator('.game-list-item', { hasText: colName })).toBeVisible()

  await page.getByRole('button', { name: `Edit ${colName}` }).click()
  const edit = page.getByRole('dialog', { name: 'Edit Collection' })
  const save = edit.getByRole('button', { name: 'Save' })
  await expect(save).toBeVisible()
  // The footer stays within the viewport rather than being pushed below the fold.
  const box = await save.boundingBox()
  expect(box).not.toBeNull()
  expect(box!.y + box!.height).toBeLessThanOrEqual(340)
})

test('the Access modal keeps Cancel/Save on-screen on a short window', async ({ page }) => {
  await page.setViewportSize({ width: 520, height: 340 })
  await login(page)

  const name = `Shorty ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(name)
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(page.locator('.game-list-item', { hasText: name })).toBeVisible()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: `Access: ${name}` })
  // "Specific people" reveals the picker + grantee list, adding enough height to
  // exercise the scrolling body; the footer must still stay above the fold.
  await dialog.getByRole('radio', { name: /Specific people/ }).click()
  const save = dialog.getByRole('button', { name: 'Save' })
  await expect(save).toBeVisible()
  const box = await save.boundingBox()
  expect(box).not.toBeNull()
  expect(box!.y + box!.height).toBeLessThanOrEqual(340)
})

test('Delete removes a collection after confirmation', async ({ page }) => {
  await login(page)

  const name = `Doomed ${Date.now()}`
  await page.getByRole('button', { name: '+ New collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Collection' })
  await create.getByLabel('Name').fill(name)
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(page.getByRole('button', { name: `Delete ${name}` })).toBeVisible()

  await page.getByRole('button', { name: `Delete ${name}` }).click()
  const dialog = page.getByRole('dialog', { name: 'Delete Collection' })
  await dialog.getByRole('button', { name: 'Delete' }).click()

  // Assert on the row's own action button (unambiguous — the name also appears
  // transiently in the confirm dialog's message text).
  await expect(page.getByRole('button', { name: `Delete ${name}` })).toBeHidden()
  await page.reload()
  await expect(page.getByRole('button', { name: `Delete ${name}` })).toBeHidden()
})
