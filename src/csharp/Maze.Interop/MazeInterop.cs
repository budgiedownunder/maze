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

        private IMazeConnector connector;

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
            /// <summary>Two-phase recursive backtracking.</summary>
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
        }
        /// <summary>
        /// Private constructor (singleton pattern)
        /// </summary>
        /// <param name="wasmPathOrName">WebAssembly path or name. WebAssembly is loaded from this location if `wasmBytes` is `null`.</param>
        /// <param name="connectionType">Type of WebAssembly connection technology to use</param>
        /// <param name="wasmBytes">WebAssembly bytes(</param>
        private MazeInterop(string wasmPathOrName, ConnectionType connectionType=ConnectionType.Wasmtime, byte[]? wasmBytes = null)
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
                if(connector is not null)
                    connector.Dispose();
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
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then at attempt is made to load WebAssembly from the default location.(</param>
        /// <returns>Interop instance</returns>
        static public MazeInterop GetInstance(ConnectionType connectionType= ConnectionType.Wasmtime,
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
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then at attempt is made to load WebAssembly from the default location.(</param>
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
            return connector.NewMazeWasm();
        }
        /// <summary>
        /// Frees a maze pointer
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void FreeMaze(UIntPtr mazePtr)
        {
            connector.FreeMazeWasm(mazePtr);
        }
        /// <summary>
        /// Tests whether a maze is empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeIsEmpty(UIntPtr mazePtr)
        {
            return connector.MazeWasmIsEmpty(mazePtr);
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
            connector.MazeWasmResize(mazePtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a maze to empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeReset(UIntPtr mazePtr)
        {
            connector.MazeWasmReset(mazePtr);
        }
        /// <summary>
        /// Gets the row count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeGetRowCount(UIntPtr mazePtr)
        {
            return connector.MazeWasmGetRowCount(mazePtr);
        }
        /// <summary>
        /// Gets the column count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeGetColCount(UIntPtr mazePtr)
        {
            return connector.MazeWasmGetColCount(mazePtr);
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
            return connector.MazeWasmGetCellType(mazePtr, row, col);
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
            connector.MazeWasmSetStartCell(mazePtr, startRow, startCol);
        }
        /// <summary>
        /// Gets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazePoint MazeGetStartCell(UIntPtr mazePtr)
        {
            return connector.MazeWasmGetStartCell(mazePtr);
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
            connector.MazeWasmSetFinishCell(mazePtr, finishRow, finishCol);
        }
        /// <summary>
        /// Gets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazePoint MazeGetFinishCell(UIntPtr mazePtr)
        {
            return connector.MazeWasmGetFinishCell(mazePtr);
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
            connector.MazeWasmSetWallCells(mazePtr, startRow, startCol, endRow, endCol);
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
            connector.MazeWasmClearCells(mazePtr, startRow, startCol, endRow, endCol);
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
            connector.MazeWasmInsertRows(mazePtr, startRow, count);
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
            connector.MazeWasmDeleteRows(mazePtr, startRow, count);
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
            connector.MazeWasmInsertCols(mazePtr, startCol, count);
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
            connector.MazeWasmDeleteCols(mazePtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeFromJson(UIntPtr mazePtr, string json)
        {
            connector.MazeWasmFromJson(mazePtr, json);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeToJson(UIntPtr mazePtr)
        {
            return connector.MazeWasmToJson(mazePtr);
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
            return connector.MazeWasmSolve(mazePtr);
        }
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            return connector.MazeWasmSolutionGetPathPoints(solutionPtr);
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeSolution(UIntPtr solutionPtr)
        {
            connector.FreeMazeWasmSolution(solutionPtr);
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
            return connector.NewGeneratorOptionsWasm(rowCount, colCount, algorithm, seed);
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
        /// Generates a maze, populating the given maze, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="optionsPtr">Pointer to <c>GeneratorOptions</c></param>
        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            connector.MazeWasmGenerate(mazePtr, optionsPtr);
        }
        /// <summary>
        /// Frees a <c>GeneratorOptions</c> pointer
        /// </summary>
        /// <param name="optionsPtr">Pointer to <c>GeneratorOptions</c></param>
        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            connector.FreeGeneratorOptionsWasm(optionsPtr);
        }
    }
}
