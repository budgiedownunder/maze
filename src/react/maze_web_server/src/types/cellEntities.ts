// Cell-entity domain types. The per-type rig style unions are derived from the
// single-source `as const` value lists in `utils/cellEntityStyles.ts` and re-exported
// here, so every cell-entity type is importable from one place AND the unions can
// never drift from the runtime arrays. The structural override (`CellEntity`) model is
// declared below. The import/re-export is type-only — this file pulls in no runtime
// code despite referencing the styles module.

import type {
  DoorStyle,
  EnemyType,
  HealthStyle,
  KeyHolderStyle,
  TreasureStyle,
  WallType,
} from '../utils/cellEntityStyles'

export type { DoorStyle, EnemyType, HealthStyle, KeyHolderStyle, TreasureStyle, WallType }

// Wire-shape single-entity override objects, mirroring the Rust `#[serde(tag="type")]`
// `CellEntity` enum and the C# polymorphic `CellEntityInfo` hierarchy. Each variant
// carries only the fields meaningful to its type, so an invalid field/type
// combination is unrepresentable. Every field is optional — the canonical form omits
// any field left at its default.
export interface EnemyCellEntity  { type: 'E'; enemyType?: EnemyType;     damage?: number; movePeriodMs?: number }
export interface HealthCellEntity { type: 'H'; healthStyle?: HealthStyle; healAmount?: number }
export interface KeyCellEntity    { type: 'K'; keyHolder?: KeyHolderStyle }
export interface DoorCellEntity   { type: 'D'; doorStyle?: DoorStyle }
export interface WallCellEntity   { type: 'W'; wallType?: WallType }
export interface TreasureCellEntity { type: 'T'; style?: TreasureStyle; value?: number }
export type CellEntity = EnemyCellEntity | HealthCellEntity | KeyCellEntity | DoorCellEntity | WallCellEntity | TreasureCellEntity

/** The grid chars that can carry an override — derived from the entity discriminators. */
export type FeatureChar = CellEntity['type']

/** A per-cell override located at a (row, col) in the grid. */
export interface CellOverride { row: number; col: number; entity: CellEntity }

// A grid cell on the wire: a bare char, or an array-of-one entity object when the cell
// is overridden (decision: char unless overridden). This char-or-array form is
// produced and parsed only by the Rust serializer — JS never constructs it by hand.
export type WireCell = string | CellEntity[]

/** The canonical maze definition as (de)serialized by the WASM layer (char-or-array cells). */
export interface CanonicalMazeDefinition { grid: WireCell[][] }
