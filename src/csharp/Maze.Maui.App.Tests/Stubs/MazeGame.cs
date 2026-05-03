// Stubs for the maze-game types from Maze.Api. The production MazeGame is
// sealed with a private constructor and its Create factory routes through
// the native maze_wasm/maze_c interop, which can't be loaded in a bare
// net10.0 test host. The MazeGameViewModel test surface only exercises
// guard-and-early-return paths that don't need a live game session, so a
// throw-on-Create stub is sufficient.
namespace Maze.Api
{
    public enum MazeGameDirection
    {
        None = 0,
        Up = 1,
        Down = 2,
        Left = 3,
        Right = 4,
    }

    public enum MazeGameMoveResult
    {
        None = 0,
        Moved = 1,
        Blocked = 2,
        Complete = 3,
    }

    public sealed class MazeGame : IDisposable
    {
        private MazeGame() { }

        public static MazeGame Create(string definitionJson)
            => throw new NotSupportedException("Maze.Api.MazeGame is stubbed in the test host; tests must avoid the StartGame happy path.");

        public MazeGameMoveResult MovePlayer(MazeGameDirection direction)
        {
            PlayerDirection = direction;
            return MazeGameMoveResult.None;
        }

        public int PlayerRow { get; set; }
        public int PlayerCol { get; set; }
        public MazeGameDirection PlayerDirection { get; set; }
        public bool IsComplete { get; set; }
        public void Dispose() { }
    }
}
