import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

// The dev:mock backend grants admin to this email, so a game can be featured.
async function loginAsAdmin(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('admin@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

test('the 3D Games hub shows the four browse tiles and Featured opens empty', async ({ page }) => {
  await login(page)
  await page.goto('/play-3d')
  await expect(page.getByRole('heading', { name: /^featured$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^my games$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^shared with me$/i })).toBeVisible()
  await expect(page.getByRole('heading', { name: /^community$/i })).toBeVisible()

  await page.getByRole('button', { name: /featured/i }).click()
  await expect(page).toHaveURL(/\/play-3d\/featured$/)
  await expect(page.getByText(/no featured games or collections yet/i)).toBeVisible()
})

test('the not-yet-built scopes show a coming-soon placeholder', async ({ page }) => {
  await login(page)
  await page.goto('/play-3d/my-games')
  await expect(page.getByRole('banner').getByText('My Games')).toBeVisible()
  await expect(page.getByText('Coming soon.')).toBeVisible()

  await page.goto('/play-3d/community')
  await expect(page.getByText('Coming soon.')).toBeVisible()
})

test('a featured game appears as a card on Featured with Play and a Leaderboard modal', async ({ page }) => {
  await loginAsAdmin(page)

  // Create a game and feature it via the Access modal (admin-only Featured tier).
  await page.goto('/workshop/games')
  await page.getByRole('button', { name: 'New Game' }).click()
  const wizard = page.getByRole('dialog', { name: 'New Game' })
  const name = `Feat ${Date.now()}`
  await wizard.getByLabel('Name').fill(name)
  await wizard.getByRole('button', { name: 'Finish' }).click()
  await expect(wizard).toBeHidden()

  await page.getByRole('button', { name: `Access for ${name}` }).click()
  const access = page.getByRole('dialog', { name: /^Access:/ })
  await access.getByRole('radio', { name: /Featured/ }).click()
  await access.getByRole('button', { name: 'Save' }).click()
  await expect(access).toBeHidden()

  // It now shows as a gallery card on the Featured browse page.
  await page.goto('/play-3d/featured')
  const card = page.locator('.play3d-card', { hasText: name })
  await expect(card).toBeVisible()
  await expect(card.getByRole('button', { name: `Play ${name}` })).toBeVisible()

  // Leaderboard opens the board modal for that game.
  await card.getByRole('button', { name: `Leaderboard for ${name}` }).click()
  await expect(page.getByRole('dialog', { name: `Leaderboard: ${name}` })).toBeVisible()
})

test('a featured multi-game Arcade collection opens the picker on Featured', async ({ page }) => {
  await loginAsAdmin(page)
  const stamp = Date.now()
  const g1 = `GA ${stamp}`
  const g2 = `GB ${stamp}`
  const colName = `Set ${stamp}`

  // Two games.
  for (const gameName of [g1, g2]) {
    await page.goto('/workshop/games')
    await page.getByRole('button', { name: 'New Game' }).click()
    const wiz = page.getByRole('dialog', { name: 'New Game' })
    await wiz.getByLabel('Name').fill(gameName)
    await wiz.getByRole('button', { name: 'Finish' }).click()
    await expect(wiz).toBeHidden()
  }

  // A collection containing both (Arcade is the default play mode).
  await page.goto('/workshop/game-collections')
  await page.getByRole('button', { name: '+ New Game Collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Game Collection' })
  await create.getByLabel('Name').fill(colName)
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(create).toBeHidden()

  await page.getByRole('button', { name: `Edit ${colName}` }).click()
  const edit = page.getByRole('dialog', { name: 'Edit Collection' })
  await edit.getByLabel('Add game').fill(g1)
  await edit.getByRole('button', { name: `Add ${g1}` }).click()
  await edit.getByLabel('Add game').fill(g2)
  await edit.getByRole('button', { name: `Add ${g2}` }).click()
  await edit.getByRole('button', { name: 'Save' }).click()
  await expect(edit).toBeHidden()

  // Feature it (admin-only Featured tier).
  await page.getByRole('button', { name: `Access for ${colName}` }).click()
  const access = page.getByRole('dialog', { name: /^Access:/ })
  await access.getByRole('radio', { name: /Featured/ }).click()
  await access.getByRole('button', { name: 'Save' }).click()
  await expect(access).toBeHidden()

  // On Featured, the collection card's Play opens the Arcade picker of its games.
  await page.goto('/play-3d/featured')
  const card = page.locator('.play3d-card', { hasText: colName })
  await card.getByRole('button', { name: `Play ${colName}` }).click()
  const picker = page.getByRole('dialog', { name: `Play: ${colName}` })
  await expect(picker).toBeVisible()
  await expect(picker.getByText(g1)).toBeVisible()
  await expect(picker.getByText(g2)).toBeVisible()
  await picker.getByRole('button', { name: 'Cancel' }).click()
  await expect(picker).toBeHidden()
})

test('a featured Campaign collection opens the ordered progression on Featured', async ({ page }) => {
  await loginAsAdmin(page)
  const stamp = Date.now()
  const g1 = `CA ${stamp}`
  const g2 = `CB ${stamp}`
  const colName = `Camp ${stamp}`

  for (const gameName of [g1, g2]) {
    await page.goto('/workshop/games')
    await page.getByRole('button', { name: 'New Game' }).click()
    const wiz = page.getByRole('dialog', { name: 'New Game' })
    await wiz.getByLabel('Name').fill(gameName)
    await wiz.getByRole('button', { name: 'Finish' }).click()
    await expect(wiz).toBeHidden()
  }

  // A Campaign collection (set the play mode at create) containing both games.
  await page.goto('/workshop/game-collections')
  await page.getByRole('button', { name: '+ New Game Collection' }).click()
  const create = page.getByRole('dialog', { name: 'New Game Collection' })
  await create.getByLabel('Name').fill(colName)
  await create.getByLabel('Play mode').selectOption('campaign')
  await create.getByRole('button', { name: 'Create' }).click()
  await expect(create).toBeHidden()

  await page.getByRole('button', { name: `Edit ${colName}` }).click()
  const edit = page.getByRole('dialog', { name: 'Edit Collection' })
  await edit.getByLabel('Add game').fill(g1)
  await edit.getByRole('button', { name: `Add ${g1}` }).click()
  await edit.getByLabel('Add game').fill(g2)
  await edit.getByRole('button', { name: `Add ${g2}` }).click()
  await edit.getByRole('button', { name: 'Save' }).click()
  await expect(edit).toBeHidden()

  await page.getByRole('button', { name: `Access for ${colName}` }).click()
  const access = page.getByRole('dialog', { name: /^Access:/ })
  await access.getByRole('radio', { name: /Featured/ }).click()
  await access.getByRole('button', { name: 'Save' }).click()
  await expect(access).toBeHidden()

  // On Featured, Play opens the ordered campaign modal: nothing played yet, so the
  // first level is current (Play) and the second is locked.
  await page.goto('/play-3d/featured')
  const card = page.locator('.play3d-card', { hasText: colName })
  await card.getByRole('button', { name: `Play ${colName}` }).click()
  const modal = page.getByRole('dialog', { name: `Play: ${colName}` })
  await expect(modal).toBeVisible()
  await expect(modal.getByRole('button', { name: `Play ${g1}` })).toBeVisible()
  await expect(modal.getByRole('button', { name: `Locked: ${g2}` })).toBeVisible()
  await expect(modal.getByRole('button', { name: 'Continue' })).toBeVisible()
  await modal.getByRole('button', { name: 'Cancel' }).click()
  await expect(modal).toBeHidden()
})
