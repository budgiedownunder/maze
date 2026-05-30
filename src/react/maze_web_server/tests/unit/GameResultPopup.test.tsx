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

  it('renders celebrate.gif by default (success tone)', () => {
    renderPopup()
    expect(screen.getByAltText('Celebration')).toHaveAttribute('src', '/images/maze/celebrate.gif')
  })

  it('shows the game-over image when tone is fail', () => {
    render(<GameResultPopup message="You're stranded!!" tone="fail" onClose={vi.fn()} />)
    expect(screen.queryByAltText('Celebration')).not.toBeInTheDocument()
    expect(screen.getByAltText('Game over')).toHaveAttribute('src', '/images/maze/game_over.png')
    expect(screen.getByText("You're stranded!!")).toBeInTheDocument()
  })

  it('Close button calls onClose', async () => {
    const onClose = vi.fn()
    renderPopup(onClose)
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('renders a Play Again button only when onPlayAgain is provided, and it fires the callback', async () => {
    const onPlayAgain = vi.fn()
    const { rerender } = render(<GameResultPopup message="You win!" onClose={vi.fn()} />)
    expect(screen.queryByRole('button', { name: /play again/i })).not.toBeInTheDocument()

    rerender(<GameResultPopup message="You win!" onClose={vi.fn()} onPlayAgain={onPlayAgain} />)
    await userEvent.click(screen.getByRole('button', { name: /play again/i }))
    expect(onPlayAgain).toHaveBeenCalledOnce()
  })

  it('Escape key does not dismiss — onCancel is prevented', () => {
    renderPopup()
    const dialog = screen.getByRole('dialog', { hidden: true })
    const event = new Event('cancel', { cancelable: true, bubbles: false })
    fireEvent(dialog, event)
    expect(event.defaultPrevented).toBe(true)
  })
})
