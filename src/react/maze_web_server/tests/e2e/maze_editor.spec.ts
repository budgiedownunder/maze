import { test, expect, type Page, devices } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
  // Tests in this file open mazes from the list — navigate there.
  await page.goto('/mazes')
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

test('toolbar is visible with editing buttons disabled before selecting a cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByLabel('Maze editor toolbar')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Set Wall' })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Generate' })).toBeEnabled()
})

test('clicking a cell enables the toolbar buttons', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Set Wall' })).toBeEnabled()
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

  await expect(page.getByRole('button', { name: 'Clear', exact: true })).toBeDisabled()
})

test('Clear button is enabled on a wall cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (1,1) in Alpha maze is 'W'
  await page.getByLabel('Cell 2,2').click()

  await expect(page.getByRole('button', { name: 'Clear', exact: true })).toBeEnabled()
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
  await page.getByRole('button', { name: 'Clear', exact: true }).click()

  // Wall image should be gone
  await expect(page.getByLabel('Cell 2,2').getByAltText('Wall')).not.toBeVisible()
})

test('clicking Clear removes a key (Clear is enabled on a key cell)', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Place a key on empty cell (0,1), then clear it
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Key' }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Key')).toBeVisible()

  // Clear must be enabled on a key cell, and clicking it removes the key
  await expect(page.getByRole('button', { name: 'Clear', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Clear', exact: true }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Key')).not.toBeVisible()
})

test('Set Key button is enabled on a selected cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Cell (0,1) in Alpha maze is empty
  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Set Key' })).toBeEnabled()
})

test('Set Door button is enabled on a selected cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Set Door' })).toBeEnabled()
})

test('clicking Set Key places a key image in the cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Key' }).click()

  await expect(page.getByLabel('Cell 1,2').getByAltText('Key')).toBeVisible()
})

test('clicking Set Door places a door image in the cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Door' }).click()

  await expect(page.getByLabel('Cell 1,2').getByAltText('Door')).toBeVisible()
})

test('Set Enemy button is enabled on a selected cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Set Enemy' })).toBeEnabled()
})

test('Set Health button is enabled on a selected cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()

  await expect(page.getByRole('button', { name: 'Set Health' })).toBeEnabled()
})

test('clicking Set Enemy places an enemy image in the cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Enemy' }).click()

  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toBeVisible()
})

test('clicking Set Health places a health image in the cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Health' }).click()

  await expect(page.getByLabel('Cell 1,2').getByAltText('Health')).toBeVisible()
})

test('Clear removes an enemy (Clear is enabled on an enemy cell)', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Enemy' }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toBeVisible()

  await expect(page.getByRole('button', { name: 'Clear', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Clear', exact: true }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).not.toBeVisible()
})

test('Clear removes a health pickup (Clear is enabled on a health cell)', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Health' }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Health')).toBeVisible()

  await expect(page.getByRole('button', { name: 'Clear', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Clear', exact: true }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Health')).not.toBeVisible()
})

test('placing E and H, saving, and refreshing persists both cells', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Enemy' }).click()
  await page.getByLabel('Cell 1,3').click()
  await page.getByRole('button', { name: 'Set Health' }).click()

  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()

  // Dirty with a throwaway wall so Refresh enables; then Refresh re-fetches
  // from the server to prove the previous save round-tripped.
  await page.getByLabel('Cell 3,1').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.getByRole('button', { name: 'Refresh' }).click()
  await page.getByRole('button', { name: 'Reload' }).click()

  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toBeVisible()
  await expect(page.getByLabel('Cell 1,3').getByAltText('Health')).toBeVisible()
  await expect(page.getByLabel('Cell 3,1').getByAltText('Wall')).not.toBeVisible()
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

test('K shortcut sets a key on the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select empty cell (0,1) and press K
  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('k')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Key')).toBeVisible()
})

test('D shortcut sets a door on the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Select empty cell (0,1) and press D
  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('d')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Door')).toBeVisible()
})

test('E shortcut sets an enemy on the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('e')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toBeVisible()
})

test('H shortcut sets a health pickup on the active cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Maze grid').press('h')

  await expect(page.getByLabel('Cell 1,2').getByAltText('Health')).toBeVisible()
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

