// @vitest-environment node
//
// Integration tests for the per-cell override (cell-entity) codec — run against the
// real maze_wasm binary. They verify that splitDefinition / buildDefinitionWithOverrides
// round-trip the char-or-array cell form correctly without any JS-side parsing: a
// pure maze stays byte-identical, and an overridden maze survives build -> split
// unchanged for all four entity types.
//
// Node cannot use the default fetch-based WASM loader, so we pass the binary bytes
// directly to init(). The ensureInit() singleton in mazeWasm.ts sees wasm already
// initialised and its own init() call becomes a no-op.

import { describe, it, expect, beforeAll } from 'vitest'
import { readFileSync } from 'fs'
import { fileURLToPath } from 'url'
import { resolve, dirname } from 'path'
import init from 'maze_wasm'
import { splitDefinition, buildDefinitionWithOverrides } from '../../src/wasm/mazeWasm'
import type { CellOverride } from '../../src/types/cellEntities'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasmPath = resolve(__dirname, '../../../../rust/maze_wasm/pkg/maze_wasm_bg.wasm')

beforeAll(async () => {
  await init(readFileSync(wasmPath))
})

// Wraps a pure-char grid + overrides into the full maze JSON string that
// splitDefinition expects, by first building the canonical definition.
async function fullMazeJson(grid: string[][], overrides: CellOverride[]): Promise<string> {
  const definition = await buildDefinitionWithOverrides(grid, overrides)
  return JSON.stringify({ id: 'test', name: 'test', definition })
}

// ── splitDefinition — pure maze ─────────────────────────────────────────────────

describe('splitDefinition (real WASM) — pure maze', () => {
  it('returns the char grid unchanged and no overrides', async () => {
    const grid = [
      ['S', ' ', 'F'],
      ['W', ' ', 'W'],
    ]
    const json = JSON.stringify({ id: '', name: '', definition: { grid } })
    const { grid: outGrid, overrides } = await splitDefinition(json)
    expect(outGrid).toEqual(grid)
    expect(overrides).toEqual([])
  })
})

// ── buildDefinitionWithOverrides — canonical form ───────────────────────────────

describe('buildDefinitionWithOverrides (real WASM) — canonical form', () => {
  it('emits a pure-char grid (byte-identical) when there are no overrides', async () => {
    const grid = [['S', ' ', 'F']]
    const def = await buildDefinitionWithOverrides(grid, [])
    expect(JSON.stringify(def.grid)).toBe(JSON.stringify(grid))
  })

  it('emits an array-of-one only for the overridden cell, bare chars elsewhere', async () => {
    const grid = [['S', 'E', 'F']]
    const def = await buildDefinitionWithOverrides(grid, [
      { row: 0, col: 1, entity: { type: 'E', enemyType: 'ghost', damage: 2 } },
    ])
    expect(def.grid[0][0]).toBe('S')
    expect(def.grid[0][2]).toBe('F')
    expect(def.grid[0][1]).toEqual([{ type: 'E', enemyType: 'ghost', damage: 2 }])
  })

  it('drops a field-less override (normalises back to a bare char)', async () => {
    const grid = [['S', 'K', 'F']]
    const def = await buildDefinitionWithOverrides(grid, [
      { row: 0, col: 1, entity: { type: 'K' } },
    ])
    expect(def.grid[0][1]).toBe('K')
  })

  it('throws when an override type does not match the grid cell char', async () => {
    const grid = [['S', 'E', 'F']]
    await expect(
      buildDefinitionWithOverrides(grid, [
        { row: 0, col: 1, entity: { type: 'H', healthStyle: 'potion' } },
      ]),
    ).rejects.toThrow()
  })
})

// ── Round-trip — build -> split — per entity type ───────────────────────────────

describe('cell-entity codec round-trip (real WASM)', () => {
  it('round-trips an enemy override', async () => {
    const grid = [['S', 'E', 'F']]
    const overrides: CellOverride[] = [
      { row: 0, col: 1, entity: { type: 'E', enemyType: 'ghost', damage: 2, movePeriodMs: 500 } },
    ]
    const { grid: outGrid, overrides: outOverrides } = await splitDefinition(await fullMazeJson(grid, overrides))
    expect(outGrid).toEqual(grid)
    expect(outOverrides).toEqual(overrides)
  })

  it('round-trips a health override', async () => {
    const grid = [['S', 'H', 'F']]
    const overrides: CellOverride[] = [
      { row: 0, col: 1, entity: { type: 'H', healthStyle: 'potion', healAmount: 3 } },
    ]
    const { grid: outGrid, overrides: outOverrides } = await splitDefinition(await fullMazeJson(grid, overrides))
    expect(outGrid).toEqual(grid)
    expect(outOverrides).toEqual(overrides)
  })

  it('round-trips a key override', async () => {
    const grid = [['S', 'K', 'F']]
    const overrides: CellOverride[] = [
      { row: 0, col: 1, entity: { type: 'K', keyHolder: 'chest' } },
    ]
    const { grid: outGrid, overrides: outOverrides } = await splitDefinition(await fullMazeJson(grid, overrides))
    expect(outGrid).toEqual(grid)
    expect(outOverrides).toEqual(overrides)
  })

  it('round-trips a door override', async () => {
    const grid = [['S', 'D', 'F']]
    const overrides: CellOverride[] = [
      { row: 0, col: 1, entity: { type: 'D', doorStyle: 'portcullis' } },
    ]
    const { grid: outGrid, overrides: outOverrides } = await splitDefinition(await fullMazeJson(grid, overrides))
    expect(outGrid).toEqual(grid)
    expect(outOverrides).toEqual(overrides)
  })

  it('round-trips multiple overrides of mixed types in one maze', async () => {
    const grid = [
      ['S', 'E', 'H'],
      ['K', 'D', 'F'],
    ]
    const overrides: CellOverride[] = [
      { row: 0, col: 1, entity: { type: 'E', enemyType: 'ghost' } },
      { row: 0, col: 2, entity: { type: 'H', healAmount: 5 } },
      { row: 1, col: 0, entity: { type: 'K', keyHolder: 'floating_key' } },
      { row: 1, col: 1, entity: { type: 'D', doorStyle: 'slide' } },
    ]
    const { grid: outGrid, overrides: outOverrides } = await splitDefinition(await fullMazeJson(grid, overrides))
    expect(outGrid).toEqual(grid)
    // Overrides come back in row-major scan order, matching the fixture above.
    expect(outOverrides).toEqual(overrides)
  })
})
