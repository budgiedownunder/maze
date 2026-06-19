// Stub of Maze.Api.Maze. The production class is backed by Wasmtime/wasm
// native libs that can't be loaded inside the bare net10.0 test host. The
// MazesViewModel test surface only needs construction, Solve, ToJson/
// FromJson, Dispose, the row/column count properties, and the cell-type
// surface — and only the guarded happy paths actually invoke them. This
// no-op stub preserves instance state so the analyzer is satisfied that
// each method touches something on `this`.
//
// The MaxTotalFeatures / MaxDoorCount constants, the
// ExceedsGenerateFeatureCap helper, and the CellType enum are pure data
// + arithmetic — they don't touch the wasm runtime, so they're mirrored
// here verbatim from the production class so linked source files
// (GenerateMazeOptionsParser, MazeViewModel.SaveMaze) compile inside
// this test host.
namespace Maze.Api
{
    public sealed class Maze : IDisposable
    {
        private readonly CellType[,] _cells;

        public Maze(int rowCount, int colCount)
        {
            RowCount = (UInt32)rowCount;
            ColCount = (UInt32)colCount;
            _cells = new CellType[rowCount, colCount];
        }

        public UInt32 RowCount { get; }
        public UInt32 ColCount { get; }
        public string Json { get; private set; } = "{}";
        public bool Solved { get; private set; }

        public enum CellType
        {
            Empty = 0,
            Start = 1,
            Finish = 2,
            Wall = 3,
            Key = 4,
            Door = 5,
            Enemy = 6,
            Health = 7,
            Treasure = 8,
        }

        public CellType GetCellType(UInt32 row, UInt32 col) =>
            row < RowCount && col < ColCount ? _cells[row, col] : CellType.Empty;

        public void SetStartCell(UInt32 row, UInt32 col) => Set(row, col, CellType.Start);
        public void SetFinishCell(UInt32 row, UInt32 col) => Set(row, col, CellType.Finish);
        public void SetWallCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Wall);
        public void SetKeyCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Key);
        public void SetDoorCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Door);
        public void SetEnemyCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Enemy);
        public void SetHealthCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Health);
        public void SetTreasureCells(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec) => SetRange(sr, sc, er, ec, CellType.Treasure);

        private void Set(UInt32 row, UInt32 col, CellType type)
        {
            if (row < RowCount && col < ColCount) _cells[row, col] = type;
        }

        private void SetRange(UInt32 sr, UInt32 sc, UInt32 er, UInt32 ec, CellType type)
        {
            for (UInt32 r = sr; r <= er && r < RowCount; r++)
                for (UInt32 c = sc; c <= ec && c < ColCount; c++)
                    _cells[r, c] = type;
        }

        public const UInt32 MaxTotalFeatures = 16;
        public const UInt32 MaxDoorCount = 8;
        public const UInt32 MaxEnemyCount = 8;
        public const UInt32 MaxHealthCount = 8;
        public const UInt32 MaxTreasureCount = 12;
        public static bool ExceedsGenerateFeatureCap(UInt32 doorCount, UInt32 spareDoors, UInt32 spareKeys)
            => 2 * doorCount + spareDoors + spareKeys > MaxTotalFeatures;

        public string ToJson() => Json;
        public void FromJson(string json) => Json = json;
        public string DefinitionToJson() => Json;
        public void Solve() => Solved = true;
        public void Dispose() { }
    }
}
