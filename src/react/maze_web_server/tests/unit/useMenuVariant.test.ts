import { describe, it, expect } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useMenuVariant } from '../../src/hooks/useMenuVariant'

describe('useMenuVariant', () => {
  it('returns hamburger', () => {
    const { result } = renderHook(() => useMenuVariant())
    expect(result.current).toBe('hamburger')
  })
})
