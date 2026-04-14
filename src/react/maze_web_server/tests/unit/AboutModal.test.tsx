import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { AboutModal } from '../../src/components/AboutModal'

describe('AboutModal', () => {
  it('renders app name', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('Maze')).toBeInTheDocument()
  })

  it('renders copyright', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('© BudgieDownUnder, 2026')).toBeInTheDocument()
  })

  it('renders description', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('Maze designer and solver')).toBeInTheDocument()
  })

  it('calls onClose when Close button is clicked', async () => {
    const onClose = vi.fn()
    render(<AboutModal onClose={onClose} />)
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})
