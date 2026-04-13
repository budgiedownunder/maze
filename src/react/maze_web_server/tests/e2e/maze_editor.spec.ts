import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Username').fill('testuser')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/mazes/)
}

async function openFirstMaze(page: Page) {
  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()
  await items.first().locator('.maze-item-text').click()
  await expect(page).toHaveURL(/\/mazes\/[^/]+$/)
  await expect(page.locator('.maze-grid-container')).toBeVisible()
}

test('clicking a maze from the list navigates to the editor and loads the grid', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  const firstName = await items.first().locator('.maze-item-name').textContent()

  // Click the item row (not an action button)
  await items.first().locator('.maze-item-text').click()

  // Should navigate to /mazes/:id
  await expect(page).toHaveURL(/\/mazes\/[^/]+$/)

  // Maze name should appear in the header
  await expect(page.locator('.app-header-title')).toHaveText(firstName!)

  // Grid container should be visible
  await expect(page.locator('.maze-grid-container')).toBeVisible()
})

test('maze grid contains expected cell images', async ({ page }) => {
  await login(page)

  const items = page.locator('.maze-list-item')
  await expect(items.first()).toBeVisible()

  await items.first().locator('.maze-item-text').click()
  await expect(page).toHaveURL(/\/mazes\/[^/]+$/)

  // Wait for the grid to appear
  await expect(page.locator('.maze-grid-container')).toBeVisible()

  // At least a Start image should be present (all mock mazes have S)
  await expect(page.locator('img[alt="Start"]')).toBeVisible()
  // And a Finish image
  await expect(page.locator('img[alt="Finish"]')).toBeVisible()
})

// ──────────────────────────────────────────────────────────────
// Cell selection + toolbar
// ──────────────────────────────────────────────────────────────

test('toolbar is hidden before selecting a cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByLabel('Maze editor toolbar')).not.toBeVisible()
})

test('clicking a cell shows the toolbar', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByLabel('Maze editor toolbar')).toBeVisible()
})

test('Set Wall button is enabled on an empty cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (0,1) in Alpha maze is empty
  await page.getByLabel('Cell 1,2').click()

  const btn = page.getByRole('button', { name: 'Set Wall' })
  await expect(btn).toBeVisible()
  await expect(btn).toBeEnabled()
})

test('Set Wall button is disabled when selected cell is already a wall', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (1,1) in Alpha maze is 'W'
  await page.getByLabel('Cell 2,2').click()

  await expect(page.getByRole('button', { name: 'Set Wall' })).toBeDisabled()
})

test('Set Start button is disabled when selected cell already contains S', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (0,0) in Alpha maze is 'S'
  await page.getByLabel('Cell 1,1').click()

  await expect(page.getByRole('button', { name: 'Set Start' })).toBeDisabled()
})

test('Set Finish button is disabled when selected cell already contains F', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (2,2) in Alpha maze is 'F'
  await page.getByLabel('Cell 3,3').click()

  await expect(page.getByRole('button', { name: 'Set Finish' })).toBeDisabled()
})

test('Clear button is disabled on an empty cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (0,1) in Alpha maze is empty
  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Clear' })).toBeDisabled()
})

test('Clear button is enabled on a wall cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (1,1) in Alpha maze is 'W'
  await page.getByLabel('Cell 2,2').click()

  await expect(page.getByRole('button', { name: 'Clear' })).toBeEnabled()
})

test('clicking Set Wall places a wall image in the cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Click empty cell (0,1)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()

  // Wall image should now appear in that cell
  const cell = page.getByLabel('Cell 1,2')
  await expect(cell.getByAltText('Wall')).toBeVisible()
})

test('clicking Set Start moves the start flag to the new cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Click empty cell (0,2) and set as start
  await page.getByLabel('Cell 1,3').click()
  await page.getByRole('button', { name: 'Set Start' }).click()

  // New cell should have start flag
  await expect(page.getByLabel('Cell 1,3').getByAltText('Start')).toBeVisible()
  // Old start cell (0,0) should no longer have start flag
  await expect(page.getByLabel('Cell 1,1').getByAltText('Start')).not.toBeVisible()
})