// ──────────────────────────────────────────────────────────────
// Structural editing — Insert / Delete rows and columns
// ──────────────────────────────────────────────────────────────

test('clicking a row header enables Insert Rows Before and Delete; disables Insert Columns Before', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Row 1').click()

  await expect(page.getByRole('button', { name: 'Insert Rows Before' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Delete' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Insert Columns Before' })).toBeDisabled()
})

test('clicking a column header enables Insert Columns Before and Delete; disables Insert Rows Before', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Column 1').click()

  await expect(page.getByRole('button', { name: 'Insert Columns Before' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Delete' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Insert Rows Before' })).toBeDisabled()
})

test('clicking the corner header enables Insert Row/Column; disables Delete', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Select all').click()

  await expect(page.getByRole('button', { name: 'Insert Rows Before' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Insert Columns Before' })).not.toBeDisabled()
  await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled()
})

test('Insert Rows Before adds a row to the grid', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; click row 1 header and insert
  await page.getByLabel('Row 1').click()
  await page.getByRole('button', { name: 'Insert Rows Before' }).click()

  // Grid should now have 4 rows
  await expect(page.getByLabel('Row 4')).toBeVisible()
})

test('Delete removes a row when a row header is selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; click row 1 header and delete
  await page.getByLabel('Row 1').click()
  await page.getByRole('button', { name: 'Delete' }).click()

  // Grid should now have 2 rows
  await expect(page.getByLabel('Row 2')).toBeVisible()
  await expect(page.getByLabel('Row 3')).not.toBeVisible()
})

test('Insert Rows Before inserts N rows when N rows are selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; select rows 1–2 (shift-click row headers)
  await page.getByLabel('Row 1').click()
  await page.getByLabel('Row 2').click({ modifiers: ['Shift'] })
  await page.getByRole('button', { name: 'Insert Rows Before' }).click()

  // 2 rows inserted → 5 rows total
  await expect(page.getByLabel('Row 5')).toBeVisible()
})

test('Insert Columns Before adds a column to the grid', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; click column 1 header and insert
  await page.getByLabel('Column 1').click()
  await page.getByRole('button', { name: 'Insert Columns Before' }).click()

  // Grid should now have 4 columns
  await expect(page.getByLabel('Column 4')).toBeVisible()
})

test('Insert Columns Before inserts N columns when N columns are selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; select cols 1–2 (shift-click column headers)
  await page.getByLabel('Column 1').click()
  await page.getByLabel('Column 2').click({ modifiers: ['Shift'] })
  await page.getByRole('button', { name: 'Insert Columns Before' }).click()

  // 2 cols inserted → 5 cols total
  await expect(page.getByLabel('Column 5')).toBeVisible()
})

test('Delete removes a column when a column header is selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Alpha is 3×3; click column 1 header and delete
  await page.getByLabel('Column 1').click()
  await page.getByRole('button', { name: 'Delete' }).click()

  // Grid should now have 2 columns
  await expect(page.getByLabel('Column 2')).toBeVisible()
  await expect(page.getByLabel('Column 3')).not.toBeVisible()
})

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

// ──────────────────────────────────────────────────────────────
// Save, Refresh, and New maze
// ──────────────────────────────────────────────────────────────

test('New maze button on mazes list navigates to blank editor', async ({ page }) => {
  await login(page)
  await page.getByRole('button', { name: 'New maze' }).click()
  await expect(page).toHaveURL(/\/mazes\/new$/)
  await expect(page.locator('.app-header-title')).toHaveText('(unsaved)')
  await expect(page.locator('.maze-grid-container')).toBeVisible()
})

test('Save button is visible and disabled on an unedited existing maze', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
})

test('Save button becomes enabled after editing a cell', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await expect(page.getByRole('button', { name: 'Save' })).not.toBeDisabled()
})

test('saving an existing maze calls the API and re-disables Save', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.getByRole('button', { name: 'Save' }).click()
  // After successful save, Save button should be disabled again
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
})

