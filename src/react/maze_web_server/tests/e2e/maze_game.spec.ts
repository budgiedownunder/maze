import { test, expect, type Page, devices } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Username').fill('testuser')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

async function navigateViaPlay(page: Page) {
  await page.goto('/mazes')
  await page.getByRole('button', { name: 'Play Alpha', exact: true }).click()
  await expect(page).toHaveURL(/\/play\//)
  await expect(page.getByAltText('Player')).toBeVisible()
}

async function completeMaze(page: Page) {
  // Alpha solution: Right, Right, Down, Down — 150ms gap clears 120ms move throttle
  for (const key of ['ArrowRight', 'ArrowRight', 'ArrowDown', 'ArrowDown']) {
    await page.keyboard.press(key)
    await page.waitForTimeout(150)
  }
}

// ──────────────────────────────────────────────────────────────
// Navigation entry points
// ──────────────────────────────────────────────────────────────

test.describe('MazeGamePage', () => {
  test.beforeEach(async ({ page }) => {
    await login(page)
  })

  test('MazesPage Play button navigates to /play/:id and shows player', async ({ page }) => {
    await page.goto('/mazes')
    await page.getByRole('button', { name: 'Play Alpha', exact: true }).click()
    await expect(page).toHaveURL(/\/play\//)
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('MazePage Play button (clean maze) navigates to /play/:id and shows player', async ({ page }) => {
    await page.goto('/mazes')
    await page.locator('.maze-list-item').first().locator('.maze-item-text').click()
    await expect(page.locator('.maze-grid-container')).toBeVisible()
    await page.getByRole('button', { name: 'Play' }).click()
    await expect(page).toHaveURL(/\/play\//)
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('direct URL loads game page (SPA deep-link)', async ({ page }) => {
    await page.goto('/play/maze-0001')
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  // ──────────────────────────────────────────────────────────────
  // Gameplay
  // ──────────────────────────────────────────────────────────────

  test('arrow key moves the player', async ({ page }) => {
    await navigateViaPlay(page)
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('WASD key moves the player', async ({ page }) => {
    await navigateViaPlay(page)
    await page.keyboard.press('d')
    await page.waitForTimeout(150)
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('visited cells show visited_dot after player leaves a non-start cell', async ({ page }) => {
    await navigateViaPlay(page)
    // Move right twice: start cell (0,0) shows start_flag when visited; (0,1) shows visited_dot once left
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(page.locator('img[src*="visited_dot"]')).toBeVisible()
  })

  test('completing the maze shows GameResultPopup', async ({ page }) => {
    await navigateViaPlay(page)
    await completeMaze(page)
    await expect(page.getByRole('dialog')).toBeVisible()
    await expect(page.getByAltText('Celebration')).toBeVisible()
    await expect(page.getByRole('dialog')).toContainText('You win!')
  })

  // ──────────────────────────────────────────────────────────────
  // GameResultPopup behaviour
  // ──────────────────────────────────────────────────────────────

  test('Close button dismisses the popup', async ({ page }) => {
    await navigateViaPlay(page)
    await completeMaze(page)
    await expect(page.getByRole('dialog')).toBeVisible()
    await page.getByRole('button', { name: 'Close' }).click()
    await expect(page.getByRole('dialog')).not.toBeVisible()
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('Escape does NOT dismiss the popup', async ({ page }) => {
    await navigateViaPlay(page)
    await completeMaze(page)
    await expect(page.getByRole('dialog')).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.getByRole('dialog')).toBeVisible()
  })
})

// ──────────────────────────────────────────────────────────────
// Mobile / touch (Pixel 7 — coarse pointer)
// ──────────────────────────────────────────────────────────────

test.describe('MazeGamePage — mobile (Pixel 7)', () => {
  const { defaultBrowserType: _ignored, ...pixel7 } = devices['Pixel 7']
  test.use(pixel7)

  test.beforeEach(async ({ page }) => {
    await login(page)
  })

  test('D-pad is visible and keyboard legend is hidden on touch device', async ({ page }) => {
    await page.goto('/play/maze-0001')
    await expect(page.getByAltText('Player')).toBeVisible()
    await expect(page.locator('[aria-label="D-pad"]')).toBeVisible()
    await expect(page.locator('.maze-shortcuts-hint')).toBeHidden()
  })

  test('D-pad button moves the player', async ({ page }) => {
    await page.goto('/mazes')
    await page.getByRole('button', { name: 'Play Alpha', exact: true }).click()
    await expect(page).toHaveURL(/\/play\//)
    await expect(page.getByAltText('Player')).toBeVisible()
    await page.getByRole('button', { name: 'Move right' }).click()
    await page.waitForTimeout(150)
    await expect(page.getByAltText('Player')).toBeVisible()
  })
})