test('clicking Clear removes a wall', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (1,1) is 'W' — select it and clear
  await page.getByLabel('Cell 2,2').click()
  await page.getByRole('button', { name: 'Clear' }).click()

  // Wall image should be gone
  await expect(page.getByLabel('Cell 2,2').getByAltText('Wall')).not.toBeVisible()
})

// ──────────────────────────────────────────────────────────────
// Keyboard navigation
// ──────────────────────────────────────────────────────────────

test('arrow keys move the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select cell (0,0) by clicking then navigate right
  await page.getByLabel('Cell 1,1').click()
  await page.getByLabel('Maze grid').press('ArrowRight')

  // Cell (0,1) should now be selected (yellow background — single selection uses anchor class)
  const cell = page.getByLabel('Cell 1,2')
  await expect(cell).toHaveClass(/maze-cell--anchor/)
})

test('W shortcut sets a wall on the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select empty cell (0,1)
  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('w')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Wall')).toBeVisible()
})

test('Delete shortcut clears a wall', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select wall cell (1,1)
  await page.getByLabel('Cell 2,2').click()
  await page.getByLabel('Maze grid').press('Delete')

  await expect(page.getByLabel('Cell 2,2').getByAltText('Wall')).not.toBeVisible()
})

test('S shortcut sets the start on an empty cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select empty cell (0,2) and press S
  await page.getByLabel('Cell 1,3').click()
  await page.getByLabel('Maze grid').press('s')

  await expect(page.getByLabel('Cell 1,3').getByAltText('Start')).toBeVisible()
  // Old start should be cleared
  await expect(page.getByLabel('Cell 1,1').getByAltText('Start')).not.toBeVisible()
})

test('F shortcut sets the finish on an empty cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select empty cell (0,1) and press F
  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('f')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Finish')).toBeVisible()
  // Old finish should be cleared
  await expect(page.getByLabel('Cell 3,3').getByAltText('Finish')).not.toBeVisible()
})

test('Home key moves active cell to start of row', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select cell (1,2) then press Home
  await page.getByLabel('Cell 2,3').click()
  await page.getByLabel('Maze grid').press('Home')

  await expect(page.getByLabel('Cell 2,1')).toHaveClass(/maze-cell--anchor/)
})

test('End key moves active cell to end of row', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; select (0,0) then press End
  await page.getByLabel('Cell 1,1').click()
  await page.getByLabel('Maze grid').press('End')

  await expect(page.getByLabel('Cell 1,3')).toHaveClass(/maze-cell--anchor/)
})

// ──────────────────────────────────────────────────────────────
// Scroll-to-active-cell
// ──────────────────────────────────────────────────────────────

test('navigating off-screen scrolls the grid to reveal the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Constrain the container so only part of the 3×3 grid is visible.
  // HEADER_SIZE=24, CELL_SIZE=32 → a 60px container shows the header + ~1 column / ~1 row.
  // Use max-width/max-height so flexbox doesn't override the explicit size constraint.
  await page.evaluate(() => {
    const el = document.querySelector('.maze-grid-container') as HTMLElement
    el.style.maxWidth  = '60px'
    el.style.maxHeight = '60px'
  })

  // Cell 1,1 is still within the visible area at this size.
  await page.getByLabel('Cell 1,1').click()

  // Navigate right — cell 1,2 is now off-screen horizontally.
  await page.getByLabel('Maze grid').press('ArrowRight')
  const scrollLeft = await page.evaluate(
    () => (document.querySelector('.maze-grid-container') as HTMLElement).scrollLeft,
  )
  expect(scrollLeft).toBeGreaterThan(0)

  // Navigate down — cell 2,2 is now off-screen vertically.
  await page.getByLabel('Maze grid').press('ArrowDown')
  const scrollTop = await page.evaluate(
    () => (document.querySelector('.maze-grid-container') as HTMLElement).scrollTop,
  )
  expect(scrollTop).toBeGreaterThan(0)
})
