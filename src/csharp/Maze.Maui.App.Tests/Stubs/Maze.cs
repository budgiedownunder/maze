// Stub of Maze.Api.Maze. The production class is backed by Wasmtime/wasm
// native libs that can't be loaded inside the bare net10.0 test host. The
// MazesViewModel test surface only needs construction, Solve, ToJson/
// FromJson, Dispose, and the row/column count properties — and only the
// guarded happy paths actually invoke them. This no-op stub preserves
// instance state so the analyzer is satisfied that each method touches
// something on `this`.
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

        public string ToJson() => Json;
        public void FromJson(string json) => Json = json;
        public void Solve() => Solved = true;
        public void Dispose() { }
    }
}
