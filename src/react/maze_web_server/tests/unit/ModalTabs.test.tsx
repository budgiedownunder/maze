import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ModalTabStrip } from '../../src/components/ModalTabs'
import { modalTabPanelProps, type ModalTab } from '../../src/utils/modalTabs'

const TABS = [
  { id: 'a', label: 'Alpha' },
  { id: 'b', label: 'Beta' },
  { id: 'c', label: 'Gamma' },
] as const satisfies readonly ModalTab[]

function renderStrip(activeTab: 'a' | 'b' | 'c', onSelect = vi.fn()) {
  render(
    <ModalTabStrip tabs={TABS} activeTab={activeTab} onSelect={onSelect} idPrefix="test" ariaLabel="Test tabs" />,
  )
  return onSelect
}

describe('ModalTabStrip', () => {
  it('renders one tab per entry with its label and aria-selected state', () => {
    renderStrip('b')
    expect(screen.getByRole('tablist', { name: 'Test tabs' })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: 'Alpha' })).toHaveAttribute('aria-selected', 'false')
    expect(screen.getByRole('tab', { name: 'Beta' })).toHaveAttribute('aria-selected', 'true')
    // The active tab is the only tab-stop; the rest are removed from the tab order.
    expect(screen.getByRole('tab', { name: 'Beta' })).toHaveAttribute('tabindex', '0')
    expect(screen.getByRole('tab', { name: 'Alpha' })).toHaveAttribute('tabindex', '-1')
  })

  it('wires each tab to its panel id', () => {
    renderStrip('a')
    const tab = screen.getByRole('tab', { name: 'Gamma' })
    expect(tab).toHaveAttribute('id', 'test-tab-c')
    expect(tab).toHaveAttribute('aria-controls', 'test-panel-c')
  })

  it('calls onSelect with the clicked tab id', async () => {
    const onSelect = renderStrip('a')
    await userEvent.click(screen.getByRole('tab', { name: 'Beta' }))
    expect(onSelect).toHaveBeenCalledWith('b')
  })

  it('ArrowRight selects the next tab and ArrowLeft the previous', () => {
    const onSelect = renderStrip('b')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Beta' }), { key: 'ArrowRight' })
    expect(onSelect).toHaveBeenCalledWith('c')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Beta' }), { key: 'ArrowLeft' })
    expect(onSelect).toHaveBeenCalledWith('a')
  })

  it('ArrowLeft on the first tab wraps to the last', () => {
    const onSelect = renderStrip('a')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Alpha' }), { key: 'ArrowLeft' })
    expect(onSelect).toHaveBeenCalledWith('c')
  })

  it('ArrowRight on the last tab wraps to the first', () => {
    const onSelect = renderStrip('c')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Gamma' }), { key: 'ArrowRight' })
    expect(onSelect).toHaveBeenCalledWith('a')
  })

  it('ignores non-arrow keys', () => {
    const onSelect = renderStrip('a')
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Alpha' }), { key: 'Enter' })
    expect(onSelect).not.toHaveBeenCalled()
  })
})

describe('modalTabPanelProps', () => {
  it('wires the panel role/id/label and hides inactive panels', () => {
    expect(modalTabPanelProps('test', 'a', 'a')).toEqual({
      role: 'tabpanel',
      id: 'test-panel-a',
      'aria-labelledby': 'test-tab-a',
      hidden: false,
    })
    expect(modalTabPanelProps('test', 'b', 'a').hidden).toBe(true)
  })
})