test('saving a new maze opens name prompt and navigates to /mazes/:id', async ({ page }) => {
  await login(page)
  await page.goto('/mazes/new')
  await expect(page.locator('.maze-grid-container')).toBeVisible()
  // Edit a cell to make the maze dirty before saving
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('dialog', { name: 'Save Maze' })).toBeVisible()
  await page.getByRole('textbox').fill('Brand New Maze')
  await page.getByRole('dialog', { name: 'Save Maze' }).getByRole('button', { name: 'Save' }).click()
  // Should navigate away from /mazes/new to /mazes/:id and show the name
  await expect(page).not.toHaveURL(/\/mazes\/new$/)
  await expect(page).toHaveURL(/\/mazes\/.+$/)
  await expect(page.locator('.app-header-title')).toHaveText('Brand New Maze')
})

// ──────────────────────────────────────────────────────────────
// Double-tap range mode (touch)
// ──────────────────────────────────────────────────────────────

test.describe('double-tap range mode', () => {
  // Spread device settings but omit defaultBrowserType — Playwright disallows it inside a describe group.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- the rename is the omission
  const { defaultBrowserType: _ignored, ...pixel7 } = devices['Pixel 7']
  test.use(pixel7)

  test('double-clicking a cell enters range mode', async ({ page }) => {
    await login(page)
    await openFirstMaze(page)
    // dblclick fires onDoubleClick → onCellDoubleClick → handleCellDoubleClick → enableRangeMode()
    await page.getByLabel('Cell 2,2').dblclick()
    await expect(page.getByRole('button', { name: 'Done' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Select range' })).not.toBeVisible()
  })

  test('double-clicking again exits range mode', async ({ page }) => {
    await login(page)
    await openFirstMaze(page)
    // Enter range mode via first double-click
    await page.getByLabel('Cell 2,2').dblclick()
    await expect(page.getByRole('button', { name: 'Done' })).toBeVisible()
    // Second double-click exits range mode
    await page.getByLabel('Cell 2,2').dblclick()
    await expect(page.getByRole('button', { name: 'Select range' })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Done' })).not.toBeVisible()
  })
})

test('Refresh button is not shown on new maze', async ({ page }) => {
  await login(page)
  await page.goto('/mazes/new')
  await expect(page.locator('.maze-grid-container')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Refresh' })).not.toBeVisible()
})

test('Refresh button is disabled on an unedited existing maze', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Refresh' })).toBeDisabled()
})

test('clicking Refresh on a dirty maze shows confirm dialog', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.getByRole('button', { name: 'Refresh' }).click()
  await expect(page.getByRole('dialog', { name: 'Discard changes?' })).toBeVisible()
})

test('confirming Refresh reloads the maze and clears dirty state', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.getByRole('button', { name: 'Refresh' }).click()
  await page.getByRole('button', { name: 'Reload' }).click()
  // After reload, Save and Refresh should both be disabled again
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Refresh' })).toBeDisabled()
})

test('navigate away while dirty shows unsaved changes dialog', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.evaluate(() => window.history.back())
  await expect(page.getByRole('dialog', { name: 'Unsaved Changes' })).toBeVisible()
})

test('navigate away from unedited new maze shows unsaved changes dialog', async ({ page }) => {
  await login(page)
  // Navigate through the app so history.back() returns to /mazes
  await page.getByRole('button', { name: 'New maze' }).click()
  await expect(page).toHaveURL(/\/mazes\/new$/)
  await expect(page.locator('.maze-grid-container')).toBeVisible()
  await page.evaluate(() => window.history.back())
  await expect(page.getByRole('dialog', { name: 'Unsaved Changes' })).toBeVisible()
})

test('discarding changes allows navigation away', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.evaluate(() => window.history.back())
  await page.getByRole('button', { name: 'Discard' }).click()
  await expect(page).not.toHaveURL(/\/mazes\/[^/]+$/)
})

test('saving from the navigate-away dialog on an existing maze clears dirty and proceeds', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await page.evaluate(() => window.history.back())
  await expect(page.getByRole('dialog', { name: 'Unsaved Changes' })).toBeVisible()
  await page.getByRole('dialog', { name: 'Unsaved Changes' }).getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('dialog', { name: 'Unsaved Changes' })).not.toBeVisible()
  await expect(page).not.toHaveURL(/\/mazes\/[^/]+$/)
})

