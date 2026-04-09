import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { AboutModal } from '../../src/components/AboutModal'

describe('AboutModal', () => {
  it('renders app name and version', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('Maze Application v1.0')).toBeInTheDocument()
  })

  it('renders copyright', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('© BudgieDownUnder, 2026')).toBeInTheDocument()
  })

  it('renders description', () => {
    render(<AboutModal onClose={() => {}} />)
    expect(screen.getByText('An app for designing and solving mazes')).toBeInTheDocument()
  })

  it('calls onClose when Close button is clicked', async () => {
    const onClose = vi.fn()
    render(<AboutModal onClose={onClose} />)
    await userEvent.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })
})
