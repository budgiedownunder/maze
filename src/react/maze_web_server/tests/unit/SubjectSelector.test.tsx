import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SubjectSelector, type SubjectSelection } from '../../src/components/SubjectSelector'

const PLAYED = [
  { mazeId: 'a.json', name: 'Alpha' },
  { mazeId: 'b.json', name: 'Beta' },
]

describe('SubjectSelector', () => {
  it('lists played maze names for My Mazes', () => {
    render(
      <SubjectSelector
        playedMazes={PLAYED}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={vi.fn()}
      />,
    )
    const game = screen.getByLabelText('Game') as HTMLSelectElement
    const labels = [...game.options].map(o => o.textContent)
    expect(labels).toEqual(['Alpha', 'Beta'])
    expect(game.value).toBe('a.json')
  })

  it('lists Easy/Tricky/Hard for Play 3D', () => {
    render(
      <SubjectSelector
        playedMazes={PLAYED}
        value={{ gameType: 'play3d', difficulty: 'easy' }}
        onChange={vi.fn()}
      />,
    )
    const game = screen.getByLabelText('Game') as HTMLSelectElement
    expect([...game.options].map(o => o.textContent)).toEqual(['Easy', 'Tricky', 'Hard'])
  })

  it('switching Game Type to Play 3D emits the first difficulty', async () => {
    const onChange = vi.fn()
    render(
      <SubjectSelector
        playedMazes={PLAYED}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={onChange}
      />,
    )
    await userEvent.selectOptions(screen.getByLabelText('Game Type'), 'play3d')
    expect(onChange).toHaveBeenCalledWith({ gameType: 'play3d', difficulty: 'easy' } satisfies SubjectSelection)
  })

  it('selecting a maze emits the my-mazes subject', async () => {
    const onChange = vi.fn()
    render(
      <SubjectSelector
        playedMazes={PLAYED}
        value={{ gameType: 'my-mazes', mazeId: 'a.json' }}
        onChange={onChange}
      />,
    )
    await userEvent.selectOptions(screen.getByLabelText('Game'), 'b.json')
    expect(onChange).toHaveBeenCalledWith({ gameType: 'my-mazes', mazeId: 'b.json' } satisfies SubjectSelection)
  })

  it('shows a placeholder + disables the game dropdown when no mazes were played', () => {
    render(
      <SubjectSelector
        playedMazes={[]}
        value={{ gameType: 'my-mazes', mazeId: '' }}
        onChange={vi.fn()}
      />,
    )
    const game = screen.getByLabelText('Game') as HTMLSelectElement
    expect(game.disabled).toBe(true)
    expect(game.options[0].textContent).toBe('(no mazes played)')
  })
})
