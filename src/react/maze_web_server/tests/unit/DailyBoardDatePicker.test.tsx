import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DailyBoardDatePicker } from '../../src/components/DailyBoardDatePicker'
import type { BoardDateOption } from '../../src/utils/gameDefinitions'

const OPTIONS: BoardDateOption[] = [
  { value: '2026-07-26', label: 'Today' },
  { value: '2026-07-20', label: '20 Jul 2026' },
  { value: '2026-07-12', label: '12 Jul 2026' },
]

describe('DailyBoardDatePicker', () => {
  it('renders the day options and shows the selected value', () => {
    render(<DailyBoardDatePicker options={OPTIONS} value="2026-07-20" onChange={vi.fn()} />)

    const select = screen.getByLabelText('Day') as HTMLSelectElement
    expect(select.value).toBe('2026-07-20')
    // Today pinned first, then the past days most-recent first.
    expect(Array.from(select.options).map(o => o.textContent)).toEqual(['Today', '20 Jul 2026', '12 Jul 2026'])
  })

  it('reports the chosen day through onChange', async () => {
    const onChange = vi.fn()
    render(<DailyBoardDatePicker options={OPTIONS} value="2026-07-26" onChange={onChange} />)

    await userEvent.selectOptions(screen.getByLabelText('Day'), '2026-07-12')
    expect(onChange).toHaveBeenCalledWith('2026-07-12')
  })
})
