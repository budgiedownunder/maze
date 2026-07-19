import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SubjectSelector, type SubjectSelection } from '../../src/components/SubjectSelector'

// The game picker has its own test file; stub it here so this stays focused on
// the Game Type cascade (and needn't fetch).
vi.mock('../../src/components/LeaderboardGamePicker', () => ({
  LeaderboardGamePicker: ({ value, onSelect }: {
    value: { name: string } | null
    onSelect: (game: { id: string; name: string; ownerId: string; rotation: string }) => void
  }) => (
    <div>
      <span>{value ? value.name : 'no game'}</span>
      <button type="button" onClick={() => onSelect({ id: 'g1', name: 'Picked', ownerId: 'owner-1', rotation: 'static' })}>
        Pick game
      </button>
    </div>
  ),
}))

const MAZES = [
  { mazeId: 'a.json', name: 'Alpha' },
  { mazeId: 'b.json', name: 'Beta' },
]

describe('SubjectSelector', () => {
  it('labels the Mazes game type and lists maze names', () => {
    render(
      <SubjectSelector
        mazes={MAZES}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={vi.fn()}
      />,
    )
    const gameType = screen.getByLabelText('Game Type') as HTMLSelectElement
    expect([...gameType.options].map(o => o.textContent)).toEqual(['Mazes', '3D Games'])
    const game = screen.getByLabelText('Game') as HTMLSelectElement
    expect([...game.options].map(o => o.textContent)).toEqual(['Alpha', 'Beta'])
    expect(game.value).toBe('a.json')
  })

  it('renders the game picker (not a maze dropdown) for 3D Games', () => {
    render(
      <SubjectSelector
        mazes={MAZES}
        value={{ gameType: 'play3d', game: { id: 'g9', name: 'Tricky', ownerId: 'o9', rotation: 'static' } }}
        onChange={vi.fn()}
      />,
    )
    expect(screen.getByText('Tricky')).toBeInTheDocument()
    expect(screen.queryByLabelText('Game')).not.toBeInTheDocument()
  })

  it('switching Game Type to 3D Games emits an empty game selection', async () => {
    const onChange = vi.fn()
    render(
      <SubjectSelector
        mazes={MAZES}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={onChange}
      />,
    )
    await userEvent.selectOptions(screen.getByLabelText('Game Type'), 'play3d')
    expect(onChange).toHaveBeenCalledWith({ gameType: 'play3d', game: null } satisfies SubjectSelection)
  })

  it('picking a game emits the play3d subject', async () => {
    const onChange = vi.fn()
    render(
      <SubjectSelector
        mazes={MAZES}
        value={{ gameType: 'play3d', game: null }}
        onChange={onChange}
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Pick game' }))
    expect(onChange).toHaveBeenCalledWith({
      gameType: 'play3d',
      game: { id: 'g1', name: 'Picked', ownerId: 'owner-1', rotation: 'static' },
    } satisfies SubjectSelection)
  })

  it('selecting a maze emits the my-mazes subject', async () => {
    const onChange = vi.fn()
    render(
      <SubjectSelector
        mazes={MAZES}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={onChange}
      />,
    )
    await userEvent.selectOptions(screen.getByLabelText('Game'), 'b.json')
    expect(onChange).toHaveBeenCalledWith({ gameType: 'my-mazes', mazeId: 'b.json' } satisfies SubjectSelection)
  })

  it('shows a placeholder + disables the game dropdown when there are no mazes', () => {
    render(
      <SubjectSelector
        mazes={[]}
        value={{ gameType: 'my-mazes', mazeId: '' }}
        onChange={vi.fn()}
      />,
    )
    const game = screen.getByLabelText('Game') as HTMLSelectElement
    expect(game.disabled).toBe(true)
    expect(game.options[0].textContent).toBe('(no mazes)')
  })
})
