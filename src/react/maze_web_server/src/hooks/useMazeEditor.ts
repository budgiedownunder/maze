import { useState, useCallback } from 'react'
import type { MazeDefinition } from '../types/api'

export interface CellPoint {
  row: number
  col: number
}

export function useMazeEditor() {
  const [grid, setGrid] = useState<string[][]>([])
  const [mazeName, setMazeName] = useState('')
  const [mazeId, setMazeId] = useState<string | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  const [activeCell, setActiveCell] = useState<CellPoint | null>(null)
  const [anchorCell, setAnchorCell] = useState<CellPoint | null>(null)
  const [solution, setSolution] = useState<Array<CellPoint> | null>(null)

  const initFromDefinition = useCallback(
    (id: string | null, name: string, definition: MazeDefinition) => {
      setMazeId(id)
      setMazeName(name)
      setGrid(definition.grid)
      setIsDirty(false)
      setActiveCell(null)
      setAnchorCell(null)
      setSolution(null)
    },
    [],
  )

  return {
    grid,
    mazeName,
    mazeId,
    isDirty,
    activeCell,
    anchorCell,
    solution,
    initFromDefinition,
  }
}