test('saving from the navigate-away dialog on a new maze prompts for a name then proceeds', async ({ page }) => {
  await login(page)
  // Navigate through the app so history.back() returns to /mazes
  await page.getByRole('button', { name: 'New maze' }).click()
  await expect(page).toHaveURL(/\/mazes\/new$/)
  await expect(page.locator('.maze-grid-container')).toBeVisible()
  await page.evaluate(() => window.history.back())
  await expect(page.getByRole('dialog', { name: 'Unsaved Changes' })).toBeVisible()
  await page.getByRole('dialog', { name: 'Unsaved Changes' }).getByRole('button', { name: 'Save' }).click()
  // Should show name prompt
  await expect(page.getByRole('dialog', { name: 'Save Maze' })).toBeVisible()
  await page.getByRole('textbox').fill('Blocker Test Maze')
  await page.getByRole('dialog', { name: 'Save Maze' }).getByRole('button', { name: 'Save' }).click()
  // Should navigate away after save
  await expect(page.getByRole('dialog', { name: 'Save Maze' })).not.toBeVisible()
  await expect(page).not.toHaveURL(/\/mazes\/new$/)
})

// ──────────────────────────────────────────────────────────────
// Generate
// ──────────────────────────────────────────────────────────────

test('Generate button is always enabled regardless of selection', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Generate' })).toBeEnabled()
  await page.getByLabel('Cell 1,1').click()
  await expect(page.getByRole('button', { name: 'Generate' })).toBeEnabled()
})

test('clicking Generate opens the Generate Maze dialog', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Generate' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' })).toBeVisible()
})

test('Generate Maze dialog has all expected fields', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  // Size & Position tab (default): dimensions + start/finish.
  await expect(dialog.getByLabel('Rows')).toBeVisible()
  await expect(dialog.getByLabel('Columns')).toBeVisible()
  await expect(dialog.getByLabel('Start Row')).toBeVisible()
  await expect(dialog.getByLabel('Start Column')).toBeVisible()
  await expect(dialog.getByLabel('Finish Row')).toBeVisible()
  await expect(dialog.getByLabel('Finish Column')).toBeVisible()
  await expect(dialog.getByLabel('Min Solution Length')).toBeVisible()
  // Features tab: doors / keys / enemies / health counts.
  await dialog.getByRole('tab', { name: 'Features' }).click()
  await expect(dialog.getByLabel('Doors', { exact: true })).toBeVisible()
  await expect(dialog.getByLabel('Spare Doors')).toBeVisible()
  await expect(dialog.getByLabel('Spare Keys')).toBeVisible()
  await expect(dialog.getByLabel('Enemies')).toBeVisible()
  await expect(dialog.getByLabel('Health')).toBeVisible()
})

test('Generate Maze dialog defaults reflect the current grid dimensions', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)  // Alpha is 3×3
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  await expect(dialog.getByLabel('Rows')).toHaveValue('3')
  await expect(dialog.getByLabel('Columns')).toHaveValue('3')
})

test('generating a maze replaces the grid with the new dimensions', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)  // Alpha is 3×3
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  // Change to 5×5
  await dialog.getByLabel('Rows').fill('5')
  await dialog.getByLabel('Columns').fill('5')
  await dialog.getByLabel('Finish Row').fill('5')
  await dialog.getByLabel('Finish Column').fill('5')
  await dialog.getByRole('button', { name: 'Generate' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' })).not.toBeVisible()
  await expect(page.getByLabel('Row 5')).toBeVisible()
  await expect(page.getByLabel('Column 5')).toBeVisible()
})

