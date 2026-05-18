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

test.describe('Game host user-edited maze launch (?id=...)', () => {
  // The /game/?id=<mazeId> path forces wallType: 'brick',
  // landmarks.wallTint: false, and landmarks.wallMaterialVariation:
  // false so the user's own maze layout reads cleanly without
  // per-quadrant material flair or per-cell tint variation. This test
  // stubs the wasm module to capture the JSON payload the host page
  // sends to start_with_config and asserts the overrides are present.
  test('sends wallType=brick, wallTint=false and wallMaterialVariation=false to start_with_config', async ({ page }) => {
    // Stub the wasm JS module — a tiny shim that captures whatever
    // payload start_with_config receives so the test can inspect it.
    await page.route('**/maze_game_bevy_wasm.js**', (route) => {
      route.fulfill({
        contentType: 'application/javascript',
        body: `
          export default async function init() {}
          export function start() {}
          export function start_with_config(json) {
            window.__lastStartConfigPayload = json;
          }
        `,
      })
    })
    // Fulfill the wasm binary fetch with an empty 200 — the module
    // script does `await fetch(WASM_URL)` BEFORE the ?id branch runs,
    // so aborting it would throw early and never reach
    // start_with_config. The stubbed init() ignores its argument so
    // the empty body is harmless.
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    // Stub the maze fetch with a tiny synthetic maze so the host page
    // builds + dispatches the start_with_config payload.
    await page.route('**/api/v1/mazes/test-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          definition: { grid: [['S', ' ', 'F']] },
        }),
      })
    })
    await page.goto('/game/index.html?t=fake&id=test-id')
    await page.waitForFunction(
      () => typeof (window as unknown as { __lastStartConfigPayload?: string }).__lastStartConfigPayload === 'string'
    )
    const payload = await page.evaluate(() =>
      JSON.parse((window as unknown as { __lastStartConfigPayload: string }).__lastStartConfigPayload)
    )
    expect(payload.wallType).toBe('brick')
    expect(payload.landmarks.wallTint).toBe(false)
    expect(payload.landmarks.wallMaterialVariation).toBe(false)
    expect(typeof payload.mazeJson).toBe('string')
    const mazeDef = JSON.parse(payload.mazeJson)
    expect(mazeDef.grid).toEqual([['S', ' ', 'F']])
  })
})
