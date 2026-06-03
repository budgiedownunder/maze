namespace Maze.Interop
{
    using System.Reflection;
    using System.Runtime.InteropServices;
    using Microsoft.Extensions.Configuration;
    using System.IO;

    /// <summary>
    ///  This class provides C# interop to the <c>maze_c</c> library and <c>maze_wasm</c> WebAssembly module,
    ///  insulating the calling application from the specifics of the underlying interop operations.
    ///
    /// Developers can use <see cref="NewMaze()">NewMaze()</see> to create
    ///  a pointer to a maze object and then other <c>Maze</c> functions, such as
    ///  <see cref="MazeInsertRows(UIntPtr,uint,uint)">MazeInsertRows()</see>,
    ///  <see cref="MazeGenerate(UIntPtr,UIntPtr)">MazeGenerate()</see>, and
    ///  <see cref="MazeSolve(UIntPtr)">MazeSolve()</see>, to interact with the maze.
    ///
    /// Once finished with, a maze should be destroyed using <see cref="FreeMaze(UIntPtr)">FreeMaze()</see>
    /// to prevent memory leaks.
    /// </summary>
    public class MazeInterop : IDisposable
    {
        const string DEFAULT_WEBASSEMBLY_NAME = "maze_wasm.wasm";

        /// <summary>
        /// Represents a type of WebAssembly interop connection technology
        /// </summary>
        public enum ConnectionType
        {
            /// <summary>
            /// The [Wasmtime](https://docs.wasmtime.dev/) WebAssembly runtime
            /// </summary>
            Wasmtime = 1,
            /// <summary>
            /// The [Wasmer](https://wasmer.io/) WebAssembly runtime
            /// </summary>
            Wasmer = 2,
            /// <summary>
            /// Native static library (`maze_c`) — no WebAssembly runtime required.
            /// Uses P/Invoke into the statically-linked library e.g. `libmaze_c.a`.
            /// </summary>
            Native = 3
        }
        // Singleton instance
        private static MazeInterop? instance = null;
        private bool _disposed = false;

        private readonly IMazeConnector connector;

        [StructLayout(LayoutKind.Sequential)]
        internal struct MazeWasmError
        {
            public UInt32 message_ptr;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct MazeWasmResult
        {
            public byte value_type;
            public UInt32 value_ptr;
            public UInt32 error_ptr;
        }

        internal enum MazeWasmResultValueType
        {
            None = 0,
            String = 1,
            Enum = 2,
            Point = 3,
            Solution = 4
        }

        /// <summary>
        /// Represents a point within a maze
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct MazePoint
        {
            /// <summary>
            /// Row index associated with the point (zero-based)
            /// </summary>
            /// <returns>Row index (zero-based)</returns>
            public UInt32 row;
            /// <summary>
            /// Column index associated with the point (zero-based)
            /// </summary>
            /// <returns>Column index (zero-based)</returns>
            public UInt32 col;
        }
        /// <summary>
        /// Identifies the maze generation algorithm to use.
        /// Mirrors the Rust <c>GenerationAlgorithmWasm</c> repr(C) enum.
        /// </summary>
        public enum MazeGenerationAlgorithm : byte
        {
            /// <summary>
            /// Generates a perfect maze using a single-pass iterative depth-first search from the start cell.
            /// See <see href="https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_depth-first_search">Randomized depth-first search</see>.
            /// </summary>
            RecursiveBacktracking = 0
        }
        /// <summary>
        /// Defines the type of a maze cell
        /// </summary>
        public enum MazeCellType
        {
            /// <summary>
            /// An empty cell
            /// </summary>
            Empty = 0,
            /// <summary>
            /// A starting cell within the maze
            /// </summary>
            Start = 1,
            /// <summary>
            ///  A finishing cell within the maze
            /// </summary>
            Finish = 2,
            /// <summary>
            /// A cell containing a wall, meaning it can't be passed through
            /// </summary>
            Wall = 3,
            /// <summary>
            /// A cell containing a key that can be picked up at gameplay time
            /// </summary>
            Key = 4,
            /// <summary>
            /// A cell containing a door that blocks passage until unlocked by a key
            /// </summary>
            Door = 5,
            /// <summary>
            /// A cell where an enemy spawns at gameplay time
            /// </summary>
            Enemy = 6,
            /// <summary>
            /// A cell containing a health pickup that restores HP when walked over
            /// </summary>
            Health = 7,
        }
        /// <summary>
        /// Reason a game ended in a loss. Mirrors the Rust `maze::LoseReason` enum.
        /// </summary>
        public enum MazeLoseReason
        {
            /// <summary>
            /// The game is not lost
            /// </summary>
            None = 0,
            /// <summary>
            /// The player can no longer hold enough keys to open every closed
            /// door remaining on a route from their current cell to the finish
            /// </summary>
            Stranded = 1,
            /// <summary>
            /// The player's HP reached zero from enemy collisions
            /// </summary>
            Killed = 2,
        }
        /// <summary>
        /// Kind of item carried in the player's bag. Mirrors the Rust
        /// `maze::BagItem` tagged enum. New item kinds extend the integer space.
        /// </summary>
        public enum MazeBagItemKind : uint
        {
            /// <summary>A key that can open one door</summary>
            Key = 0,
        }
        /// <summary>
        /// One item in the player's bag.
        /// </summary>
        public struct MazeBagItem
        {
            /// <summary>The kind of item</summary>
            public MazeBagItemKind Kind;
            /// <summary>Stable identifier for the item (e.g. derived from the key's origin cell)</summary>
            public uint Id;
        }
        /// <summary>
        /// Lifecycle state of a door cell. Mirrors the Rust `maze::DoorState` enum.
        /// </summary>
        public enum MazeDoorState : uint
        {
            /// <summary>Closed and locked; requires a key to open</summary>
            Locked = 0,
            /// <summary>Currently opening; will transition to <see cref="Open"/> on the next sufficient tick</summary>
            Opening = 1,
            /// <summary>Fully open and permanently passable</summary>
            Open = 2,
        }
        /// <summary>
        /// One door cell along with its current state.
        /// </summary>
        public struct MazeDoor
        {
            /// <summary>Row of the door cell</summary>
            public uint Row;
            /// <summary>Column of the door cell</summary>
            public uint Column;
            /// <summary>Current state of the door</summary>
            public MazeDoorState State;
        }
        /// <summary>
        /// Kind of event produced by <see cref="MazeInterop.MazeGameTick(UIntPtr, float)">MazeGameTick()</see>.
        /// Mirrors the Rust `maze::GameEvent` tagged enum.
        /// </summary>
        public enum MazeGameEventKind : uint
        {
            /// <summary>A door finished opening — its <see cref="MazeDoorState"/> is now <see cref="MazeDoorState.Open"/>. <see cref="MazeGameEvent.Row"/> / <see cref="MazeGameEvent.Column"/> is the door cell.</summary>
            DoorOpened = 0,
            /// <summary>An enemy advanced one cell. <see cref="MazeGameEvent.Row"/> / <see cref="MazeGameEvent.Column"/> is its new cell; <see cref="MazeGameEvent.Payload"/> is the enemy id.</summary>
            EnemyMoved = 1,
            /// <summary>The player took same-cell collision damage. <see cref="MazeGameEvent.Payload"/> is the player's HP after the hit; the cell fields are unused.</summary>
            PlayerDamaged = 2,
            /// <summary>The player consumed a health pickup. <see cref="MazeGameEvent.Row"/> / <see cref="MazeGameEvent.Column"/> is the consumed cell; <see cref="MazeGameEvent.Payload"/> is the player's HP after the heal.</summary>
            PlayerHealed = 3,
            /// <summary>The player walked onto a health pickup that did not apply (already at max HP). The cell is spared; <see cref="MazeGameEvent.Payload"/> is the machine-readable reason code (0 = already at max HP).</summary>
            PlayerNotHealed = 4,
            /// <summary>The player walked onto a key and it was auto-collected into the bag. <see cref="MazeGameEvent.Row"/> / <see cref="MazeGameEvent.Column"/> is the consumed key cell; <see cref="MazeGameEvent.Payload"/> is the collected key id.</summary>
            KeyCollected = 5,
        }
        /// <summary>
        /// One time-based game event emitted by a tick.
        /// </summary>
        public struct MazeGameEvent
        {
            /// <summary>The kind of event</summary>
            public MazeGameEventKind Kind;
            /// <summary>Row of the cell the event applies to (unused for <see cref="MazeGameEventKind.PlayerDamaged"/>)</summary>
            public uint Row;
            /// <summary>Column of the cell the event applies to (unused for <see cref="MazeGameEventKind.PlayerDamaged"/>)</summary>
            public uint Column;
            /// <summary>
            /// Event-specific scalar payload: enemy id (<see cref="MazeGameEventKind.EnemyMoved"/>),
            /// HP-after (<see cref="MazeGameEventKind.PlayerDamaged"/> / <see cref="MazeGameEventKind.PlayerHealed"/>),
            /// reason code (<see cref="MazeGameEventKind.PlayerNotHealed"/>), or <c>0</c> (<see cref="MazeGameEventKind.DoorOpened"/>).
            /// </summary>
            public uint Payload;
        }
        /// <summary>
        /// One uncollected key cell along with its stable id.
        /// </summary>
        public struct MazeKey
        {
            /// <summary>Row of the key cell</summary>
            public uint Row;
            /// <summary>Column of the key cell</summary>
            public uint Column;
            /// <summary>Stable identifier derived from the key's origin cell</summary>
            public uint Id;
        }
        /// <summary>
        /// One enemy's current cell along with its stable id.
        /// </summary>
        public struct MazeEnemy
        {
            /// <summary>Current row of the enemy</summary>
            public uint Row;
            /// <summary>Current column of the enemy</summary>
            public uint Column;
            /// <summary>Stable identifier assigned at construction in row-major scan order of the `'E'` cells</summary>
            public uint Id;
            /// <summary>Damage dealt per same-cell collision (resolved: per-cell override else the per-game default)</summary>
            public uint Damage;
            /// <summary>Milliseconds between one-cell moves (resolved: per-cell override else the per-game default)</summary>
            public float MovePeriodMs;
            /// <summary>Visual-rig override ordinal: <c>-1</c> = none (renderer default), <c>0</c> = goblin, <c>1</c> = ghost</summary>
            public int EnemyType;
        }
        /// <summary>
        /// One uncollected health-pickup cell. <see cref="Id"/> is always <c>0</c> —
        /// pickups have no stable id; the cell coordinate is the natural key. The
        /// field is kept for shape parity with <see cref="MazeKey"/> / <see cref="MazeEnemy"/>.
        /// </summary>
        public struct MazeHealthPickup
        {
            /// <summary>Row of the health-pickup cell</summary>
            public uint Row;
            /// <summary>Column of the health-pickup cell</summary>
            public uint Column;
            /// <summary>Always <c>0</c> — pickups have no stable id</summary>
            public uint Id;
        }
        /// <summary>
        /// Private constructor (singleton pattern)
        /// </summary>
        /// <param name="wasmPathOrName">WebAssembly path or name. WebAssembly is loaded from this location if `wasmBytes` is `null`.</param>
        /// <param name="connectionType">Type of WebAssembly connection technology to use</param>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then an attempt is made to load WebAssembly from the default location.</param>
        private MazeInterop(string wasmPathOrName, ConnectionType connectionType = ConnectionType.Wasmtime, byte[]? wasmBytes = null)
        {
            switch (connectionType)
            {
#if !IOS && !ANDROID
                case ConnectionType.Wasmtime:
                    connector = new MazeWasmtimeConnector(wasmPathOrName, wasmBytes);
                    break;
#endif
#if !IOS
                case ConnectionType.Wasmer:
                    connector = new MazeWasmerConnector(wasmPathOrName, wasmBytes);
                    break;
#endif
#if IOS
                case ConnectionType.Native:
                    connector = new MazeNativeConnector();
                    break;
#endif
                default:
                    throw new InvalidOperationException($"Unsupported connection type: {connectionType}");
            }
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~MazeInterop()
        {
            Dispose(false);
        }
        /// <summary>
        /// Handles object disposal, releasing managed and unmanaged resources and marking
        /// the object as having been finalized
        /// </summary>
        /// <returns>Nothing</returns>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }
        /// <summary>
        /// Handles object disposal
        /// </summary>
        /// <param name="disposing">Flag indicating whether the object should be fully disposed (ie. including managed
        /// as well as unmanaged  resources)</param>
        /// <returns>Nothing</returns>
        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                connector?.Dispose();
                _disposed = true;
            }
        }
        /// <summary>
        /// Returns the path to the `maze_wasm` Web Assembly
        /// </summary>
        /// <returns>Web Assembly path</returns>
        static public string GetWasmPath()
        {
            // Console.WriteLine("Current Directory: " + Environment.CurrentDirectory);

            const string WASM_FILE_NAME = "maze_wasm.wasm";
            const string APP_SETTINGS_FILE_NAME = "appsettings.json";

            // Check app settings first (if they exist)
            var executionPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
            if (string.IsNullOrEmpty(executionPath))
            {
                throw new InvalidOperationException("Could not determine execution directory");
            }
            string appsettingsFile = Path.Combine(executionPath, APP_SETTINGS_FILE_NAME);
            if (File.Exists(appsettingsFile))
            {
                var configuration = new ConfigurationBuilder()
                .SetBasePath(executionPath)
                .AddJsonFile(APP_SETTINGS_FILE_NAME)
                .AddEnvironmentVariables()
                .Build();

                string? path = configuration["MAZE_WASM_PATH"];
                if (!string.IsNullOrEmpty(path) && File.Exists(path))
                {
                    return path;
                }
            }

            // Default to execution path
            string wasmExecutionFile = Path.Combine(executionPath, WASM_FILE_NAME);
            if (!File.Exists(wasmExecutionFile))
            {
                throw new InvalidOperationException($"Web assembly file '{WASM_FILE_NAME}' not found at default path ${wasmExecutionFile}");
            }

            return wasmExecutionFile;
        }
        /// <summary>
        /// Returns the path or name to the `maze_wasm` Web Assembly to use
        /// </summary>
        /// <param name="returnDefaultName">Flag indicating whether to return the default name without determing the
        /// physical path and verifying its existence.</param>
        /// <returns>Web Assembly path</returns>
        static private string GetWasmPathOrName(bool returnDefaultName)
        {
            return returnDefaultName ? DEFAULT_WEBASSEMBLY_NAME : GetWasmPath();
        }
        /// <summary>
        /// Returns the instance for the interop (creating if needed)
        /// </summary>
        /// <param name="connectionType">Type of connection technology to use</param>
        /// <param name="createNew">Create a new instance even if a global one already exists</param>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then an attempt is made to load WebAssembly from the default location.</param>
        /// <returns>Interop instance</returns>
        static public MazeInterop GetInstance(ConnectionType connectionType = ConnectionType.Wasmtime,
            bool createNew = false, byte[]? wasmBytes = null)
        {
            if (instance is null || createNew)
            {
                bool useDefaultName = wasmBytes is not null;
                MazeInterop newInstance = new MazeInterop(GetWasmPathOrName(useDefaultName), connectionType, wasmBytes);
                if (instance is not null)
                    return newInstance;
                instance = newInstance;
            }
            return instance;
        }
        /// <summary>
        /// Initializes the interop instance if needed
        /// </summary>
        /// <param name="connectionType">Type of WebAssembly connection technology to use</param>
        /// <param name="createNew">Create a new instance, even if a global one already exists (overwriting existing)</param>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then an attempt is made to load WebAssembly from the default location.</param>
        /// <returns>Interop instance</returns>
        static public void Initialize(ConnectionType connectionType = ConnectionType.Wasmtime,
            bool createNew = false, byte[]? wasmBytes = null)
        {
            if (instance is null || createNew)
            {
                bool useDefaultName = wasmBytes is not null;
                instance = new MazeInterop(GetWasmPathOrName(useDefaultName), connectionType, wasmBytes);
            }
        }
        /// <summary>
        /// Disconnects the WebAssembly connector
        /// </summary>
        static public void Disconnect()
        {
            if (instance is null)
                return;
            instance.Dispose();
            instance = null;
        }

        /// <summary>
        /// Creates a new, empty maze, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the maze, which should later be freed by calling <see cref="FreeMaze(UIntPtr)">FreeMaze()</see></returns>
        public UIntPtr NewMaze()
        {
            return connector.NewMaze();
        }
        /// <summary>
        /// Frees a maze pointer
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void FreeMaze(UIntPtr mazePtr)
        {
            connector.FreeMaze(mazePtr);
        }
        /// <summary>
        /// Tests whether a maze is empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeIsEmpty(UIntPtr mazePtr)
        {
            return connector.MazeIsEmpty(mazePtr);
        }
        /// <summary>
        /// Resizes a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeResize(UIntPtr mazePtr, UInt32 newRowCount, UInt32 newColCount)
        {
            connector.MazeResize(mazePtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a maze to empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeReset(UIntPtr mazePtr)
        {
            connector.MazeReset(mazePtr);
        }
        /// <summary>
        /// Gets the row count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeGetRowCount(UIntPtr mazePtr)
        {
            return connector.MazeGetRowCount(mazePtr);
        }
        /// <summary>
        /// Gets the column count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeGetColCount(UIntPtr mazePtr)
        {
            return connector.MazeGetColCount(mazePtr);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within a maze, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeCellType MazeGetCellType(UIntPtr mazePtr, UInt32 row, UInt32 col)
        {
            return connector.MazeGetCellType(mazePtr, row, col);
        }
        /// <summary>
        /// Sets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void MazeSetStartCell(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol)
        {
            connector.MazeSetStartCell(mazePtr, startRow, startCol);
        }
        /// <summary>
        /// Gets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazePoint MazeGetStartCell(UIntPtr mazePtr)
        {
            return connector.MazeGetStartCell(mazePtr);
        }
        /// <summary>
        /// Sets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void MazeSetFinishCell(UIntPtr mazePtr, UInt32 finishRow, UInt32 finishCol)
        {
            connector.MazeSetFinishCell(mazePtr, finishRow, finishCol);
        }
        /// <summary>
        /// Gets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazePoint MazeGetFinishCell(UIntPtr mazePtr)
        {
            return connector.MazeGetFinishCell(mazePtr);
        }
        /// <summary>
        /// Sets a range of cells to walls within a maze, or will throw an exception
        /// if the walls cannot be set
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="endRow">Target end row</param>
        /// <param name="endCol">Target end column</param>
        /// <returns>Nothing</returns>
        public void MazeSetWallCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeSetWallCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Sets a range of cells in a maze to keys, or throws an
        /// exception if the cells cannot be set.
        /// </summary>
        public void MazeSetKeyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeSetKeyCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Sets a range of cells in a maze to doors, or throws an
        /// exception if the cells cannot be set.
        /// </summary>
        public void MazeSetDoorCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeSetDoorCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Sets a range of cells in a maze to enemy spawns, or throws an
        /// exception if the cells cannot be set.
        /// </summary>
        public void MazeSetEnemyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeSetEnemyCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Sets a range of cells in a maze to health pickups, or throws an
        /// exception if the cells cannot be set.
        /// </summary>
        public void MazeSetHealthCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeSetHealthCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Clears a range of wall cells within a maze, or will throw an exception
        /// if the cells cannot be cleared
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="endRow">Target end row</param>
        /// <param name="endCol">Target end column</param>
        /// <returns>Nothing</returns>
        public void MazeClearCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeClearCells(mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Inserts rows into a maze, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void MazeInsertRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            connector.MazeInsertRows(mazePtr, startRow, count);
        }
        /// <summary>
        /// Deletes rows from a maze, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void MazeDeleteRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            connector.MazeDeleteRows(mazePtr, startRow, count);
        }
        /// <summary>
        /// Inserts columns into a maze, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void MazeInsertCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            connector.MazeInsertCols(mazePtr, startCol, count);
        }
        /// <summary>
        /// Deletes columns from a maze, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void MazeDeleteCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            connector.MazeDeleteCols(mazePtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeFromJson(UIntPtr mazePtr, string json)
        {
            connector.MazeFromJson(mazePtr, json);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeToJson(UIntPtr mazePtr)
        {
            return connector.MazeToJson(mazePtr);
        }
        /// <summary>Returns the per-cell entity override at the given location as its wire JSON, or <c>null</c> when the cell carries none.</summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="col">Column index (zero-based)</param>
        /// <returns>The entity wire JSON, or <c>null</c> when the cell has no override</returns>
        public string? MazeGetCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            return connector.MazeGetCellEntity(mazePtr, row, col);
        }
        /// <summary>Sets the per-cell entity override at the given location from its wire JSON, replacing any existing one.</summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="col">Column index (zero-based)</param>
        /// <param name="json">The entity override wire JSON</param>
        public void MazeSetCellEntity(UIntPtr mazePtr, uint row, uint col, string json)
        {
            connector.MazeSetCellEntity(mazePtr, row, col, json);
        }
        /// <summary>Clears any per-cell entity override at the given location.</summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="col">Column index (zero-based)</param>
        public void MazeClearCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            connector.MazeClearCellEntity(mazePtr, row, col);
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails.
        ///
        /// If successful, use <see cref="MazeSolutionGetPathPoints(UIntPtr)">MazeSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeSolution(UIntPtr)">FreeMazeSolution()</see></returns>
        public UIntPtr MazeSolve(UIntPtr mazePtr)
        {
            return connector.MazeSolve(mazePtr);
        }
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            return connector.MazeSolutionGetPathPoints(solutionPtr);
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeSolution(UIntPtr solutionPtr)
        {
            connector.FreeMazeSolution(solutionPtr);
        }
        /// <summary>
        /// Allocates a sized memory block of a given size. A sized memory block is a block of
        /// memory of (`size` + 4) bytes, where the first 4 bytes contain the size of the block (u32)
        /// and then the next `size` bytes is reserved for data use.
        /// </summary>
        /// <param name="size">Number of bytes to allocate</param>
        /// <returns>Pointer to memory</returns>
        public UInt32 AllocateSizedMemory(UInt32 size)
        {
            return connector.AllocateSizedMemory(size);
        }
        /// <summary>
        /// Frees the sized memory associated with a given pointer
        /// </summary>
        /// <param name="ptr">Pointer to memory</param>
        /// <returns>Nothing</returns>
        public void FreeSizedMemory(UInt32 ptr)
        {
            connector.FreeSizedMemory(ptr);
        }
        /// <summary>
        /// Gets the amount of sized memory currenty allocated
        /// </summary>
        /// <returns>Memory used count</returns>
        public Int64 GetSizedMemoryUsed()
        {
            return connector.GetSizedMemoryUsed();
        }
        /// <summary>
        /// Gets the number of objects currenty allocated
        /// </summary>
        /// <returns>Object count</returns>
        public Int64 GetNumObjectsAllocated()
        {
            return connector.GetNumObjectsAllocated();
        }
        /// <summary>
        /// Creates a new <c>GeneratorOptions</c>, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="rowCount">Number of rows to generate</param>
        /// <param name="colCount">Number of columns to generate</param>
        /// <param name="algorithm">Generation algorithm</param>
        /// <param name="seed">Random number generator seed for deterministic generation</param>
        /// <returns>Pointer to the <c>GeneratorOptions</c>, which should later be freed by calling <see cref="FreeGeneratorOptions(UIntPtr)">FreeGeneratorOptions()</see></returns>
        public UIntPtr NewGeneratorOptions(UInt32 rowCount, UInt32 colCount, MazeGenerationAlgorithm algorithm, UInt64 seed)
        {
            return connector.NewGeneratorOptions(rowCount, colCount, algorithm, seed);
        }
        /// <summary>
        /// Sets the start cell on a <c>GeneratorOptions</c>
        /// </summary>
        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            connector.GeneratorOptionsSetStart(optionsPtr, row, col);
        }
        /// <summary>
        /// Sets the finish cell on a <c>GeneratorOptions</c>
        /// </summary>
        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            connector.GeneratorOptionsSetFinish(optionsPtr, row, col);
        }
        /// <summary>
        /// Sets the minimum spine length on a <c>GeneratorOptions</c>
        /// </summary>
        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetMinSpineLength(optionsPtr, value);
        }
        /// <summary>
        /// Sets the maximum retries on a <c>GeneratorOptions</c>
        /// </summary>
        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetMaxRetries(optionsPtr, value);
        }
        /// <summary>
        /// Sets the branch_from_finish flag on a <c>GeneratorOptions</c> (0 = false, 1 = true)
        /// </summary>
        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value)
        {
            connector.GeneratorOptionsSetBranchFromFinish(optionsPtr, value);
        }
        /// <summary>
        /// Sets the door_count on a <c>GeneratorOptions</c> (0 = none, the default).
        /// Real path doors auto-placed on the spine by the Rust generator; each
        /// real door contributes one key and one door cell to the
        /// produced grid, so the joint cap is <c>2 * door_count + spare_doors +
        /// spare_keys &lt;= MAX_TOTAL_FEATURES (16)</c>.
        /// </summary>
        public void GeneratorOptionsSetDoorCount(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetDoorCount(optionsPtr, value);
        }
        /// <summary>
        /// Sets the spare_doors on a <c>GeneratorOptions</c> (0 = none, the
        /// default). Decoy doors planted on off-spine branches.
        /// </summary>
        public void GeneratorOptionsSetSpareDoors(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetSpareDoors(optionsPtr, value);
        }
        /// <summary>
        /// Sets the spare_keys on a <c>GeneratorOptions</c> (0 = none, the
        /// default). Spare keys planted on off-spine branches, giving the
        /// player a budget to burn on decoys before they risk stranding.
        /// </summary>
        public void GeneratorOptionsSetSpareKeys(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetSpareKeys(optionsPtr, value);
        }
        /// <summary>
        /// Sets the enemy_count on a <c>GeneratorOptions</c> (0 = none, the
        /// default). Enemies auto-placed at random passable cells, clamped by
        /// the generator to its enemy ceiling and the eligible-cell count.
        /// </summary>
        public void GeneratorOptionsSetEnemyCount(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetEnemyCount(optionsPtr, value);
        }
        /// <summary>
        /// Sets the health_count on a <c>GeneratorOptions</c> (0 = none, the
        /// default). Health pickups auto-placed at random passable cells,
        /// clamped by the generator to its health ceiling and the eligible-cell
        /// count.
        /// </summary>
        public void GeneratorOptionsSetHealthCount(UIntPtr optionsPtr, UInt32 value)
        {
            connector.GeneratorOptionsSetHealthCount(optionsPtr, value);
        }
        /// <summary>
        /// Generates a maze, populating the given maze, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="optionsPtr">Pointer to <c>GeneratorOptions</c></param>
        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            connector.MazeGenerate(mazePtr, optionsPtr);
        }
        /// <summary>
        /// Frees a <c>GeneratorOptions</c> pointer
        /// </summary>
        /// <param name="optionsPtr">Pointer to <c>GeneratorOptions</c></param>
        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            connector.FreeGeneratorOptions(optionsPtr);
        }
        /// <summary>
        /// Creates a new maze game session from a maze definition JSON string,
        /// or will throw an exception if the operation fails.
        /// </summary>
        /// <param name="definitionJson">Maze definition JSON string ({"grid":[...]})</param>
        /// <returns>Opaque game session pointer. Free with <see cref="FreeMazeGame(UIntPtr)">FreeMazeGame()</see> when done.</returns>
        public UIntPtr NewMazeGame(string definitionJson)
        {
            return connector.NewMazeGame(definitionJson);
        }
        /// <summary>
        /// Frees a game session pointer returned by <see cref="NewMazeGame(string)">NewMazeGame()</see>
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        public void FreeMazeGame(UIntPtr gamePtr)
        {
            connector.FreeMazeGame(gamePtr);
        }
        /// <summary>
        /// Moves the player one cell in the given direction
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="dir">Direction: 0=None 1=Up 2=Down 3=Left 4=Right</param>
        /// <returns>0=None 1=Moved 2=Blocked 3=Complete 4=BlockedByLockedDoor 5=StartedUnlocking 6=Stranded</returns>
        public int MazeGameMovePlayer(UIntPtr gamePtr, int dir)
        {
            return connector.MazeGameMovePlayer(gamePtr, dir);
        }
        /// <summary>
        /// Gets the player's current row (zero-based)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Row index</returns>
        public int MazeGamePlayerRow(UIntPtr gamePtr)
        {
            return connector.MazeGamePlayerRow(gamePtr);
        }
        /// <summary>
        /// Gets the player's current column (zero-based)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Column index</returns>
        public int MazeGamePlayerCol(UIntPtr gamePtr)
        {
            return connector.MazeGamePlayerCol(gamePtr);
        }
        /// <summary>
        /// Gets the player's current facing direction
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>0=None 1=Up 2=Down 3=Left 4=Right</returns>
        public int MazeGamePlayerDirection(UIntPtr gamePtr)
        {
            return connector.MazeGamePlayerDirection(gamePtr);
        }
        /// <summary>
        /// Returns whether the player has reached the finish cell
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>1 if complete, 0 otherwise</returns>
        public int MazeGameIsComplete(UIntPtr gamePtr)
        {
            return connector.MazeGameIsComplete(gamePtr);
        }
        /// <summary>
        /// Returns whether the game is in a lost state
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>1 if lost, 0 otherwise</returns>
        public int MazeGameIsLost(UIntPtr gamePtr)
        {
            return connector.MazeGameIsLost(gamePtr);
        }
        /// <summary>
        /// Returns the lose-reason code for the game session
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>0=None 1=Stranded</returns>
        public int MazeGameLoseReason(UIntPtr gamePtr)
        {
            return connector.MazeGameLoseReason(gamePtr);
        }
        /// <summary>
        /// Attempts to pick up a collectible at the player's current cell.
        /// Adds the item to the bag and clears the cell on success.
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="item">Receives the picked item on success</param>
        /// <returns>True if an item was picked up; false if the player's cell holds no collectible</returns>
        public bool MazeGamePickup(UIntPtr gamePtr, out MazeBagItem item)
        {
            return connector.MazeGamePickup(gamePtr, out item);
        }
        /// <summary>
        /// Returns the number of items currently in the player's bag
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Bag size</returns>
        public int MazeGameBagCount(UIntPtr gamePtr)
        {
            return connector.MazeGameBagCount(gamePtr);
        }
        /// <summary>
        /// Retrieves a single bag item by index
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the bag</param>
        /// <param name="item">Receives the bag item on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetBagItem(UIntPtr gamePtr, int index, out MazeBagItem item)
        {
            return connector.MazeGameGetBagItem(gamePtr, index, out item);
        }
        /// <summary>
        /// Returns the number of door cells in the maze, regardless of state
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Door count</returns>
        public int MazeGameDoorCount(UIntPtr gamePtr)
        {
            return connector.MazeGameDoorCount(gamePtr);
        }
        /// <summary>
        /// Retrieves a single door cell by index
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the door list</param>
        /// <param name="door">Receives the door cell + state on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetDoor(UIntPtr gamePtr, int index, out MazeDoor door)
        {
            return connector.MazeGameGetDoor(gamePtr, index, out door);
        }
        /// <summary>
        /// Advances time-based game state by <paramref name="dtMs"/> milliseconds
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="dtMs">Elapsed time in milliseconds</param>
        /// <returns>Number of events produced by this tick</returns>
        public int MazeGameTick(UIntPtr gamePtr, float dtMs)
        {
            return connector.MazeGameTick(gamePtr, dtMs);
        }
        /// <summary>
        /// Returns the number of events currently buffered from the most recent tick
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Tick event count</returns>
        public int MazeGameTickEventCount(UIntPtr gamePtr)
        {
            return connector.MazeGameTickEventCount(gamePtr);
        }
        /// <summary>
        /// Retrieves a single tick event from the buffer by index
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the tick event buffer</param>
        /// <param name="evt">Receives the tick event on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetTickEvent(UIntPtr gamePtr, int index, out MazeGameEvent evt)
        {
            return connector.MazeGameGetTickEvent(gamePtr, index, out evt);
        }
        /// <summary>
        /// Returns the number of uncollected key cells in the maze
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Uncollected key count</returns>
        public int MazeGameKeyCount(UIntPtr gamePtr)
        {
            return connector.MazeGameKeyCount(gamePtr);
        }
        /// <summary>
        /// Retrieves a single uncollected key cell by index
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the uncollected-keys list</param>
        /// <param name="key">Receives the key cell + stable id on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetKey(UIntPtr gamePtr, int index, out MazeKey key)
        {
            return connector.MazeGameGetKey(gamePtr, index, out key);
        }
        /// <summary>Returns the player's current HP</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Current HP</returns>
        public uint MazeGameHp(UIntPtr gamePtr)
        {
            return connector.MazeGameHp(gamePtr);
        }
        /// <summary>Returns the player's maximum HP</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Maximum HP</returns>
        public uint MazeGameMaxHp(UIntPtr gamePtr)
        {
            return connector.MazeGameMaxHp(gamePtr);
        }
        /// <summary>Returns the number of active enemies</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Enemy count</returns>
        public int MazeGameEnemyCount(UIntPtr gamePtr)
        {
            return connector.MazeGameEnemyCount(gamePtr);
        }
        /// <summary>Retrieves a single enemy's current cell + stable id by index</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the enemy list</param>
        /// <param name="enemy">Receives the enemy cell + id on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetEnemy(UIntPtr gamePtr, int index, out MazeEnemy enemy)
        {
            return connector.MazeGameGetEnemy(gamePtr, index, out enemy);
        }
        /// <summary>Returns the number of uncollected health-pickup cells</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Uncollected health-pickup count</returns>
        public int MazeGameHealthPickupCount(UIntPtr gamePtr)
        {
            return connector.MazeGameHealthPickupCount(gamePtr);
        }
        /// <summary>Retrieves a single uncollected health-pickup cell by index</summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the health-pickup list</param>
        /// <param name="pickup">Receives the pickup cell on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetHealthPickup(UIntPtr gamePtr, int index, out MazeHealthPickup pickup)
        {
            return connector.MazeGameGetHealthPickup(gamePtr, index, out pickup);
        }
        /// <summary>
        /// Returns the number of cells visited by the player (including the start cell)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Visited cell count</returns>
        public int MazeGameVisitedCellCount(UIntPtr gamePtr)
        {
            return connector.MazeGameVisitedCellCount(gamePtr);
        }
        /// <summary>
        /// Retrieves a visited cell by index
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="index">Zero-based index into the visited-cells list</param>
        /// <param name="row">Receives the cell row on success</param>
        /// <param name="col">Receives the cell column on success</param>
        /// <returns>True if the index was valid; false if out of range</returns>
        public bool MazeGameGetVisitedCell(UIntPtr gamePtr, int index, out int row, out int col)
        {
            return connector.MazeGameGetVisitedCell(gamePtr, index, out row, out col);
        }
    }
}