test('generating with doors places key and door cells in the grid', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  await dialog.getByLabel('Rows').fill('15')
  await dialog.getByLabel('Columns').fill('15')
  await dialog.getByLabel('Start Row').fill('1')
  await dialog.getByLabel('Start Column').fill('1')
  await dialog.getByLabel('Finish Row').fill('15')
  await dialog.getByLabel('Finish Column').fill('15')
  await dialog.getByLabel('Min Solution Length').fill('8')
  // Doors lives on the Features tab.
  await dialog.getByRole('tab', { name: 'Features' }).click()
  await dialog.getByLabel('Doors', { exact: true }).fill('3')
  await dialog.getByRole('button', { name: 'Generate' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' })).not.toBeVisible()
  // Auto-placed key and door cells render their icons in the editor grid.
  await expect(page.getByAltText('Door').first()).toBeVisible()
  await expect(page.getByAltText('Key').first()).toBeVisible()
})

test('generating with enemies and health places enemy and health cells in the grid', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  await dialog.getByLabel('Rows').fill('15')
  await dialog.getByLabel('Columns').fill('15')
  await dialog.getByLabel('Start Row').fill('1')
  await dialog.getByLabel('Start Column').fill('1')
  await dialog.getByLabel('Finish Row').fill('15')
  await dialog.getByLabel('Finish Column').fill('15')
  await dialog.getByLabel('Min Solution Length').fill('8')
  // Enemies and Health live on the Features tab.
  await dialog.getByRole('tab', { name: 'Features' }).click()
  await dialog.getByLabel('Enemies').fill('2')
  await dialog.getByLabel('Health').fill('2')
  await dialog.getByRole('button', { name: 'Generate' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' })).not.toBeVisible()
  await expect(page.getByAltText('Enemy').first()).toBeVisible()
  await expect(page.getByAltText('Health').first()).toBeVisible()
})

test('cancelling Generate dialog leaves the grid unchanged', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)  // Alpha is 3×3
  await page.getByLabel('Cell 1,1').click()
  await page.getByRole('button', { name: 'Generate' }).click()
  await page.getByRole('dialog', { name: 'Generate Maze' }).getByRole('button', { name: 'Cancel' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' })).not.toBeVisible()
  // Grid should still be 3 rows
  await expect(page.getByLabel('Row 3')).toBeVisible()
  await expect(page.getByLabel('Row 4')).not.toBeVisible()
})

test('Generate dialog remembers last used Min Solution Length within the session', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  // Open Generate dialog and set Min Solution Length to 5
  await page.getByRole('button', { name: 'Generate' }).click()
  const dialog = page.getByRole('dialog', { name: 'Generate Maze' })
  await dialog.getByLabel('Min Solution Length').fill('5')
  await dialog.getByRole('button', { name: 'Generate' }).click()
  await expect(dialog).not.toBeVisible()
  // Reopen — should show 5, not 1
  await page.getByRole('button', { name: 'Generate' }).click()
  await expect(page.getByRole('dialog', { name: 'Generate Maze' }).getByLabel('Min Solution Length')).toHaveValue('5')
})

// ──────────────────────────────────────────────────────────────
// Solve / Clear Solution
// ──────────────────────────────────────────────────────────────

test('Solve button is present and enabled when no solution is shown', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Solve' })).toBeEnabled()
})

test('Clear Solution button is hidden when no solution is shown', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Clear Solution' })).not.toBeVisible()
})

test('clicking Solve on a solvable maze shows the solution overlay', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)  // Alpha has S and F
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).toBeEnabled()
})

test('clicking Clear Solution removes the solution overlay', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await page.getByRole('button', { name: 'Clear Solution' }).click()
  await expect(page.locator('img[alt="Solution path"]')).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).not.toBeVisible()
})

test('clicking Solve on an unsolvable maze shows an alert dialog', async ({ page }) => {
  await login(page)
  await page.goto('/mazes/new')  // blank grid has no S or F
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.getByRole('dialog', { name: 'Unable to solve maze' })).toBeVisible()
  await page.getByRole('button', { name: 'OK' }).click()
  await expect(page.getByRole('dialog', { name: 'Unable to solve maze' })).not.toBeVisible()
})

test('editing buttons and Solve are disabled while the solution is displayed', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Set Wall' })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Generate' })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'Solve' })).toBeDisabled()
})

test('single active cell is retained after solving', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.locator('td[aria-label="Cell 1,1"]').click()
  // Selection frame present with --single modifier before solving
  await expect(page.locator('.maze-selection-frame--single')).toBeVisible()
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  // Selection frame still present after solving — active cell was retained
  await expect(page.locator('.maze-selection-frame')).toBeVisible()
})

