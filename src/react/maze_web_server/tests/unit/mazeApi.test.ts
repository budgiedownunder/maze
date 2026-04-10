import { describe, it, expect, beforeEach } from 'vitest'
import { getMazes, getMaze, createMaze, updateMaze, deleteMaze } from '../../src/api/client'
import { mockMazeAlpha, mockMazeBeta, resetMockMazes } from '../../src/mocks/handlers'

const TOKEN = 'test-token'

beforeEach(() => {
  resetMockMazes()
})

describe('getMazes', () => {
  it('returns the list of mazes with definitions when includeDefinitions is true', async () => {
    const result = await getMazes(TOKEN, true)

    expect(result).toHaveLength(2)
    expect(result[0]).toEqual(mockMazeAlpha)
    expect(result[1]).toEqual(mockMazeBeta)
  })

  it('returns the list of mazes without definitions when includeDefinitions is false', async () => {
    const result = await getMazes(TOKEN, false)

    expect(result).toHaveLength(2)
    expect(result[0].id).toBe(mockMazeAlpha.id)
    expect(result[0].name).toBe(mockMazeAlpha.name)
  })
})

describe('getMaze', () => {
  it('returns the maze for the given id', async () => {
    const result = await getMaze(TOKEN, mockMazeAlpha.id)

    expect(result).toEqual(mockMazeAlpha)
  })

  it('throws for an unknown id', async () => {
    await expect(getMaze(TOKEN, 'nonexistent')).rejects.toThrow()
  })
})

describe('createMaze', () => {
  it('creates a new maze and returns it with a server-assigned id', async () => {
    const body = { name: 'Gamma', definition: { grid: [['S', 'F']] } }

    const result = await createMaze(TOKEN, body)

    expect(result.name).toBe('Gamma')
    expect(result.id).toBeTruthy()
    expect(result.id).not.toBe('')
  })
})

describe('updateMaze', () => {
  it('updates the maze name and returns the updated maze', async () => {
    const body = { name: 'Alpha Renamed', definition: mockMazeAlpha.definition }

    const result = await updateMaze(TOKEN, mockMazeAlpha.id, body)

    expect(result.id).toBe(mockMazeAlpha.id)
    expect(result.name).toBe('Alpha Renamed')
  })

  it('throws for an unknown id', async () => {
    const body = { name: 'X', definition: { grid: [] } }

    await expect(updateMaze(TOKEN, 'nonexistent', body)).rejects.toThrow()
  })
})

describe('deleteMaze', () => {
  it('deletes the maze without throwing', async () => {
    await expect(deleteMaze(TOKEN, mockMazeAlpha.id)).resolves.toBeUndefined()
  })

  it('throws for an unknown id', async () => {
    await expect(deleteMaze(TOKEN, 'nonexistent')).rejects.toThrow()
  })
})
