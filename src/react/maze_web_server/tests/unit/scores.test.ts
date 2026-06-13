import { describe, it, expect } from 'vitest'
import { buildChallenge, SCORE_METRICS, SORT_DIRECTIONS } from '../../src/utils/scores'

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