test('multi-cell selection is collapsed to the anchor cell after solving', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.locator('td[aria-label="Cell 1,1"]').click()
  await page.locator('td[aria-label="Cell 2,2"]').click({ modifiers: ['Shift'] })
  // Multi-cell range: frame exists but does NOT have --single modifier
  await expect(page.locator('.maze-selection-frame')).toBeVisible()
  await expect(page.locator('.maze-selection-frame--single')).not.toBeVisible()
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  // Range collapsed to single anchor cell — frame now has --single modifier
  await expect(page.locator('.maze-selection-frame--single')).toBeVisible()
})

test('active cell is re-highlighted after clearing the solution', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.locator('td[aria-label="Cell 1,1"]').click()
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await expect(page.locator('.maze-selection-frame')).toBeVisible()
  await page.getByRole('button', { name: 'Clear Solution' }).click()
  await expect(page.locator('img[alt="Solution path"]')).not.toBeVisible()
  // Active cell still selected after clearing
  await expect(page.locator('.maze-selection-frame--single')).toBeVisible()
})

// ── Keyboard shortcut hint bar ────────────────────────────────

test('shortcut hint bar is not visible before a cell is selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.locator('.maze-shortcuts-hint')).not.toBeVisible()
})

test('shortcut hint bar appears after a cell is selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.locator('td[aria-label="Cell 1,1"]').click()
  await expect(page.locator('.maze-shortcuts-hint')).toBeVisible()
})

test('shortcut hint bar contains expected shortcuts', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.locator('td[aria-label="Cell 1,1"]').click()
  const hint = page.locator('.maze-shortcuts-hint')
  await expect(hint).toContainText('[W]')
  await expect(hint).toContainText('[S]')
  await expect(hint).toContainText('[F]')
  await expect(hint).toContainText('[K]')
  await expect(hint).toContainText('[D]')
  await expect(hint).toContainText('[E]')
  await expect(hint).toContainText('[H]')
  await expect(hint).toContainText('[DEL]')
  await expect(hint).toContainText('[Shift] Range')
})

// ──────────────────────────────────────────────────────────────
// Walk Solution
// ──────────────────────────────────────────────────────────────

test('Walk Solution button is visible and enabled when no solution is shown', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await expect(page.getByRole('button', { name: 'Walk Solution' })).toBeEnabled()
})

test('Walk Solution button is disabled when a solution is already displayed', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Solve' }).click()
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Walk Solution' })).toBeDisabled()
})

test('clicking Walk Solution on an unsolvable maze shows an alert dialog', async ({ page }) => {
  await login(page)
  await page.goto('/mazes/new')  // blank grid has no S or F
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  await expect(page.getByRole('dialog', { name: 'Unable to solve maze' })).toBeVisible()
  await page.getByRole('button', { name: 'OK' }).click()
  await expect(page.getByRole('dialog', { name: 'Unable to solve maze' })).not.toBeVisible()
})

test('clicking Walk Solution shows walker image stepping through the maze', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  // Walker GIF should appear during animation
  await expect(page.locator('img[alt="Walker"]')).toBeVisible()
})

test('after Walk Solution completes the full solution is displayed', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  await expect(page.locator('img[alt="Walker"]')).toHaveAttribute('src', /walker_celebrate/, { timeout: 10000 })
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).toBeEnabled()
  await expect(page.getByRole('button', { name: 'Walk Solution' })).toBeDisabled()
})

test('Clear Solution button is enabled while walk is in progress', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  await expect(page.locator('img[alt="Walker"]')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).toBeEnabled()
})

test('clicking Clear Solution mid-walk cancels the animation and resets the grid', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  await expect(page.locator('img[alt="Walker"]')).toBeVisible()
  await page.getByRole('button', { name: 'Clear Solution' }).click()
  await expect(page.locator('img[alt="Walker"]')).not.toBeVisible()
  await expect(page.locator('img[alt="Solution path"]')).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Walk Solution' })).toBeEnabled()
})

test('clicking a cell during walk does not produce a selection frame', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  await expect(page.locator('img[alt="Walker"]')).toBeVisible()
  await page.locator('td[aria-label="Cell 1,3"]').click()
  await expect(page.locator('.maze-selection-frame')).not.toBeVisible()
})

