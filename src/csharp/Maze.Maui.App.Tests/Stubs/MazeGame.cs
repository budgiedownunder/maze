// Stubs for the maze-game types from Maze.Api. The production MazeGame is
// sealed with a private constructor and its Create factory routes through
// the native maze_wasm/maze_c interop, which can't be loaded in a bare
// net10.0 test host. The MazeGameViewModel test surface only exercises
// guard-and-early-return paths that don't need a live game session, so a
// throw-on-Create stub is sufficient for those tests; tests that need the
// game session set the public properties directly (the view-model holds a
// reference but doesn't gate them by the Create() factory).
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
        BlockedByLockedDoor = 4,
        StartedUnlocking = 5,
        Stranded = 6,
        Killed = 7,
    }

    public enum LoseReason
    {
        None = 0,
        Stranded = 1,
        Killed = 2,
    }

    public enum BagItemKind
    {
        Key = 0,
    }

    public readonly record struct BagItem(BagItemKind Kind, uint Id);

    public enum DoorState
    {
        Locked = 0,
        Opening = 1,
        Open = 2,
    }

    public readonly record struct DoorInfo(uint Row, uint Column, DoorState State);

    public enum GameEventKind
    {
        DoorOpened = 0,
        EnemyMoved = 1,
        PlayerDamaged = 2,
        PlayerHealed = 3,
        PlayerNotHealed = 4,
    }

    public readonly record struct GameEvent(GameEventKind Kind, uint Row, uint Column, uint Payload);

    public readonly record struct KeyInfo(uint Row, uint Column, uint Id);

    public readonly record struct EnemyInfo(uint Row, uint Column, uint Id);

    public readonly record struct HealthPickupInfo(uint Row, uint Column);

    public sealed class MazeGame : IDisposable
    {
        private MazeGame() { }

        public static MazeGame Create(string definitionJson)
            => throw new NotSupportedException("Maze.Api.MazeGame is stubbed in the test host; tests must avoid the StartGame happy path.");

        // Test-only factory used by ViewModel tests to spin up a game-shaped object
        // without going through the native interop chain. The view-model only reads
        // the surface members listed below, so a settable plain instance is enough.
        public static MazeGame CreateForTests() => new MazeGame();

        public MazeGameMoveResult MovePlayer(MazeGameDirection direction)
        {
            PlayerDirection = direction;
            return NextMoveResult;
        }

        public BagItem? Pickup()
        {
            BagItem? next = NextPickupItem;
            if (next is not null)
            {
                var newBag = new List<BagItem>(Bag) { next.Value };
                Bag = newBag;
                NextPickupItem = null;
                // Remove the key at the current player position from the Keys collection.
                Keys = Keys.Where(k => !(k.Row == (uint)PlayerRow && k.Column == (uint)PlayerCol)).ToList();
            }
            return next;
        }

        public GameEvent[] Tick(double dtMs)
        {
            _ = dtMs; // Stub does not consume elapsed time — tests fire NextTickEvents directly.
            var events = NextTickEvents;
            NextTickEvents = [];
            // Apply any DoorOpened events to the Doors collection in the stub.
            if (events.Length > 0)
            {
                var updated = Doors.ToList();
                foreach (var e in events)
                {
                    if (e.Kind == GameEventKind.DoorOpened)
                    {
                        for (int i = 0; i < updated.Count; i++)
                        {
                            if (updated[i].Row == e.Row && updated[i].Column == e.Column)
                                updated[i] = new DoorInfo(e.Row, e.Column, DoorState.Open);
                        }
                    }
                }
                Doors = updated;
            }
            return events;
        }

        public int PlayerRow { get; set; }
        public int PlayerCol { get; set; }
        public MazeGameDirection PlayerDirection { get; set; }
        public bool IsComplete { get; set; }
        public bool IsLost { get; set; }
        public LoseReason LoseReason { get; set; } = LoseReason.None;

        /// <summary>Test-controlled outcome of the next <see cref="MovePlayer"/> call.</summary>
        public MazeGameMoveResult NextMoveResult { get; set; } = MazeGameMoveResult.None;

        /// <summary>Test-controlled item returned by the next <see cref="Pickup"/> call (one-shot).</summary>
        public BagItem? NextPickupItem { get; set; }

        /// <summary>Test-controlled events returned by the next <see cref="Tick"/> call (one-shot).</summary>
        public GameEvent[] NextTickEvents { get; set; } = [];

        public IReadOnlyList<BagItem> Bag { get; set; } = [];
        public IReadOnlyList<DoorInfo> Doors { get; set; } = [];
        public IReadOnlyList<KeyInfo> Keys { get; set; } = [];

        public uint Hp { get; set; }
        public uint MaxHp { get; set; }
        public IReadOnlyList<EnemyInfo> Enemies { get; set; } = [];
        public IReadOnlyList<HealthPickupInfo> HealthPickups { get; set; } = [];

        public void Dispose() { }
    }
}
