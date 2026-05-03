using Maze.Interop;

namespace Maze.Api
{
    /// <summary>Direction of player movement within the maze (when viewed in 2D from above).</summary>
    public enum MazeGameDirection
    {
        /// <summary>No direction — initial state before the player's first move.</summary>
        None = 0,
        /// <summary>Move toward lower row indices.</summary>
        Up = 1,
        /// <summary>Move toward higher row indices.</summary>
        Down = 2,
        /// <summary>Move toward lower column indices.</summary>
        Left = 3,
        /// <summary>Move toward higher column indices.</summary>
        Right = 4
    }

    /// <summary>Outcome of a player move attempt.</summary>
    public enum MazeGameMoveResult
    {
        /// <summary>No action — returned when <see cref="MazeGameDirection.None"/> is passed to <see cref="MazeGame.MovePlayer"/>.</summary>
        None = 0,
        /// <summary>The player moved successfully.</summary>
        Moved = 1,
        /// <summary>The move was blocked by a wall or grid boundary.</summary>
        Blocked = 2,
        /// <summary>The player reached the finish cell — the game is complete.</summary>
        Complete = 3
    }

    /// <summary>A cell visited by the player, identified by its zero-based row and column.</summary>
    public record MazeGameVisitedCell(int Row, int Col);

    /// <summary>
    /// A running maze game session driven by the <c>maze_wasm</c> / <c>maze_c</c> library.
    /// Create via <see cref="Create"/>. Dispose when done to free the native resource.
    /// </summary>
    public sealed class MazeGame : IDisposable
    {
        static readonly MazeInterop _interop = MazeInterop.GetInstance();

        /// <summary>
        /// When true (the default), all instances share the static <see cref="MazeInterop"/> singleton.
        /// Set to false in tests that require an isolated interop instance.
        /// </summary>
        public static bool UseStaticInterop { get; set; } = true;

        /// <summary>Returns the <see cref="MazeInterop"/> instance used by this game session.</summary>
        public static MazeInterop Interop => UseStaticInterop ? _interop : MazeInterop.GetInstance();

        private UIntPtr _gamePtr;
        private bool _disposed;

        private MazeGame(UIntPtr gamePtr) { _gamePtr = gamePtr; }

        /// <summary>Finalizer — releases the native game session if <see cref="Dispose()"/> was not called.</summary>
        ~MazeGame() { Dispose(false); }

        /// <inheritdoc/>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        [System.Diagnostics.CodeAnalysis.SuppressMessage("Style", "IDE0060:Remove unused parameter",
            Justification = "Standard IDisposable+finalizer dispatcher pattern; 'disposing' must remain in the signature so future managed-cleanup logic can branch on it.")]
        private void Dispose(bool disposing)
        {
            if (_disposed) return;
            if (_gamePtr != UIntPtr.Zero)
            {
                Interop.FreeMazeGame(_gamePtr);
                _gamePtr = UIntPtr.Zero;
            }
            _disposed = true;
        }

        /// <summary>
        /// Creates a new game session from a maze definition JSON string.
        /// Throws if the JSON is invalid or the maze has no start cell.
        /// </summary>
        /// <param name="definitionJson">
        /// Maze definition JSON — the <c>{"grid":[...]}</c> portion only, not the full maze JSON.
        /// </param>
        /// <returns>A new <see cref="MazeGame"/> positioned at the start cell.</returns>
        public static MazeGame Create(string definitionJson)
        {
            var interop = UseStaticInterop ? _interop : MazeInterop.GetInstance();
            UIntPtr ptr = interop.NewMazeGame(definitionJson);
            return new MazeGame(ptr);
        }

        /// <summary>Attempts to move the player one cell in the given direction.</summary>
        /// <param name="direction">The direction to move.</param>
        /// <returns>The outcome of the move attempt.</returns>
        public MazeGameMoveResult MovePlayer(MazeGameDirection direction)
            => (MazeGameMoveResult)Interop.MazeGameMovePlayer(_gamePtr, (int)direction);

        /// <summary>Current player row (zero-based).</summary>
        public int PlayerRow => Interop.MazeGamePlayerRow(_gamePtr);

        /// <summary>Current player column (zero-based).</summary>
        public int PlayerCol => Interop.MazeGamePlayerCol(_gamePtr);

        /// <summary>Current player facing direction.</summary>
        public MazeGameDirection PlayerDirection
            => (MazeGameDirection)Interop.MazeGamePlayerDirection(_gamePtr);

        /// <summary>Whether the player has reached the finish cell.</summary>
        public bool IsComplete => Interop.MazeGameIsComplete(_gamePtr) != 0;

        /// <summary>All cells visited by the player (including the start cell), in visit order.</summary>
        public IReadOnlyList<MazeGameVisitedCell> VisitedCells
        {
            get
            {
                int count = Interop.MazeGameVisitedCellCount(_gamePtr);
                var cells = new List<MazeGameVisitedCell>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetVisitedCell(_gamePtr, i, out int row, out int col))
                        cells.Add(new MazeGameVisitedCell(row, col));
                }
                return cells;
            }
        }
    }
}
