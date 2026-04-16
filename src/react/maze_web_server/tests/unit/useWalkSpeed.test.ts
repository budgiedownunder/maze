import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useWalkSpeed, SPEED_LEVELS } from '../../src/hooks/useWalkSpeed'

const STORAGE_KEY = 'walkSpeed'
const DEFAULT_INDEX = 1 // Normal

beforeEach(() => {
  localStorage.clear()
})

afterEach(() => {
  localStorage.clear()
})

describe('useWalkSpeed — defaults', () => {
  it('starts at Normal (index 1) when localStorage is empty', () => {
    const { result } = renderHook(() => useWalkSpeed())
    expect(result.current.speedIndex).toBe(DEFAULT_INDEX)
    expect(result.current.speedLabel).toBe('Normal')
    expect(result.current.stepMs).toBe(500)
  })

  it('speedRef.current matches stepMs on mount', () => {
    const { result } = renderHook(() => useWalkSpeed())
    expect(result.current.speedRef.current).toBe(result.current.stepMs)
  })

  it('SPEED_LEVELS has Slow, Normal, Fast, Turbo in order', () => {
    expect(SPEED_LEVELS.map(l => l.label)).toEqual(['Slow', 'Normal', 'Fast', 'Turbo'])
  })

  it('Turbo is the last level at 20ms', () => {
    const last = SPEED_LEVELS[SPEED_LEVELS.length - 1]
    expect(last.label).toBe('Turbo')
    expect(last.ms).toBe(20)
  })
})

describe('useWalkSpeed — setSpeedIndex', () => {
  it('moves to the specified index', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(2))
    expect(result.current.speedIndex).toBe(2)
    expect(result.current.speedLabel).toBe('Fast')
    expect(result.current.stepMs).toBe(200)
  })

  it('can select Turbo (last level)', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(SPEED_LEVELS.length - 1))
    expect(result.current.speedLabel).toBe('Turbo')
    expect(result.current.stepMs).toBe(20)
  })

  it('clamps below 0', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(-5))
    expect(result.current.speedIndex).toBe(0)
    expect(result.current.speedLabel).toBe('Slow')
  })

  it('clamps above the last index', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(99))
    expect(result.current.speedIndex).toBe(SPEED_LEVELS.length - 1)
    expect(result.current.speedLabel).toBe('Turbo')
  })
})

describe('useWalkSpeed — speedRef stays in sync', () => {
  it('speedRef.current updates when speed changes', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(SPEED_LEVELS.length - 1))
    expect(result.current.speedRef.current).toBe(20)
  })
})

describe('useWalkSpeed — localStorage persistence', () => {
  it('saves index to localStorage on change', () => {
    const { result } = renderHook(() => useWalkSpeed())
    act(() => result.current.setSpeedIndex(2))
    expect(localStorage.getItem(STORAGE_KEY)).toBe('2')
  })

  it('loads saved index from localStorage on mount', () => {
    localStorage.setItem(STORAGE_KEY, String(SPEED_LEVELS.length - 1)) // Turbo
    const { result } = renderHook(() => useWalkSpeed())
    expect(result.current.speedIndex).toBe(SPEED_LEVELS.length - 1)
    expect(result.current.speedLabel).toBe('Turbo')
  })

  it('falls back to default for invalid localStorage value', () => {
    localStorage.setItem(STORAGE_KEY, 'not-a-number')
    const { result } = renderHook(() => useWalkSpeed())
    expect(result.current.speedIndex).toBe(DEFAULT_INDEX)
  })

  it('falls back to default for out-of-range localStorage value', () => {
    localStorage.setItem(STORAGE_KEY, '99')
    const { result } = renderHook(() => useWalkSpeed())
    expect(result.current.speedIndex).toBe(DEFAULT_INDEX)
  })
})
