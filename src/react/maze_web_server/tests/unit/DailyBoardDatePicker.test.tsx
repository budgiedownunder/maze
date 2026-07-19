import { describe, it, expect, vi, beforeEach } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { DailyBoardDatePicker } from '../../src/components/DailyBoardDatePicker'
import { server } from '../../src/mocks/server'
import { todayUtc } from '../../src/utils/gameDefinitions'

function boardDates(dates: string[]) {
  return http.get('/api/v1/scores/board-dates', () => HttpResponse.json({ dates }))
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('DailyBoardDatePicker', () => {
  it('renders a date input capped at today with the selected value', async () => {
    server.use(boardDates([]))
    render(<DailyBoardDatePicker token="t" gameId="g1" value="2026-07-05" onChange={vi.fn()} />)

    const input = screen.getByLabelText('Day') as HTMLInputElement
    expect(input.type).toBe('date')
    expect(input.value).toBe('2026-07-05')
    expect(input.max).toBe(todayUtc())
  })

  it('renders the days-with-runs as quick-pick chips and selects one on click', async () => {
    server.use(boardDates(['2026-07-10', '2026-07-05']))
    const onChange = vi.fn()
    render(<DailyBoardDatePicker token="t" gameId="g1" value="2026-07-10" onChange={onChange} />)

    // The chips appear once the board-dates load.
    const chip = await screen.findByRole('button', { name: '2026-07-05' })
    // The selected day's chip is marked pressed.
    expect(screen.getByRole('button', { name: '2026-07-10' })).toHaveAttribute('aria-pressed', 'true')
    expect(chip).toHaveAttribute('aria-pressed', 'false')

    await userEvent.click(chip)
    expect(onChange).toHaveBeenCalledWith('2026-07-05')
  })

  it('reports a picked date through onChange but ignores a cleared input', () => {
    server.use(boardDates([]))
    const onChange = vi.fn()
    render(<DailyBoardDatePicker token="t" gameId="g1" value="2026-07-05" onChange={onChange} />)
    const input = screen.getByLabelText('Day')

    // Clearing yields an empty value, which can't key a board — no callback.
    fireEvent.change(input, { target: { value: '' } })
    expect(onChange).not.toHaveBeenCalled()

    fireEvent.change(input, { target: { value: '2026-07-08' } })
    expect(onChange).toHaveBeenCalledWith('2026-07-08')
  })

  it('shows no quick-picks when the game has no dated boards', async () => {
    server.use(boardDates([]))
    render(<DailyBoardDatePicker token="t" gameId="g1" value={todayUtc()} onChange={vi.fn()} />)

    // Give the (empty) fetch a chance to settle, then assert nothing rendered.
    await waitFor(() => expect(screen.getByLabelText('Day')).toBeInTheDocument())
    expect(screen.queryByRole('group', { name: 'Days with scores' })).not.toBeInTheDocument()
  })

  it('leaves quick-picks empty when the board-dates fetch fails', async () => {
    server.use(http.get('/api/v1/scores/board-dates', () => new HttpResponse(null, { status: 403 })))
    render(<DailyBoardDatePicker token="t" gameId="g1" value={todayUtc()} onChange={vi.fn()} />)

    await waitFor(() => expect(screen.getByLabelText('Day')).toBeInTheDocument())
    expect(screen.queryByRole('group', { name: 'Days with scores' })).not.toBeInTheDocument()
  })
})
