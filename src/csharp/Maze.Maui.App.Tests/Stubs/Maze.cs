// Stub of Maze.Api.Maze. The production class is backed by Wasmtime/wasm
// native libs that can't be loaded inside the bare net10.0 test host. The
// MazesViewModel test surface only needs construction, Solve, ToJson/
// FromJson, Dispose, and the row/column count properties — and only the
// guarded happy paths actually invoke them. This no-op stub preserves
// instance state so the analyzer is satisfied that each method touches
// something on `this`.
//
// The MaxTotalFeatures / MaxDoorCount constants and the
// ExceedsGenerateFeatureCap helper are pure data + arithmetic — they
// don't touch the wasm runtime, so they're mirrored here verbatim
// from the production class so the linked GenerateMazeOptionsParser
// compiles inside this test host.
namespace Maze.Api
{
    public sealed class Maze : IDisposable
    {
        public Maze(int rowCount, int colCount)
        {
            RowCount = rowCount;
            ColCount = colCount;
        }

        public int RowCount { get; set; }
        public int ColCount { get; set; }
        public string Json { get; private set; } = "{}";
        public bool Solved { get; private set; }

        public const UInt32 MaxTotalFeatures = 16;
        public const UInt32 MaxDoorCount = 8;
        public static bool ExceedsGenerateFeatureCap(UInt32 doorCount, UInt32 spareDoors, UInt32 spareKeys)
            => 2 * doorCount + spareDoors + spareKeys > MaxTotalFeatures;

        public string ToJson() => Json;
        public void FromJson(string json) => Json = json;
        public string DefinitionToJson() => Json;
        public void Solve() => Solved = true;
        public void Dispose() { }
    }
}
