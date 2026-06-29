import { describe, it, expect } from 'vitest'
import { buildChallenge, formatElapsedMs, SCORE_METRICS, SORT_DIRECTIONS } from '../../src/utils/scores'

describe('buildChallenge', () => {
  it('joins difficulty and seed as "<difficulty>:<seed>"', () => {
    expect(buildChallenge('easy', 42)).toBe('easy:42')
    expect(buildChallenge('hard', 0)).toBe('hard:0')
  })
})

describe('score vocabularies', () => {
  it('exposes the metric and direction query values', () => {
    expect(SCORE_METRICS).toEqual(['time', 'score'])
    expect(SORT_DIRECTIONS).toEqual(['asc', 'desc'])
  })
})

describe('formatElapsedMs', () => {
  it('formats as m:ss.mmm with zero-padding', () => {
    expect(formatElapsedMs(42137)).toBe('0:42.137')
    expect(formatElapsedMs(90500)).toBe('1:30.500')
    expect(formatElapsedMs(5)).toBe('0:00.005')
    expect(formatElapsedMs(0)).toBe('0:00.000')
  })

  it('clamps negatives to zero', () => {
    expect(formatElapsedMs(-100)).toBe('0:00.000')
  })
})
