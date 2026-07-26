import { describe, it, expect } from 'vitest'
import { boardDateOptions, defaultBoardDate, formatBoardDate } from '../../src/utils/gameDefinitions'

describe('formatBoardDate', () => {
  it('formats yyyy-mm-dd as "d MMM yyyy" (no leading zero on the day)', () => {
    expect(formatBoardDate('2026-07-20')).toBe('20 Jul 2026')
    expect(formatBoardDate('2026-01-05')).toBe('5 Jan 2026')
  })

  it('falls back to the raw value when malformed', () => {
    expect(formatBoardDate('nope')).toBe('nope')
  })
})

describe('boardDateOptions', () => {
  const today = '2026-07-26'

  it('pins Today first, then the past days most-recent first', () => {
    expect(boardDateOptions(['2026-07-20', '2026-07-12'], today)).toEqual([
      { value: today, label: 'Today' },
      { value: '2026-07-20', label: '20 Jul 2026' },
      { value: '2026-07-12', label: '12 Jul 2026' },
    ])
  })

  it('always offers Today, even when the game has no runs', () => {
    expect(boardDateOptions([], today)).toEqual([{ value: today, label: 'Today' }])
  })

  it('dedupes today — the pin covers it', () => {
    expect(boardDateOptions([today, '2026-07-12'], today)).toEqual([
      { value: today, label: 'Today' },
      { value: '2026-07-12', label: '12 Jul 2026' },
    ])
  })
})

describe('defaultBoardDate', () => {
  const today = '2026-07-26'

  it('is the most-recent day that has runs', () => {
    expect(defaultBoardDate(['2026-07-20', '2026-07-12'], today)).toBe('2026-07-20')
  })

  it('is Today when today itself has runs', () => {
    expect(defaultBoardDate([today, '2026-07-12'], today)).toBe(today)
  })

  it('is Today when the game has no runs at all', () => {
    expect(defaultBoardDate([], today)).toBe(today)
  })
})
