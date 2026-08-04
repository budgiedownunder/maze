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
  // doesn't redirect to /. No subject param is needed: the WASM abort makes
  // the module script throw at init() before the routing chain runs, and the
  // pause-menu wiring lives in a separate <script> that runs regardless.
  // Use the explicit index.html path so Vite's dev server serves the
  // static public/game/index.html rather than falling back to the SPA root.
  await page.goto('/game/index.html?t=fake')
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

  test('Restart asks for confirmation before reloading', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    // First tap shows the confirmation view, not a reload.
    await page.locator('#pm-restart').click()
    await expect(page.locator('#pm-confirm')).toBeVisible()
    await expect(page.locator('#pm-main')).toBeHidden()
    // Confirming reloads the page.
    const reloadPromise = page.waitForEvent('load')
    await page.locator('#pm-restart-confirm').click()
    await reloadPromise
    // After reload the menu is hidden again (no paused event yet).
    await expect(page.locator('#pause-menu')).toBeHidden()
  })

  test('Restart confirmation can be cancelled', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await page.locator('#pm-restart').click()
    await expect(page.locator('#pm-confirm')).toBeVisible()
    // Cancel returns to the main button list without reloading.
    await page.locator('#pm-restart-cancel').click()
    await expect(page.locator('#pm-main')).toBeVisible()
    await expect(page.locator('#pm-confirm')).toBeHidden()
    await expect(page.locator('#pm-resume')).toBeVisible()
  })

  test('reopening the menu resets to the main view after a cancelled restart', async ({ page }) => {
    await loadGameHostStubbed(page)
    await firePaused(page, true)
    await page.locator('#pm-restart').click()
    await expect(page.locator('#pm-confirm')).toBeVisible()
    // Resume out and pause again — the menu must reopen on the main list.
    await firePaused(page, false)
    await firePaused(page, true)
    await expect(page.locator('#pm-main')).toBeVisible()
    await expect(page.locator('#pm-confirm')).toBeHidden()
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

test.describe('Game host score submission (on win)', () => {
  // Drives the vanilla `maze-game-result` listener in index.html in
  // isolation from Bevy: the WASM bundle is aborted (the module script
  // fails fast, but the result-listener <script> runs regardless), and we
  // fire a synthetic `maze-game-result` CustomEvent — the same event Bevy
  // dispatches on completion. A route over POST /api/v1/scores captures the
  // submitted body.

  async function loadAndCaptureScores(page: Page, url: string) {
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (r) => r.abort())
    await page.route('**/maze_game_bevy_wasm.js**', (r) => r.abort())

    const posted: Array<Record<string, unknown>> = []
    await page.route('**/api/v1/scores', (route) => {
      const req = route.request()
      if (req.method() === 'POST') {
        posted.push(JSON.parse(req.postData() ?? '{}'))
        route.fulfill({
          status: 201,
          contentType: 'application/json',
          body: JSON.stringify({
            id: '00000000-0000-0000-0000-000000000001',
            user_id: '00000000-0000-0000-0000-000000000002',
            score: 0,
            elapsed_ms: 0,
            recorded_at: '2025-04-01T12:00:00Z',
          }),
        })
      } else {
        route.continue()
      }
    })

    await page.goto(url)
    await expect(page.locator('#pause-menu')).toBeAttached()
    return posted
  }

  async function fireResult(page: Page, detail: Record<string, unknown>) {
    await page.evaluate((d) => {
      window.dispatchEvent(new CustomEvent('maze-game-result', { detail: d }))
    }, detail)
  }

  test('submits the maze-id subject on a win', async ({ page }) => {
    const posted = await loadAndCaptureScores(page, '/game/index.html?t=fake&id=test-id')
    await fireResult(page, { outcome: 'win', score: 7, elapsedMs: 42137, rows: 3, cols: 3 })
    await expect.poll(() => posted.length).toBe(1)
    expect(posted[0]).toEqual({ maze_id: 'test-id', score: 7, elapsed_ms: 42137 })
  })

  test('does not submit on a loss', async ({ page }) => {
    const posted = await loadAndCaptureScores(page, '/game/index.html?t=fake&id=test-id')
    await fireResult(page, { outcome: 'lose', score: 0, elapsedMs: 5000, rows: 3, cols: 3 })
    // Give any (erroneous) submit a chance to fire before asserting none did.
    await page.waitForTimeout(200)
    expect(posted).toHaveLength(0)
  })

  test('does not submit when the run has no stable subject', async ({ page }) => {
    // No ?id and no ?def — nothing to key a board on.
    const posted = await loadAndCaptureScores(page, '/game/index.html?t=fake')
    await fireResult(page, { outcome: 'win', score: 4, elapsedMs: 1234, rows: 3, cols: 3 })
    await page.waitForTimeout(200)
    expect(posted).toHaveLength(0)
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
          // The host page imports these too; a missing export is an ES module
          // link error, which fails the whole module rather than one call.
          export function stop() {}
          export function live_bytes() { return 0 }
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

  test('leaves the diagnostics readout off unless ?mem=1 is present', async ({ page }) => {
    await stubGameHost(page, 'My Maze')
    await page.goto('/game/index.html?t=fake&id=test-id')
    const payload = await capturedPayload(page)
    expect(payload.debugMemory).toBe(false)
  })

  test('turns the diagnostics readout on for ?mem=1', async ({ page }) => {
    await stubGameHost(page, 'My Maze')
    await page.goto('/game/index.html?t=fake&id=test-id&mem=1')
    const payload = await capturedPayload(page)
    expect(payload.debugMemory).toBe(true)
  })

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
    expect(payload.enemyType).toBe('goblin')
    expect(payload.healthStyle).toBe('heart')
    expect(payload.timerSeconds).toBe(60)
    expect(payload.mode).toBe('My Maze')
    expect(typeof payload.mazeJson).toBe('string')
    const mazeDef = JSON.parse(payload.mazeJson)
    expect(mazeDef.grid).toEqual([['S', ' ', 'F']])
  })

  test('reads localStorage settings written by the MazeGameSettingsModal', async ({ page }) => {
    await stubGameHost(page, 'My Maze')
    await page.addInitScript(() => {
      localStorage.setItem(
        'mazeGameSettings',
        JSON.stringify({
          skyType: 'sunset',
          wallType: 'wood',
          doorStyle: 'portcullis',
          keyHolder: 'chest',
          enemyType: 'ghost',
          healthStyle: 'potion',
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
    expect(payload.doorStyle).toBe('portcullis')
    expect(payload.keyHolder).toBe('chest')
    expect(payload.enemyType).toBe('ghost')
    expect(payload.healthStyle).toBe('potion')
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
    // The MAUI native MazeGameSettingsPopup writes the chosen settings
    // as URL query params (MAUI's WebView can't share the React SPA's
    // localStorage), and /game/index.html prefers URL params over
    // localStorage so MAUI's explicit per-launch choice always wins.
    await stubGameHost(page, 'My Maze')
    await page.addInitScript(() => {
      // Stale localStorage from a prior browser-only launch.
      localStorage.setItem(
        'mazeGameSettings',
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
      doorStyle: 'dissolve',
      keyHolder: 'floating_key',
      enemyType: 'ghost',
      healthStyle: 'potion',
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
    expect(payload.doorStyle).toBe('dissolve')
    expect(payload.keyHolder).toBe('floating_key')
    expect(payload.enemyType).toBe('ghost')
    expect(payload.healthStyle).toBe('potion')
    expect(payload.timerSeconds).toBe(300)
    expect(payload.landmarks.wallTint).toBe(true)
    expect(payload.landmarks.wallMaterialVariation).toBe(false)
    expect(payload.landmarks.deadEndObjects).toBe(false)
    expect(payload.landmarks.wallDecorations).toBe(false)
    expect(payload.landmarks.floorAccents).toBe(true)
  })
})

test.describe('Game host stored game-definition launch (?def=...)', () => {
  // Stub the wasm JS module (capturing the start_with_config payload),
  // fulfill the wasm binary with an empty 200, and stub the play-fetch of
  // the definition. Mirrors the ?id launch harness above.
  async function stubDefHost(
    page: Page,
    body: { config: Record<string, unknown>; challengeKey: string; leaderboardTracked: boolean }
  ) {
    await page.route('**/maze_game_bevy_wasm.js**', (route) => {
      route.fulfill({
        contentType: 'application/javascript',
        body: `
          export default async function init() {}
          export function start() {}
          // The host page imports these too; a missing export is an ES module
          // link error, which fails the whole module rather than one call.
          export function stop() {}
          export function live_bytes() { return 0 }
          export function start_with_config(json) {
            window.__lastStartConfigPayload = json;
          }
        `,
      })
    })
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    await page.route('**/api/v1/game-definitions/def-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        // The play-fetch response: a flattened definition (with the effective
        // seed already spliced into config) plus the leaderboard subject/flag.
        body: JSON.stringify({
          id: 'def-id',
          ownerId: '00000000-0000-0000-0000-0000000000aa',
          name: 'Tower',
          description: null,
          imageUpdatedAt: null,
          visibility: 'public',
          seed: 7,
          rotation: 'static',
          config: body.config,
          createdAt: '2025-04-01T12:00:00Z',
          updatedAt: '2025-04-01T12:00:00Z',
          challengeKey: body.challengeKey,
          leaderboardTracked: body.leaderboardTracked,
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

  test('forwards the definition config verbatim to start_with_config', async ({ page }) => {
    await stubDefHost(page, {
      config: { timerSeconds: 90, mode: 'Tower', skyType: 'day', wallType: 'wood' },
      challengeKey: 'def:def-id',
      leaderboardTracked: false,
    })
    await page.goto('/game/index.html?t=fake&def=def-id')
    const payload = await capturedPayload(page)
    expect(payload.timerSeconds).toBe(90)
    expect(payload.mode).toBe('Tower')
    expect(payload.skyType).toBe('day')
    expect(payload.wallType).toBe('wood')
    // Unpublished preview → the config is marked un-tracked (no win banners).
    expect(payload.leaderboardTracked).toBe(false)
  })

  // A killed page runs no JavaScript on its way out, so the crash can only be
  // noticed on the load the browser performs afterwards — by a run sentinel that
  // outlived it. Without this the reload just starts the same game again.
  test.describe('crash containment', () => {
    const SENTINEL = 'mazeRunActive'

    async function launch(page: Page) {
      await stubDefHost(page, {
        config: { timerSeconds: 90, mode: 'Tower' },
        challengeKey: 'def:def-id',
        leaderboardTracked: false,
      })
      await page.goto('/game/index.html?t=fake&def=def-id')
    }

    const sentinel = (page: Page) =>
      page.evaluate((key) => sessionStorage.getItem(key), SENTINEL)

    // Leaves behind what a killed run leaves behind. Seeded once and not on any
    // later navigation, so a reload sees exactly what a real retry would: the
    // sentinel already cleared by the load that reported it.
    async function seedAbandonedRun(page: Page) {
      await page.addInitScript((key) => {
        if (sessionStorage.getItem('seeded')) return
        sessionStorage.setItem('seeded', '1')
        sessionStorage.setItem(key, 'def=def-id')
      }, SENTINEL)
    }

    test('a started run is recorded, so a kill mid-play leaves a trace', async ({ page }) => {
      await launch(page)
      await capturedPayload(page)
      expect(await sentinel(page)).toBe('def=def-id')
    })

    test('leaving the page deliberately clears the record', async ({ page }) => {
      await launch(page)
      await capturedPayload(page)
      // `pagehide` covers every ordinary exit — navigation, reload, and being
      // frozen into the back-forward cache.
      await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
      expect(await sentinel(page)).toBeNull()
    })

    test('a run that never ended shows the stopped panel and does not restart the game', async ({ page }) => {
      let wasmRequested = false
      page.on('request', (r) => {
        if (r.url().includes('maze_game_bevy_wasm_bg.wasm')) wasmRequested = true
      })
      await seedAbandonedRun(page)
      await launch(page)
      await expect(page.locator('#run-stopped')).toBeVisible()
      await expect(page.locator('#run-stopped h1')).toHaveText('GAME STOPPED')
      // The definition is never fetched, let alone started.
      expect(
        await page.evaluate(
          () => (window as unknown as { __lastStartConfigPayload?: string }).__lastStartConfigPayload
        )
      ).toBeUndefined()
      // Nor is the WASM binary — the slowest thing this page does, and the
      // reason the check cannot wait for the module to run.
      expect(wasmRequested).toBe(false)
    })

    // Navigating away fires `pagehide`, which puts the end panel up before the
    // document is frozen — so returning through the back-forward cache would
    // restore a page wearing both panels.
    test('leaving a stopped page does not stack an end panel on top of it', async ({ page }) => {
      await seedAbandonedRun(page)
      await launch(page)
      await expect(page.locator('#run-stopped')).toBeVisible()
      await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
      await expect(page.locator('#game-ended')).toHaveCount(0)
      await expect(page.locator('#run-stopped')).toBeVisible()
    })

    test('Try Again starts the game rather than reporting the same stop twice', async ({ page }) => {
      await seedAbandonedRun(page)
      await launch(page)
      await page.locator('#run-stopped button').click()
      await capturedPayload(page)
      await expect(page.locator('#run-stopped')).toHaveCount(0)
    })
  })
})

test.describe('Game host stored game-definition score submission (?def=)', () => {
  // A ?def run records against the server-computed challengeKey that the loader
  // stashes on window during the play-fetch — so the module must actually run
  // here (working wasm-JS stub), not be aborted.
  async function setup(page: Page, opts: { challengeKey: string; leaderboardTracked: boolean }) {
    const posted: Array<Record<string, unknown>> = []
    await page.route('**/maze_game_bevy_wasm.js**', (route) => {
      route.fulfill({
        contentType: 'application/javascript',
        body: `
          export default async function init() {}
          export function start() {}
          // The host page imports these too; a missing export is an ES module
          // link error, which fails the whole module rather than one call.
          export function stop() {}
          export function live_bytes() { return 0 }
          export function start_with_config(json) {
            window.__lastStartConfigPayload = json;
          }
        `,
      })
    })
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    await page.route('**/api/v1/game-definitions/def-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          id: 'def-id',
          ownerId: '00000000-0000-0000-0000-0000000000aa',
          name: 'Tower',
          description: null,
          imageUpdatedAt: null,
          visibility: 'public',
          seed: 7,
          rotation: 'static',
          config: { timerSeconds: 90, mode: 'Tower' },
          createdAt: '2025-04-01T12:00:00Z',
          updatedAt: '2025-04-01T12:00:00Z',
          challengeKey: opts.challengeKey,
          leaderboardTracked: opts.leaderboardTracked,
        }),
      })
    })
    await page.route('**/api/v1/scores*', (route) => {
      const req = route.request()
      if (req.method() === 'POST') {
        posted.push(JSON.parse(req.postData() ?? '{}'))
        route.fulfill({
          status: 201,
          contentType: 'application/json',
          body: JSON.stringify({
            id: '00000000-0000-0000-0000-000000000001',
            user_id: '00000000-0000-0000-0000-000000000002',
            score: 0,
            elapsed_ms: 0,
            recorded_at: '2025-04-01T12:00:00Z',
          }),
        })
      } else {
        // Board read for the win-banner thresholds (applyRecordThresholds).
        route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ scores: [] }) })
      }
    })
    await page.goto('/game/index.html?t=fake&def=def-id')
    await expect(page.locator('#pause-menu')).toBeAttached()
    return posted
  }

  async function fireResult(page: Page, detail: Record<string, unknown>) {
    await page.evaluate((d) => {
      window.dispatchEvent(new CustomEvent('maze-game-result', { detail: d }))
    }, detail)
  }

  test('submits the definition challengeKey on a published-definition win', async ({ page }) => {
    const posted = await setup(page, { challengeKey: 'def:def-id', leaderboardTracked: true })
    // The loader stashes the challenge subject once the play-fetch resolves.
    await page.waitForFunction(
      () => (window as unknown as { __mazeDefChallenge?: string }).__mazeDefChallenge === 'def:def-id'
    )
    await fireResult(page, { outcome: 'win', score: 5, elapsedMs: 12345, rows: 3, cols: 3 })
    await expect.poll(() => posted.length).toBe(1)
    expect(posted[0]).toEqual({ challenge: 'def:def-id', score: 5, elapsed_ms: 12345 })
  })

  test('does not submit for an unpublished-definition preview win', async ({ page }) => {
    const posted = await setup(page, { challengeKey: 'def:def-id', leaderboardTracked: false })
    // The loader ran (payload captured) but stashed no challenge — so no record.
    await page.waitForFunction(
      () => typeof (window as unknown as { __lastStartConfigPayload?: string }).__lastStartConfigPayload === 'string'
    )
    await fireResult(page, { outcome: 'win', score: 5, elapsedMs: 12345, rows: 3, cols: 3 })
    await page.waitForTimeout(200)
    expect(posted).toHaveLength(0)
  })
})

test.describe('Game host failure reporting', () => {
  // Drives the failure-capture <script> in index.html — the handlers that turn a
  // Rust panic, an uncaught error, or a rejected module load into a message on
  // the host bridge plus an on-screen reason.
  //
  // Most tests abort the module's own .js so the module script never executes:
  // its top-level await would otherwise reject, and that rejection is itself a
  // failure which would consume the one-shot report before the test could
  // trigger its own. The one test that wants the real rejection path lets the
  // .js load and aborts only the .wasm.

  type BridgeWindow = Window & {
    __hostMessages?: string[]
    chrome?: { webview?: { postMessage: (json: string) => void } }
  }

  type FailurePayload = {
    kind: string
    reason: string
    detail: string
    phase: string
  }

  // Stands in for the native host: the page posts to whichever platform channel
  // it finds, and WebView2's is the easiest to fake in Chromium.
  async function installHostBridge(page: Page) {
    await page.addInitScript(() => {
      const w = window as BridgeWindow
      w.__hostMessages = []
      w.chrome = { ...(w.chrome ?? {}), webview: { postMessage: (json: string) => { w.__hostMessages?.push(json) } } }
    })
  }

  async function hostMessages(page: Page): Promise<string[]> {
    return page.evaluate(() => (window as BridgeWindow).__hostMessages ?? [])
  }

  async function loadWithModuleStubbed(page: Page, url = '/game/index.html?t=fake&id=test-id') {
    await installHostBridge(page)
    await page.route('**/maze_game_bevy_wasm.js**', (r) => r.abort())
    await page.goto(url)
    await expect(page.locator('#pause-menu')).toBeAttached()
  }

  // Lets the module script really run: its listeners and window handles are
  // registered there, so a test that aborts the module registers none of them.
  async function loadWithModuleRunning(page: Page, url = '/game/index.html?t=fake&id=test-id') {
    await page.route('**/maze_game_bevy_wasm.js**', (route) => {
      route.fulfill({
        contentType: 'application/javascript',
        body: `
          export default async function init() {}
          export function start() {}
          export function start_with_config() {}
          export function stop() {}
          export function live_bytes() { return 0 }
        `,
      })
    })
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    await page.route('**/api/v1/mazes/test-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({ id: 'test-id', name: 'M', definition: { grid: [['S', ' ', 'F']] } }),
      })
    })
    await page.goto(url)
    await page.waitForFunction(
      () => typeof (window as unknown as { __mazeStop?: unknown }).__mazeStop === 'function'
    )
  }

  async function firePanic(page: Page, message: string, location: string) {
    await page.evaluate(
      (d) => { window.dispatchEvent(new CustomEvent('maze-game-panic', { detail: d })) },
      { message, location }
    )
  }

  async function fireError(page: Page, message: string) {
    await page.evaluate((m) => { window.dispatchEvent(new ErrorEvent('error', { message: m })) }, message)
  }

  test('reports a failed WASM load as a load-phase failure', async ({ page }) => {
    // The real rejection path: the module runs and its top-level await on the
    // WASM fetch rejects.
    await installHostBridge(page)
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (r) => r.abort())
    await page.goto('/game/index.html?t=fake&id=test-id')

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const failure = JSON.parse((await hostMessages(page))[0]) as FailurePayload
    expect(failure.kind).toBe('failure')
    expect(failure.phase).toBe('load')
  })

  test('reports a Rust panic with its message and location', async ({ page }) => {
    await loadWithModuleStubbed(page)
    await firePanic(page, 'maze JSON did not parse', 'src/world/mod.rs:1243:5')

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const failure = JSON.parse((await hostMessages(page))[0]) as FailurePayload
    expect(failure.kind).toBe('failure')
    expect(failure.detail).toContain('maze JSON did not parse')
    expect(failure.detail).toContain('src/world/mod.rs:1243:5')
    expect(failure.reason).toBe('The game stopped unexpectedly.')
  })

  test('classifies an out-of-memory error distinctly from an ordinary one', async ({ page }) => {
    await loadWithModuleStubbed(page)
    await fireError(page, 'RuntimeError: Out of memory')

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const failure = JSON.parse((await hostMessages(page))[0]) as FailurePayload
    expect(failure.reason).toContain('ran out of memory')
    // The player is told what to do about it, not just that it happened.
    expect(failure.reason).toContain('fewer levels')
  })

  test('classifies a failed memory growth as out of memory', async ({ page }) => {
    // What a WebAssembly.Memory growth failure actually surfaces as.
    await loadWithModuleStubbed(page)
    await fireError(page, 'RangeError: WebAssembly.Memory.grow(): Unable to grow instance memory')

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const failure = JSON.parse((await hostMessages(page))[0]) as FailurePayload
    expect(failure.reason).toContain('ran out of memory')
  })

  test('reports only the first failure', async ({ page }) => {
    // A panic leaves Bevy's animation-frame loop running over a poisoned app,
    // which can throw again every frame — the first cause must not be buried.
    await loadWithModuleStubbed(page)
    await firePanic(page, 'first cause', 'src/lib.rs:1:1')
    await fireError(page, 'follow-on error')
    await fireError(page, 'another follow-on error')
    await page.waitForTimeout(200)

    const messages = await hostMessages(page)
    expect(messages).toHaveLength(1)
    expect((JSON.parse(messages[0]) as FailurePayload).detail).toContain('first cause')
  })

  test('shows the reason in the loading panel when the game never started', async ({ page }) => {
    await loadWithModuleStubbed(page)
    await fireError(page, 'RuntimeError: Out of memory')

    await expect(page.locator('#loading p')).toContainText('ran out of memory')
    // No standalone overlay is needed while the loading panel is still there.
    await expect(page.locator('#fatal-error')).toHaveCount(0)
  })

  test('shows a standalone overlay and reports the play phase after the game started', async ({ page }) => {
    await loadWithModuleStubbed(page)
    // Starting the game removes the loading panel, so a crash from then on has
    // nothing to write into — the case the standalone overlay exists for.
    await page.evaluate(() => { document.getElementById('loading')?.remove() })
    await fireError(page, 'RuntimeError: Out of memory')

    await expect(page.locator('#fatal-error')).toBeVisible()
    await expect(page.locator('#fatal-error')).toContainText('ran out of memory')
    const failure = JSON.parse((await hostMessages(page))[0]) as FailurePayload
    expect(failure.phase).toBe('play')
  })

  test('exposes the teardown handle even without the diagnostics flag', async ({ page }) => {
    // __mazeStop is a production teardown API, not a debug aid — a native host
    // calls it before destroying the document, on launches that never ask for
    // diagnostics. __mazeLiveBytes stays debug-only. Needs the module to really
    // run, so both wasm routes are fulfilled rather than aborted.
    await page.route('**/maze_game_bevy_wasm.js**', (route) => {
      route.fulfill({
        contentType: 'application/javascript',
        body: `
          export default async function init() {}
          export function start() {}
          export function start_with_config() {}
          export function stop() {}
          export function live_bytes() { return 0 }
        `,
      })
    })
    await page.route('**/maze_game_bevy_wasm_bg.wasm**', (route) => {
      route.fulfill({ status: 200, contentType: 'application/wasm', body: '' })
    })
    await page.route('**/api/v1/mazes/test-id*', (route) => {
      route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({ id: 'test-id', name: 'M', definition: { grid: [['S', ' ', 'F']] } }),
      })
    })

    await page.goto('/game/index.html?t=fake&id=test-id')
    await page.waitForFunction(() => typeof (window as unknown as { __mazeStop?: unknown }).__mazeStop === 'function')
    expect(
      await page.evaluate(() => typeof (window as unknown as { __mazeLiveBytes?: unknown }).__mazeLiveBytes)
    ).toBe('undefined')
  })

  test('forwards the game teardown confirmation to the host', async ({ page }) => {
    // The handshake that lets a host wait for the release rather than guess at
    // a delay — a guess that came up short would silently defeat the release.
    await loadWithModuleStubbed(page)
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('maze-game-stopped'))
    })

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const stopped = JSON.parse((await hostMessages(page))[0]) as { kind: string }
    expect(stopped.kind).toBe('stopped')
  })

  test('replaces the game with an end panel when the page is hidden', async ({ page }) => {
    // Navigating away tears the game down, so the canvas is left on a stale
    // frame. The panel goes up synchronously with the teardown so a document
    // frozen into the back-forward cache carries it, and a restored page shows
    // an ended game rather than a hung one.
    await loadWithModuleRunning(page)
    await expect(page.locator('#game-ended')).toHaveCount(0)

    await page.evaluate(() => { window.dispatchEvent(new PageTransitionEvent('pagehide')) })

    await expect(page.locator('#game-ended')).toBeVisible()
    await expect(page.locator('#game-ended h1')).toHaveText('GAME ENDED')
    await expect(page.locator('body.game-ended')).toHaveCount(1)
    // The stale canvas and the controls for a game that no longer exists go.
    await expect(page.locator('canvas')).toBeHidden()
  })

  test('the end panel hides a pause menu left open behind it', async ({ page }) => {
    // Pausing and then navigating away left Resume and Restart visible behind
    // the panel on return — both driving a game that no longer exists.
    await loadWithModuleRunning(page)
    await firePaused(page, true)
    await expect(page.locator('#pause-menu')).toBeVisible()

    await page.evaluate(() => { window.dispatchEvent(new PageTransitionEvent('pagehide')) })

    await expect(page.locator('#game-ended')).toBeVisible()
    await expect(page.locator('#pause-menu')).toBeHidden()
  })

  test('the end panel offers a working replay', async ({ page }) => {
    await loadWithModuleRunning(page)
    await page.evaluate(() => { window.dispatchEvent(new PageTransitionEvent('pagehide')) })

    const reloaded = page.waitForEvent('load')
    await page.locator('#game-ended button').click()
    await reloaded
    // A fresh document — the panel is gone until this game ends in its turn.
    await expect(page.locator('#game-ended')).toHaveCount(0)
  })

  test('tags a forwarded result with its envelope kind', async ({ page }) => {
    await loadWithModuleStubbed(page)
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('maze-game-result', {
        detail: { outcome: 'win', score: 7, elapsedMs: 42137, rows: 3, cols: 3 },
      }))
    })

    await expect.poll(async () => (await hostMessages(page)).length).toBe(1)
    const result = JSON.parse((await hostMessages(page))[0]) as { kind: string; outcome: string; score: number }
    expect(result.kind).toBe('result')
    expect(result.outcome).toBe('win')
    expect(result.score).toBe(7)
  })
})
