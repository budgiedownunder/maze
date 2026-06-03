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
        Complete = 3,
        /// <summary>The move was blocked by a locked door and the player has no key.</summary>
        BlockedByLockedDoor = 4,
        /// <summary>The player held against a locked door with a key in their bag; a key was consumed and the door began opening.</summary>
        StartedUnlocking = 5,
        /// <summary>The player moved through an open door and can no longer hold enough keys to open every remaining closed door on a route to the finish — the game is now lost.</summary>
        Stranded = 6,
        /// <summary>The player's HP reached zero from an enemy collision — the game is now lost.</summary>
        Killed = 7
    }

    /// <summary>Why a game ended in a loss. Mirrors the Rust <c>maze::LoseReason</c> enum.</summary>
    public enum LoseReason
    {
        /// <summary>The game is not lost.</summary>
        None = 0,
        /// <summary>The player can no longer hold enough keys to open every closed door remaining on a route from their current cell to the finish.</summary>
        Stranded = 1,
        /// <summary>The player's HP reached zero from enemy collisions.</summary>
        Killed = 2
    }

    /// <summary>Kind of item carried in the player's bag. Mirrors the Rust <c>maze::BagItem</c> tagged enum.</summary>
    public enum BagItemKind
    {
        /// <summary>A key that can open one door.</summary>
        Key = 0
    }

    /// <summary>One item in the player's bag — see <see cref="MazeGame.Bag"/> and <see cref="MazeGame.Pickup"/>.</summary>
    /// <param name="Kind">The kind of item.</param>
    /// <param name="Id">Stable identifier for the item (e.g. derived from the key's origin cell).</param>
    public readonly record struct BagItem(BagItemKind Kind, uint Id);

    /// <summary>Lifecycle state of a door cell. Mirrors the Rust <c>maze::DoorState</c> enum.</summary>
    public enum DoorState
    {
        /// <summary>Closed and locked; requires a key to open.</summary>
        Locked = 0,
        /// <summary>Currently opening; will transition to <see cref="Open"/> on the next sufficient tick.</summary>
        Opening = 1,
        /// <summary>Fully open and permanently passable.</summary>
        Open = 2
    }

    /// <summary>One door cell along with its current state — see <see cref="MazeGame.Doors"/>.</summary>
    /// <param name="Row">Row of the door cell.</param>
    /// <param name="Column">Column of the door cell.</param>
    /// <param name="State">Current state of the door.</param>
    public readonly record struct DoorInfo(uint Row, uint Column, DoorState State);

    /// <summary>Kind of time-based game event emitted by <see cref="MazeGame.Tick"/>.</summary>
    public enum GameEventKind
    {
        /// <summary>A door finished opening — its <see cref="DoorState"/> is now <see cref="DoorState.Open"/>. <see cref="GameEvent.Row"/> / <see cref="GameEvent.Column"/> is the door cell.</summary>
        DoorOpened = 0,
        /// <summary>An enemy advanced one cell. <see cref="GameEvent.Row"/> / <see cref="GameEvent.Column"/> is its new cell; <see cref="GameEvent.Payload"/> is the enemy id.</summary>
        EnemyMoved = 1,
        /// <summary>The player took same-cell collision damage. <see cref="GameEvent.Payload"/> is the player's HP after the hit; the cell fields are unused.</summary>
        PlayerDamaged = 2,
        /// <summary>The player consumed a health pickup. <see cref="GameEvent.Row"/> / <see cref="GameEvent.Column"/> is the consumed cell; <see cref="GameEvent.Payload"/> is the player's HP after the heal.</summary>
        PlayerHealed = 3,
        /// <summary>The player walked onto a health pickup that did not apply (already at max HP). The cell is spared; <see cref="GameEvent.Payload"/> is the reason code (0 = already at max HP).</summary>
        PlayerNotHealed = 4,
        /// <summary>The player walked onto a key and it was auto-collected into the bag. <see cref="GameEvent.Row"/> / <see cref="GameEvent.Column"/> is the consumed key cell; <see cref="GameEvent.Payload"/> is the collected key id.</summary>
        KeyCollected = 5
    }

    /// <summary>One time-based game event emitted by <see cref="MazeGame.Tick"/>.</summary>
    /// <param name="Kind">The kind of event.</param>
    /// <param name="Row">Row of the cell the event applies to (unused for <see cref="GameEventKind.PlayerDamaged"/>).</param>
    /// <param name="Column">Column of the cell the event applies to (unused for <see cref="GameEventKind.PlayerDamaged"/>).</param>
    /// <param name="Payload">Event-specific scalar: enemy id (<see cref="GameEventKind.EnemyMoved"/>), HP-after (<see cref="GameEventKind.PlayerDamaged"/> / <see cref="GameEventKind.PlayerHealed"/>), reason code (<see cref="GameEventKind.PlayerNotHealed"/>), or 0 (<see cref="GameEventKind.DoorOpened"/>).</param>
    public readonly record struct GameEvent(GameEventKind Kind, uint Row, uint Column, uint Payload);

    /// <summary>One uncollected key cell along with its stable id — see <see cref="MazeGame.Keys"/>.</summary>
    /// <param name="Row">Row of the key cell.</param>
    /// <param name="Column">Column of the key cell.</param>
    /// <param name="Id">Stable identifier derived from the key's origin cell.</param>
    public readonly record struct KeyInfo(uint Row, uint Column, uint Id);

    /// <summary>One enemy's current cell, stable id, and resolved per-enemy characteristics — see <see cref="MazeGame.Enemies"/>.</summary>
    /// <param name="Row">Current row of the enemy.</param>
    /// <param name="Column">Current column of the enemy.</param>
    /// <param name="Id">Stable identifier assigned at construction in row-major scan order of the <c>'E'</c> cells.</param>
    /// <param name="Damage">Damage dealt per same-cell collision (resolved: per-cell override else the per-game default).</param>
    /// <param name="MovePeriodMs">Milliseconds between one-cell moves (resolved: per-cell override else the per-game default).</param>
    /// <param name="EnemyType">Per-cell visual-rig override, or <c>null</c> when the spawn cell set none (the renderer uses its default rig).</param>
    public readonly record struct EnemyInfo(uint Row, uint Column, uint Id, uint Damage, float MovePeriodMs, EnemyType? EnemyType);

    /// <summary>One uncollected health-pickup cell — see <see cref="MazeGame.HealthPickups"/>. The cell coordinate is the natural key (pickups have no stable id).</summary>
    /// <param name="Row">Row of the health-pickup cell.</param>
    /// <param name="Column">Column of the health-pickup cell.</param>
    public readonly record struct HealthPickupInfo(uint Row, uint Column);

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

        /// <summary>Whether the game is in a lost state. See <see cref="LoseReason"/> for the cause.</summary>
        public bool IsLost => Interop.MazeGameIsLost(_gamePtr) != 0;

        /// <summary>Why the game is lost, or <see cref="Api.LoseReason.None"/> if the game is not lost.</summary>
        public LoseReason LoseReason => (LoseReason)Interop.MazeGameLoseReason(_gamePtr);

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

        /// <summary>
        /// Attempts to pick up a collectible at the player's current cell.
        /// On success the item is added to <see cref="Bag"/> and the cell is cleared.
        /// </summary>
        /// <returns>The collected item, or <c>null</c> if the player's cell holds no collectible.</returns>
        public BagItem? Pickup()
        {
            if (Interop.MazeGamePickup(_gamePtr, out var item))
                return new BagItem((BagItemKind)item.Kind, item.Id);
            return null;
        }

        /// <summary>All items currently in the player's bag, in pickup order.</summary>
        public IReadOnlyList<BagItem> Bag
        {
            get
            {
                int count = Interop.MazeGameBagCount(_gamePtr);
                var items = new List<BagItem>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetBagItem(_gamePtr, i, out var item))
                        items.Add(new BagItem((BagItemKind)item.Kind, item.Id));
                }
                return items;
            }
        }

        /// <summary>All door cells along with their current state, sorted by (row, column).</summary>
        public IReadOnlyList<DoorInfo> Doors
        {
            get
            {
                int count = Interop.MazeGameDoorCount(_gamePtr);
                var doors = new List<DoorInfo>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetDoor(_gamePtr, i, out var d))
                        doors.Add(new DoorInfo(d.Row, d.Column, (DoorState)d.State));
                }
                return doors;
            }
        }

        /// <summary>
        /// Advances time-based game state by <paramref name="dtMs"/> milliseconds and returns the events
        /// produced by this tick (e.g. doors that finished opening).
        /// </summary>
        /// <param name="dtMs">Elapsed time in milliseconds.</param>
        /// <returns>The events produced by this tick. Empty when nothing time-based is in flight.</returns>
        public GameEvent[] Tick(double dtMs)
        {
            int count = Interop.MazeGameTick(_gamePtr, (float)dtMs);
            var events = new GameEvent[count];
            for (int i = 0; i < count; i++)
            {
                if (Interop.MazeGameGetTickEvent(_gamePtr, i, out var e))
                    events[i] = new GameEvent((GameEventKind)e.Kind, e.Row, e.Column, e.Payload);
            }
            return events;
        }

        /// <summary>All uncollected key cells along with their stable ids, sorted by (row, column).
        /// Shrinks as the player picks keys up — collected keys move into <see cref="Bag"/>.</summary>
        public IReadOnlyList<KeyInfo> Keys
        {
            get
            {
                int count = Interop.MazeGameKeyCount(_gamePtr);
                var keys = new List<KeyInfo>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetKey(_gamePtr, i, out var k))
                        keys.Add(new KeyInfo(k.Row, k.Column, k.Id));
                }
                return keys;
            }
        }

        /// <summary>The player's current HP.</summary>
        public uint Hp => Interop.MazeGameHp(_gamePtr);

        /// <summary>The player's maximum HP.</summary>
        public uint MaxHp => Interop.MazeGameMaxHp(_gamePtr);

        /// <summary>All active enemies along with their current cells and stable ids.</summary>
        public IReadOnlyList<EnemyInfo> Enemies
        {
            get
            {
                int count = Interop.MazeGameEnemyCount(_gamePtr);
                var enemies = new List<EnemyInfo>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetEnemy(_gamePtr, i, out var e))
                    {
                        EnemyType? rig = e.EnemyType switch
                        {
                            0 => EnemyType.Goblin,
                            1 => EnemyType.Ghost,
                            _ => null,
                        };
                        enemies.Add(new EnemyInfo(e.Row, e.Column, e.Id, e.Damage, e.MovePeriodMs, rig));
                    }
                }
                return enemies;
            }
        }

        /// <summary>All uncollected health-pickup cells, sorted by (row, column).
        /// Shrinks as the player walks over pickups below max HP.</summary>
        public IReadOnlyList<HealthPickupInfo> HealthPickups
        {
            get
            {
                int count = Interop.MazeGameHealthPickupCount(_gamePtr);
                var pickups = new List<HealthPickupInfo>(count);
                for (int i = 0; i < count; i++)
                {
                    if (Interop.MazeGameGetHealthPickup(_gamePtr, i, out var p))
                        pickups.Add(new HealthPickupInfo(p.Row, p.Column));
                }
                return pickups;
            }
        }
    }
}