test('Clear Solution after Walk Solution resets to normal editing state', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)
  await page.getByRole('button', { name: 'Walk Solution' }).click()
  // The walk is timer-driven and normally finishes in ~2s; under parallel-worker
  // CPU contention the timers lag, so allow generous headroom for the walker to
  // finish and disappear rather than flaking on a starved animation.
  await expect(page.locator('img[alt="Walker"]')).not.toBeVisible({ timeout: 30000 })
  await expect(page.locator('img[alt="Solution path"]').first()).toBeVisible()
  await page.getByRole('button', { name: 'Clear Solution' }).click()
  await expect(page.locator('img[alt="Solution path"]')).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Clear Solution' })).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Walk Solution' })).toBeEnabled()
  // Generate and Solve don't require a cell selection — confirm editing is re-enabled
  await expect(page.getByRole('button', { name: 'Generate' })).toBeEnabled()
  await expect(page.getByRole('button', { name: 'Solve' })).toBeEnabled()
})

// ──────────────────────────────────────────────────────────────
// Per-cell override authoring
// ──────────────────────────────────────────────────────────────

test('authoring a ghost enemy override shows the ghost sprite plus a badge and saves', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Stamp an enemy on the empty cell (0,1), which opens the override panel.
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Enemy' }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toBeVisible()

  const typeSelect = page.getByRole('combobox', { name: 'Type' })
  await expect(typeSelect).toBeVisible()
  await typeSelect.selectOption('ghost')
  await page.getByRole('spinbutton', { name: 'Damage' }).fill('2')

  // The grid cell now shows the ghost variant sprite and an override badge.
  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toHaveAttribute('src', /ghost\.svg/)
  await expect(page.getByLabel('Cell 1,2').getByLabel('Has override')).toBeVisible()

  // Saving runs the real build-definition codec; the button disables on success and
  // no save error appears.
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
  await expect(page.locator('.error-msg')).not.toBeVisible()
})

test('a maze saved with a per-cell override loads and renders it', async ({ page }) => {
  await login(page)
  // Deep-link to a maze whose stored definition already carries the ghost override.
  await page.goto('/mazes/maze-override')
  await expect(page.locator('.maze-grid-container')).toBeVisible()

  // The persisted override round-trips through load -> split and renders.
  await expect(page.getByLabel('Cell 1,2').getByAltText('Enemy')).toHaveAttribute('src', /ghost\.svg/)
  await expect(page.getByLabel('Cell 1,2').getByLabel('Has override')).toBeVisible()

  // Selecting the cell re-seeds the panel from the loaded override.
  await page.getByLabel('Cell 1,2').click()
  await expect(page.getByRole('combobox', { name: 'Type' })).toHaveValue('ghost')
  await expect(page.getByRole('spinbutton', { name: 'Damage' })).toHaveValue('2')
})

test('the override panel is hidden when a non-feature cell is selected', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Enemy' }).click()
  await expect(page.getByRole('combobox', { name: 'Type' })).toBeVisible()

  // Cell (0,0) is the start — not overridable — so selecting it hides the panel.
  // (Walls are overridable now, so a wall cell would keep the panel open.)
  await page.getByLabel('Cell 1,1').click()
  await expect(page.getByRole('combobox', { name: 'Type' })).not.toBeVisible()
})

test('authoring a lava wall override shows the lava sprite plus a badge and saves', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // Stamp a wall on the empty cell (0,1), which opens the two-tier wall panel.
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await expect(page.getByLabel('Cell 1,2').getByAltText('Wall')).toBeVisible()

  // Choosing the special "Lava" type swaps the sprite and adds an override badge.
  const typeSelect = page.getByRole('combobox', { name: 'Type' })
  await expect(typeSelect).toBeVisible()
  await typeSelect.selectOption('lava')
  await expect(page.getByLabel('Cell 1,2').getByAltText('Wall')).toHaveAttribute('src', /lava\.svg/)
  await expect(page.getByLabel('Cell 1,2').getByLabel('Has override')).toBeVisible()

  // Saving runs the real build-definition codec; the button disables on success and
  // no save error appears.
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
  await expect(page.locator('.error-msg')).not.toBeVisible()
})

