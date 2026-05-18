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
  // Common test setup: stub the wasm JS module so we can capture the
  // JSON payload start_with_config receives, fulfill the wasm binary
  // with an empty 200 so the module's `await fetch(WASM_URL)` succeeds,
  // and stub the maze fetch with a synthetic maze + name.
  async function stubGameHost(page: Page, mazeName: string) {
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
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    await page.route('**/api/v1/mazes/test-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'test-id',
          name: mazeName,
          definition: { grid: [['S', ' ', 'F']] },
        }),
      })
    })
  }

  async function capturedPayload(page: Page) {
    await page.waitForFunction(
      () => typeof (window as unknown as { __lastStartConfigPayload?: string }).__lastStartConfigPayload === 'string'
    )
    return await page.evaluate(() =>
      JSON.parse((window as unknown as { __lastStartConfigPayload: string }).__lastStartConfigPayload)
    )
  }

  test('falls back to clean defaults when localStorage is empty', async ({ page }) => {
    await stubGameHost(page, 'My Maze')
    await page.goto('/game/index.html?t=fake&id=test-id')
    const payload = await capturedPayload(page)
    expect(payload.wallType).toBe('brick')
    expect(payload.landmarks.wallTint).toBe(false)
    expect(payload.landmarks.wallMaterialVariation).toBe(false)
    expect(payload.landmarks.deadEndObjects).toBe(true)
    expect(payload.landmarks.wallDecorations).toBe(true)
    expect(payload.landmarks.floorAccents).toBe(true)
    expect(payload.skyType).toBe('night')
    expect(payload.timerSeconds).toBe(60)
    expect(payload.mode).toBe('My Maze')
    expect(typeof payload.mazeJson).toBe('string')
    const mazeDef = JSON.parse(payload.mazeJson)
    expect(mazeDef.grid).toEqual([['S', ' ', 'F']])
  })

  test('reads localStorage settings written by the Play3dCustomLaunchModal', async ({ page }) => {
    await stubGameHost(page, 'My Maze')
    await page.addInitScript(() => {
      localStorage.setItem(
        'play3dCustomLaunchSettings',
        JSON.stringify({
          skyType: 'sunset',
          wallType: 'wood',
          wallTint: true,
          wallMaterialVariation: false,
          deadEndObjects: false,
          wallDecorations: true,
          floorAccents: false,
          timerSeconds: 240,
        })
      )
    })
    await page.goto('/game/index.html?t=fake&id=test-id')
    const payload = await capturedPayload(page)
    expect(payload.skyType).toBe('sunset')
    expect(payload.wallType).toBe('wood')
    expect(payload.timerSeconds).toBe(240)
    expect(payload.landmarks.wallTint).toBe(true)
    expect(payload.landmarks.wallMaterialVariation).toBe(false)
    expect(payload.landmarks.deadEndObjects).toBe(false)
    expect(payload.landmarks.wallDecorations).toBe(true)
    expect(payload.landmarks.floorAccents).toBe(false)
  })

  test('truncates long maze names so the status bar mode label fits the header', async ({ page }) => {
    // Status bar fits ~20 chars at the configured font; longer names
    // are truncated with an ellipsis so the label doesn't overflow.
    await stubGameHost(page, 'This is a really long maze name that will not fit')
    await page.goto('/game/index.html?t=fake&id=test-id')
    const payload = await capturedPayload(page)
    expect(payload.mode.length).toBeLessThanOrEqual(20)
    expect(payload.mode).toMatch(/…$/)
  })

  test('uses "Play" when the maze name is empty / whitespace', async ({ page }) => {
    await stubGameHost(page, '   ')
    await page.goto('/game/index.html?t=fake&id=test-id')
    const payload = await capturedPayload(page)
    expect(payload.mode).toBe('Play')
  })

  test('URL query params override localStorage (MAUI native-popup path)', async ({ page }) => {
    // The MAUI native Play3dCustomLaunchPopup writes the chosen settings
    // as URL query params (MAUI's WebView can't share the React SPA's
    // localStorage), and /game/index.html prefers URL params over
    // localStorage so MAUI's explicit per-launch choice always wins.
    await stubGameHost(page, 'My Maze')
    await page.addInitScript(() => {
      // Stale localStorage from a prior browser-only launch.
      localStorage.setItem(
        'play3dCustomLaunchSettings',
        JSON.stringify({
          skyType: 'night',
          wallType: 'brick',
          wallTint: false,
          wallMaterialVariation: false,
          deadEndObjects: true,
          wallDecorations: true,
          floorAccents: true,
          timerSeconds: 60,
        })
      )
    })
    // MAUI URL: explicit settings layered on top of localStorage.
    const params = new URLSearchParams({
      t: 'fake',
      id: 'test-id',
      skyType: 'day',
      wallType: 'cobblestone',
      wallTint: '1',
      wallMaterialVariation: '0',
      deadEndObjects: '0',
      wallDecorations: '0',
      floorAccents: '1',
      timerSeconds: '300',
    }).toString()
    await page.goto(`/game/index.html?${params}`)
    const payload = await capturedPayload(page)
    // URL wins for every field.
    expect(payload.skyType).toBe('day')
    expect(payload.wallType).toBe('cobblestone')
    expect(payload.timerSeconds).toBe(300)
    expect(payload.landmarks.wallTint).toBe(true)
    expect(payload.landmarks.wallMaterialVariation).toBe(false)
    expect(payload.landmarks.deadEndObjects).toBe(false)
    expect(payload.landmarks.wallDecorations).toBe(false)
    expect(payload.landmarks.floorAccents).toBe(true)
  })
})
