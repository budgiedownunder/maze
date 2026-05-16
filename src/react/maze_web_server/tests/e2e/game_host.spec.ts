import { test, expect, type Page, devices } from '@playwright/test'

// Tests for the /game/ host page (public/game/index.html) — the
// persistent corner pause button, the pause menu overlay, and the
// D-pad show/hide preference. These exercise the host-side JavaScript
// in isolation from Bevy: the WASM bundle is aborted at the network
// level so the module script fails fast, and the page's vanilla
// <script> wiring (which is independent of WASM load success) is
// driven by manually firing the `maze-game-paused` CustomEvent — the
// same event Bevy dispatches when the pause state changes.

async function loadGameHostStubbed(page: Page) {
  // Abort the WASM-related fetches so the module script throws fast.
  // The vanilla pause-menu wiring runs in its own <script> tag earlier
  // in the document and is unaffected by the module's failure.
  await page.route('**/maze_game_bevy_wasm_bg.wasm**', (r) => r.abort())
  await page.route('**/maze_game_bevy_wasm.js**', (r) => r.abort())
  // ?t=fake satisfies the auth-guard IIFE in index.html so the page
  // doesn't redirect to /.
  // Use the explicit index.html path so Vite's dev server serves the
  // static public/game/index.html rather than falling back to the SPA root.
  await page.goto('/game/index.html?t=fake&difficulty=easy')
  await expect(page.locator('#pause-menu')).toBeAttached()
}

async function firePaused(page: Page, paused: boolean) {
  await page.evaluate((p) => {
    window.dispatchEvent(new CustomEvent('maze-game-paused', { detail: { paused: p } }))
  }, paused)
}

test.describe('Game host pause menu', () => {
  test('pause menu starts hidden', async ({ page }) => {
    await loadGameHostStubbed(page)
    await expect(page.locator('#pause-menu')).toBeHidden()
  })

  test('paused event reveals the menu with Resume Restart and Hide D-pad', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await expect(page.locator('#pause-menu')).toBeVisible()
    await expect(page.locator('#pm-resume')).toHaveText('Resume')
    await expect(page.locator('#pm-restart')).toHaveText('Restart')
    await expect(page.locator('#pm-dpad-toggle')).toHaveText('Hide D-pad')
  })

  test('resumed event hides the menu', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await expect(page.locator('#pause-menu')).toBeVisible()
    await firePaused(page, false)
    await expect(page.locator('#pause-menu')).toBeHidden()
  })

  test('Restart button reloads the page', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    const reloadPromise = page.waitForEvent('load')
    await page.locator('#pm-restart').click()
    await reloadPromise
    // After reload the menu is hidden again (no paused event yet).
    await expect(page.locator('#pause-menu')).toBeHidden()
  })

})

test.describe('Game host D-pad toggle (touch)', () => {
  // The D-pad toggle is hidden on fine-pointer devices, so this group
  // emulates a coarse-pointer device (Pixel 7) to exercise it.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- the rename is the omission
  const { defaultBrowserType: _ignored, ...pixel7 } = devices['Pixel 7']
  test.use(pixel7)

  test('D-pad toggle persists across reload', async ({ page }) => {
    await loadGameHostStubbed(page)
    // Start state — body class absent, localStorage empty for this key.
    await expect(page.locator('body.dpad-hidden')).toHaveCount(0)

    // Open the pause menu and click Hide D-pad.
    await firePaused(page, true)
    await page.locator('#pm-dpad-toggle').click()
    await expect(page.locator('body.dpad-hidden')).toHaveCount(1)
    await expect(page.locator('#pm-dpad-toggle')).toHaveText('Show D-pad')
    expect(await page.evaluate(() => localStorage.getItem('dpadHidden'))).toBe('1')

    // Reload — the early boot-time script re-applies the body class
    // BEFORE #controls is parsed, so no flash and the D-pad stays hidden.
    await page.reload()
    await expect(page.locator('#pause-menu')).toBeAttached()
    await expect(page.locator('body.dpad-hidden')).toHaveCount(1)

    // Open menu again, toggle back to visible, persisted to localStorage.
    await firePaused(page, true)
    await expect(page.locator('#pm-dpad-toggle')).toHaveText('Show D-pad')
    await page.locator('#pm-dpad-toggle').click()
    await expect(page.locator('body.dpad-hidden')).toHaveCount(0)
    await expect(page.locator('#pm-dpad-toggle')).toHaveText('Hide D-pad')
    expect(await page.evaluate(() => localStorage.getItem('dpadHidden'))).toBe('0')
  })
})

test.describe('Game host pause menu desktop layout', () => {
  test('D-pad toggle is hidden on fine-pointer devices', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await expect(page.locator('#pause-menu')).toBeVisible()
    await expect(page.locator('#pm-dpad-toggle')).toBeHidden()
    // Resume and Restart are still available.
    await expect(page.locator('#pm-resume')).toBeVisible()
    await expect(page.locator('#pm-restart')).toBeVisible()
  })
})

test.describe('Game host persistent pause button (touch)', () => {
  // Spread device settings but omit defaultBrowserType — Playwright disallows
  // overriding the browser type inside a describe group.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- the rename is the omission
  const { defaultBrowserType: _ignored, ...pixel7 } = devices['Pixel 7']
  test.use(pixel7)

  test('touch-pause button is visible on coarse-pointer devices', async ({ page }) => {
    await loadGameHostStubbed(page)
    await expect(page.locator('#touch-pause-btn')).toBeVisible()
    await expect(page.locator('#touch-pause-btn')).toHaveAttribute('aria-label', 'Pause')
  })

  test('touch-pause button stays visible even when the D-pad is hidden', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await page.locator('#pm-dpad-toggle').click()
    await firePaused(page, false)
    // D-pad is gone, but the corner pause button is still there as the
    // sole touch path back to the pause menu.
    await expect(page.locator('#controls')).toBeHidden()
    await expect(page.locator('#touch-pause-btn')).toBeVisible()
  })

  test('paused event swaps the touch-pause icon and label', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await expect(page.locator('#touch-pause-btn')).toHaveAttribute('aria-label', 'Resume')
    await expect(page.locator('#touch-pause-btn img')).toHaveAttribute('src', /dpad_play\.png$/)
    await firePaused(page, false)
    await expect(page.locator('#touch-pause-btn')).toHaveAttribute('aria-label', 'Pause')
    await expect(page.locator('#touch-pause-btn img')).toHaveAttribute('src', /dpad_pause\.png$/)
  })
})

test.describe('Game host pause button hidden on desktop', () => {
  test('touch-pause button is hidden on fine-pointer devices', async ({ page }) => {
    await loadGameHostStubbed(page)
    await expect(page.locator('#touch-pause-btn')).toBeHidden()
  })
})
