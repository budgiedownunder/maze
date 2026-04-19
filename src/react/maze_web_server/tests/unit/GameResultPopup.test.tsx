import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GameResultPopup } from '../../src/components/GameResultPopup'

beforeEach(() => {
  // jsdom doesn't implement showModal; stub it and mark the dialog open so
  // testing-library can query its contents without hidden:true everywhere.
  HTMLDialogElement.prototype.showModal = vi.fn().mockImplementation(function(this: HTMLDialogElement) {
    this.setAttribute('open', '')
  })
})

function renderPopup(onClose = vi.fn()) {
  return render(<GameResultPopup message="You did it!" onClose={onClose} />)
}

describe('GameResultPopup', () => {
  it('renders the message', () => {
    renderPopup()
    expect(screen.getByText('You did it!')).toBeInTheDocument()
  })

  it('renders celebrate.gif', () => {
    renderPopup()
    expect(screen.getByAltText('Celebration')).toHaveAttribute('src', '/images/maze/celebrate.gif')
  })

  it('Close button calls onClose', async () => {
    const onClose = vi.fn()
    renderPopup(onClose)
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('Escape key does not dismiss — onCancel is prevented', () => {
    renderPopup()
    const dialog = screen.getByRole('dialog', { hidden: true })
    const event = new Event('cancel', { cancelable: true, bubbles: false })
    fireEvent(dialog, event)
    expect(event.defaultPrevented).toBe(true)
  })
})
