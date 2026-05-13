import { test, expect, type Page } from '@playwright/test'

async function login(page: Page) {
  await page.goto('/login')
  await page.getByLabel('Email').fill('test@example.com')
  await page.getByLabel('Password', { exact: true }).fill('Password1!')
  await page.getByRole('button', { name: /sign in/i }).click()
  await expect(page).toHaveURL(/\/$/)
}

async function mintVerificationToken(page: Page, email: string): Promise<string> {
  // Drives the request endpoint through the live React app, which lands a
  // token in the MSW mock state. Reading that map is the standing pattern
  // for the verification-confirm flow (see project_e2e_network_strategy.md).
  await page.evaluate(async (target) => {
    await fetch('/api/v1/email-verifications/request', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Bearer test-token',
      },
      body: JSON.stringify({ email: target }),
    })
  }, email)
  const token = await page.evaluate(() => {
    const state = (window as unknown as { __mswState?: { verificationTokens: Map<string, string> } }).__mswState
    return state ? [...state.verificationTokens.keys()][0] : null
  })
  if (!token) throw new Error('No verification token minted in MSW state')
  return token
}

test('Verify Email without a token shows the invalid state', async ({ page }) => {
  await page.goto('/verify-email')
  await expect(page.getByRole('alert')).toContainText(/invalid or has already been used/i)
})

test('Verify Email with an unknown token shows the invalid state', async ({ page }) => {
  await page.goto('/verify-email?token=not-a-real-token')
  await expect(page.getByRole('alert')).toContainText(/invalid or has already been used/i)
})

test('Verify Email with a freshly-minted token reaches the success state', async ({ page }) => {
  await login(page)
  const token = await mintVerificationToken(page, 'test@example.com')

  await page.goto(`/verify-email?token=${encodeURIComponent(token)}`)
  await expect(page.getByRole('status')).toContainText(/email verified/i)
})

test('Verify Email re-submitting a consumed token shows the invalid state', async ({ page }) => {
  await login(page)
  const token = await mintVerificationToken(page, 'test@example.com')

  await page.goto(`/verify-email?token=${encodeURIComponent(token)}`)
  await expect(page.getByRole('status')).toContainText(/email verified/i)

  // Same token again — server (mock) should now reject as consumed.
  await page.goto(`/verify-email?token=${encodeURIComponent(token)}`)
  await expect(page.getByRole('alert')).toContainText(/invalid or has already been used/i)
})