test('editing game settings marks the maze dirty and the maze save succeeds', async ({ page }) => {
  await login(page)
  await openFirstMaze(page) // Alpha — a clean maze

  // The toolbar Save starts disabled (nothing to save on a freshly-loaded maze).
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()

  // Open the per-maze game settings editor from the toolbar gear.
  await page.getByRole('button', { name: 'Game settings' }).click()
  const dialog = page.getByRole('dialog', { name: /game settings/i })
  await expect(dialog).toBeVisible()

  // Change a value and save the settings — the modal closes.
  await dialog.getByLabel(/sky/i).selectOption('day')
  await dialog.getByRole('button', { name: 'Save' }).click()
  await expect(dialog).toBeHidden()

  // The settings edit marked the maze dirty → the toolbar Save is now enabled,
  // and saving the maze succeeds (button disables, no error).
  const saveBtn = page.getByRole('button', { name: 'Save' })
  await expect(saveBtn).toBeEnabled()
  await saveBtn.click()
  await expect(saveBtn).toBeDisabled()
  await expect(page.locator('.error-msg')).not.toBeVisible()
})

test('a maze with persisted game settings seeds the settings editor', async ({ page }) => {
  await login(page)
  // Deep-link to a maze whose stored definition carries game settings.
  await page.goto('/mazes/maze-settings')
  await expect(page.locator('.maze-grid-container')).toBeVisible()

  await page.getByRole('button', { name: 'Game settings' }).click()
  const dialog = page.getByRole('dialog', { name: /game settings/i })
  await expect(dialog).toBeVisible()

  // The modal is seeded from the maze's saved settings, not the localStorage defaults.
  await expect(dialog.getByLabel(/sky/i)).toHaveValue('day')
  await expect(dialog.getByLabel(/wall texture/i)).toHaveValue('wood')
  await expect(dialog.getByLabel(/time limit/i)).toHaveValue('222')
})

test('Apply to all stamps a block of wall cells with one override', async ({ page }) => {
  await login(page)
  await openFirstMaze(page) // Alpha — a 3x3 grid

  // Select a 2x2 block and fill it with walls (Set Wall fills the whole selection),
  // leaving a uniform 'W' block still selected.
  await page.getByLabel('Cell 1,2').click()
  await page.getByLabel('Cell 2,3').click({ modifiers: ['Shift'] })
  await page.getByRole('button', { name: 'Set Wall' }).click()

  // The two-tier wall panel shows for the block; choosing Lava live-applies to the
  // top-left cell only.
  const typeSelect = page.getByRole('combobox', { name: 'Type' })
  await expect(typeSelect).toBeVisible()
  await typeSelect.selectOption('lava')
  await expect(page.getByLabel('Cell 1,2').getByAltText('Wall')).toHaveAttribute('src', /lava\.svg/)
  await expect(page.getByLabel('Cell 2,3').getByLabel('Has override')).toHaveCount(0)

  // Apply-to-all propagates the lava override across the whole 4-cell block.
  await page.getByRole('button', { name: 'Apply to all 4 cells' }).click()
  for (const label of ['Cell 1,2', 'Cell 1,3', 'Cell 2,2', 'Cell 2,3']) {
    await expect(page.getByLabel(label).getByAltText('Wall')).toHaveAttribute('src', /lava\.svg/)
    await expect(page.getByLabel(label).getByLabel('Has override')).toBeVisible()
  }

  // Saving runs the real codec; success disables the button with no error.
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled()
  await expect(page.locator('.error-msg')).not.toBeVisible()
})

test('the override panel is hidden for a mixed-type selection', async ({ page }) => {
  await login(page)
  await openFirstMaze(page)

  // A single wall cell opens the two-tier panel...
  await page.getByLabel('Cell 1,2').click()
  await page.getByRole('button', { name: 'Set Wall' }).click()
  await expect(page.getByRole('combobox', { name: 'Type' })).toBeVisible()

  // ...but extending the selection to include the start cell (0,0) makes the block
  // mixed (S + W), so the panel hides.
  await page.getByLabel('Cell 1,1').click({ modifiers: ['Shift'] })
  await expect(page.getByRole('combobox', { name: 'Type' })).not.toBeVisible()
})
