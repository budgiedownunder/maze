import { test, expect, type Page, devices } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
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
    await page.getByRole('button', { name: 'Play', exact: true }).click()
    await expect(page).toHaveURL(/\/play\//)
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('direct URL loads game page (SPA deep-link)', async ({ page }) => {
    await page.goto('/play/maze-0001')
    await expect(page.getByAltText('Player')).toBeVisible()
  })

  test('walking into enemies decrements HP and HP=0 shows the You died popup', async ({ page }) => {
    // EnemyGauntlet maze grid: ['S', 'E', 'E', 'E', 'F'] — three enemies in a
    // row directly between start and finish. Each Move-right collides the
    // player into another enemy; HP starts at 3 and drains to 0 on the third.
    await page.goto('/play/maze-enemy-gauntlet')
    await expect(page.getByAltText('Player')).toBeVisible()

    const hpHud = page.getByLabel('Health')
    await expect(hpHud).toBeVisible()
    // HP starts at 3/3 — all three hearts filled (alt="Health"), zero dimmed.
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(3)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(0)

    // Walk into the first enemy — HP goes 3 → 2 (one dimmed heart).
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(2)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(1)

    // Second enemy — HP goes 2 → 1.
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(1)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(2)

    // Third enemy — HP drains to 0, the move returns Killed, and the result
    // popup shows "You died!".
    await page.keyboard.press('ArrowRight')
    await expect(page.getByText('You died!')).toBeVisible()
  })

  test('Play Again on the result popup restarts the maze', async ({ page }) => {
    // Die on the gauntlet, then Play Again restarts: the popup closes and HP is
    // back to full (3/3) on a fresh game from the same definition.
    await page.goto('/play/maze-enemy-gauntlet')
    await expect(page.getByAltText('Player')).toBeVisible()
    for (let i = 0; i < 3; i++) {
      await page.keyboard.press('ArrowRight')
      await page.waitForTimeout(150)
    }
    await expect(page.getByText('You died!')).toBeVisible()

    await page.getByRole('button', { name: 'Play Again' }).click()

    await expect(page.getByText('You died!')).toBeHidden()
    const hpHud = page.getByLabel('Health')
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(3)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(0)
  })

  test('enemies keep advancing on their move period while the player presses keys (tick loop not reset by moves)', async ({ page }) => {
    // EnemyGauntlet grid ['S','E','E','E','F']: the player starts at column 0
    // with an enemy on the adjacent cell. Pressing ArrowLeft is blocked at the
    // left edge, so the player never leaves the start cell — but every keypress
    // still fires move() -> scheduleWake(). The adjacent enemy must still commit
    // onto the player on its move period (~1500ms) and deal damage. Before the
    // tick-loop fix, each move reset the enemy countdown, so a player holding a
    // key froze the enemies and was never hit.
    await page.goto('/play/maze-enemy-gauntlet')
    await expect(page.getByAltText('Player')).toBeVisible()

    const hpHud = page.getByLabel('Health')
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(3)

    // Simulate a held key: repeated blocked moves, faster than one enemy period.
    const deadline = Date.now() + 5000
    let damaged = false
    while (Date.now() < deadline) {
      await page.keyboard.press('ArrowLeft')
      await page.waitForTimeout(130)
      if (await hpHud.getByAltText('Lost health', { exact: true }).count() > 0) { damaged = true; break }
    }
    expect(damaged).toBe(true)
  })

  test('walking onto a health pickup below max HP heals and removes the in-grid symbol', async ({ page }) => {
    // EnemyHealth grid ['S','E','H','F']: collide with the enemy to drop to 2/3,
    // then walk onto the health pickup to heal back to 3/3. The consumed pickup's
    // in-grid symbol must disappear — it is rendered from the runtime's live
    // health-pickup list, not the static grid char (which never changes).
    await page.goto('/play/maze-enemy-health')
    await expect(page.getByAltText('Player')).toBeVisible()

    const grid = page.locator('.maze-grid-container')
    const hpHud = page.getByLabel('Health')

    // The pickup symbol shows in the grid and HP starts full (3/3).
    await expect(grid.getByAltText('Health')).toBeVisible()
    await expect(hpHud.getByAltText('Health', { exact: true })).toHaveCount(3)

    // Step onto the enemy: HP 3 → 2.
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(1)

    // Step onto the health pickup: HP 2 → 3 and the in-grid symbol is consumed.
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(hpHud.getByAltText('Lost health', { exact: true })).toHaveCount(0)
    await expect(grid.getByAltText('Health')).toHaveCount(0)
  })

  test('collecting a key opens a door and completes the maze', async ({ page }) => {
    // KeyDoor maze grid: ['S', 'K', 'D', 'F']
    await page.goto('/play/maze-keydoor')
    await expect(page.getByAltText('Player')).toBeVisible()

    // The key shows on the grid and the bag starts empty.
    await expect(page.locator('.maze-grid-container').getByAltText('Key')).toBeVisible()
    await expect(page.locator('.maze-bag')).toContainText('empty')

    // Step onto the key — it is auto-collected on walk-over, no button press.
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    // The key is now in the bag.
    await expect(page.locator('.maze-bag').getByAltText('Key')).toBeVisible()

    // Move toward the locked door — begins unlocking; the door opens over ~1s.
    await page.keyboard.press('ArrowRight')
    await expect(page.getByAltText('Door')).toBeHidden({ timeout: 3000 })

    // Pass through the now-open door and reach the finish.
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await page.keyboard.press('ArrowRight')
    await page.waitForTimeout(150)
    await expect(page.getByText('You win!')).toBeVisible()
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

  test('MazePage toolbar shows a Play in 3D button', async ({ page }) => {
    await page.goto('/mazes')
    await page.locator('.maze-list-item').first().locator('.maze-item-text').click()
    await expect(page.locator('.maze-grid-container')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Play in 3D' })).toBeVisible()
  })

  test('3D play button on Mazes list opens the custom-launch modal, then Play navigates to /game/?id=...', async ({ page }) => {
    await page.route(/\/game\//, route => route.fulfill({
      contentType: 'text/html',
      body: '<html><body>stub</body></html>',
    }))
    await page.goto('/mazes')
    await page.getByRole('button', { name: 'Play in 3D Alpha', exact: true }).click()
    const modal = page.getByRole('dialog', { name: /Play 3D — customise launch/i })
    await expect(modal).toBeVisible()
    await modal.getByRole('button', { name: 'Play', exact: true }).click()
    await page.waitForURL(/\/game\/\?id=/)
  })
})

test('unauthenticated visit to /game/ redirects to login', async ({ page }) => {
  await page.goto('/game/')
  await expect(page).not.toHaveURL(/\/game\//)
})

// ──────────────────────────────────────────────────────────────
// Mobile / touch (Pixel 7 — coarse pointer)
// ──────────────────────────────────────────────────────────────

test.describe('MazeGamePage — mobile (Pixel 7)', () => {
  // Spread device settings but omit defaultBrowserType — Playwright disallows it inside a describe group.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- the rename is the omission
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
