import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../../src/mocks/server'
import { resetMockEmails } from '../../src/mocks/handlers'
import { EmailAddressesPanel } from '../../src/components/EmailAddressesPanel'
import type { UserEmailsResponse } from '../../src/types/api'

const TOKEN = 'test-token'

beforeEach(() => {
  resetMockEmails()
})

async function renderPanel() {
  render(<EmailAddressesPanel token={TOKEN} />)
  // Wait for the initial GET to resolve so subsequent assertions don't race.
  await waitFor(() => expect(screen.queryByText(/loading emails/i)).not.toBeInTheDocument())
}

describe('EmailAddressesPanel', () => {
  it('shows loading state initially and then the seeded primary email with badges', async () => {
    render(<EmailAddressesPanel token={TOKEN} />)
    expect(screen.getByText(/loading emails/i)).toBeInTheDocument()

    expect(await screen.findByText('test@example.com')).toBeInTheDocument()
    expect(screen.getByText('Primary')).toBeInTheDocument()
    expect(screen.getByText('Verified')).toBeInTheDocument()
  })

  it('hides Remove on the only email and hides Make Primary on the primary row', async () => {
    await renderPanel()

    const row = screen.getByText('test@example.com').closest('li')!
    expect(within(row).queryByRole('button', { name: /^Remove$/ })).not.toBeInTheDocument()
    expect(within(row).queryByRole('button', { name: /Make Primary/ })).not.toBeInTheDocument()
  })

  it('hides Remove on the primary row when other emails exist', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true,  verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: false, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
    )

    await renderPanel()

    const oneRow = screen.getByText('one@example.com').closest('li')!
    expect(within(oneRow).queryByRole('button', { name: /^Remove$/ })).not.toBeInTheDocument()

    const twoRow = screen.getByText('two@example.com').closest('li')!
    expect(within(twoRow).getByRole('button', { name: /^Remove$/ })).toBeInTheDocument()
  })

  it('promotes a non-primary email to primary and clears the previous primary', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true,  verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: false, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
      http.put('/api/v1/users/me/emails/:email/primary', ({ params }) => {
        const target = decodeURIComponent(params.email as string)
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: target === 'one@example.com', verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: target === 'two@example.com', verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
    )

    await renderPanel()

    const twoRow = screen.getByText('two@example.com').closest('li')!
    await userEvent.click(within(twoRow).getByRole('button', { name: /Make Primary/ }))

    await waitFor(() => {
      const updatedTwoRow = screen.getByText('two@example.com').closest('li')!
      expect(within(updatedTwoRow).getByText('Primary')).toBeInTheDocument()
    })
    const oneRow = screen.getByText('one@example.com').closest('li')!
    expect(within(oneRow).queryByText('Primary')).not.toBeInTheDocument()
  })

  it('removes a non-primary email after confirming the API succeeded', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true,  verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: false, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
      http.delete('/api/v1/users/me/emails/:email', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
    )

    await renderPanel()
    expect(screen.getByText('two@example.com')).toBeInTheDocument()

    const twoRow = screen.getByText('two@example.com').closest('li')!
    await userEvent.click(within(twoRow).getByRole('button', { name: /^Remove$/ }))

    await waitFor(() => expect(screen.queryByText('two@example.com')).not.toBeInTheDocument())
    expect(screen.getByText('one@example.com')).toBeInTheDocument()
  })

  it('reverts the optimistic update and shows an error when set-primary fails', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true,  verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: false, verified: true, verified_at: '2026-01-01T00:00:00.000Z' },
          ],
        }
        return HttpResponse.json(emails)
      }),
      http.put('/api/v1/users/me/emails/:email/primary', () => {
        return new HttpResponse('Cannot promote unverified email', { status: 409 })
      }),
    )

    await renderPanel()

    const twoRow = screen.getByText('two@example.com').closest('li')!
    await userEvent.click(within(twoRow).getByRole('button', { name: /Make Primary/ }))

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument())
    expect(screen.getByRole('alert').textContent).toMatch(/failed to set primary email/i)
    // The original primary is restored.
    const oneRow = screen.getByText('one@example.com').closest('li')!
    expect(within(oneRow).getByText('Primary')).toBeInTheDocument()
  })

  it('renders a disabled Resend Verification button on unverified rows', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => {
        const emails: UserEmailsResponse = {
          emails: [
            { email: 'one@example.com', is_primary: true,  verified: true,  verified_at: '2026-01-01T00:00:00.000Z' },
            { email: 'two@example.com', is_primary: false, verified: false, verified_at: null },
          ],
        }
        return HttpResponse.json(emails)
      }),
    )

    await renderPanel()

    const twoRow = screen.getByText('two@example.com').closest('li')!
    expect(within(twoRow).getByText('Unverified')).toBeInTheDocument()
    const resend = within(twoRow).getByRole('button', { name: /Resend Verification/ })
    expect(resend).toBeDisabled()
    // Make Primary is also disabled when the row is unverified.
    expect(within(twoRow).getByRole('button', { name: /Make Primary/ })).toBeDisabled()
  })

  it('shows a load error when the initial GET fails', async () => {
    server.use(
      http.get('/api/v1/users/me/emails', () => new HttpResponse(null, { status: 500 })),
    )

    render(<EmailAddressesPanel token={TOKEN} />)

    expect(await screen.findByRole('alert')).toHaveTextContent(/failed to load emails/i)
  })

  it('keeps the Add Email button disabled until the typed address is well-formed', async () => {
    await renderPanel()

    const input = screen.getByPlaceholderText(/add another email/i)
    const button = screen.getByRole('button', { name: /^Add Email$/ })

    expect(button).toBeDisabled()
    await userEvent.type(input, 'not-an-email')
    expect(button).toBeDisabled()
    await userEvent.type(input, '@example.com')
    expect(button).toBeEnabled()
  })

  it('appends the new email on a successful add and clears the input', async () => {
    await renderPanel()

    const input = screen.getByPlaceholderText(/add another email/i)
    await userEvent.type(input, 'second@example.com')
    await userEvent.click(screen.getByRole('button', { name: /^Add Email$/ }))

    await waitFor(() => expect(screen.getByText('second@example.com')).toBeInTheDocument())
    expect(input).toHaveValue('')
    // Existing primary row still present and primary.
    const oneRow = screen.getByText('test@example.com').closest('li')!
    expect(within(oneRow).getByText('Primary')).toBeInTheDocument()
  })

  it('surfaces a 409 duplicate inline and keeps the typed value in the input', async () => {
    await renderPanel()

    const input = screen.getByPlaceholderText(/add another email/i)
    // The seeded mock list already contains test@example.com — the default
    // POST handler returns 409 on duplicates, so retyping it triggers the
    // error path.
    await userEvent.type(input, 'test@example.com')
    await userEvent.click(screen.getByRole('button', { name: /^Add Email$/ }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/already in use/i))
    expect(input).toHaveValue('test@example.com')
  })
})
