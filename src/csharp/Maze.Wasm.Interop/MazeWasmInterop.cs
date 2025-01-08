namespace Maze.Wasm.Interop
{
    using System.Reflection;
    using System.Runtime.InteropServices;
    using Microsoft.Extensions.Configuration;
    using System.Text;
    using Wasmtime;
    using System.IO;

    /// <summary>
    ///  This class provides C# interop to the `maze_wasm` web assembly, insulating the
    ///  calling application from the specifics of the underlying Web Assembly interop operations.
    ///  
    /// Developers can use <see cref="NewMazeWasm()">NewMazeWasm()</see> to create
    ///  a pointer to a maze object within Web Assembly and then other `MazeWasm` functions, such as 
    ///  <see cref="MazeWasmInsertRows(UInt32,UInt32,UInt32)">MazeWasmInsertRows()</see> and 
    ///  <see cref="MazeWasmSolve(UInt32)">MazeWasmSolve()</see>, to interact with the maze.
    ///  
    /// Once finished with, a maze should be destroyed using <see cref="FreeMazeWasm(UInt32)">FreeMazeWasm()</see>
    /// to prevent memory leaks within Web Assembly.
    /// </summary>
    public class MazeWasmInterop : IDisposable
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
            Wasmer = 2
        }
        // Singleton instance
        private static MazeWasmInterop? instance = null;
        private bool _disposed = false;

        private IMazeWasmConnector connector;

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
        public struct MazeWasmPoint
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
        /// Defines the type of a maze cell
        /// </summary>
        public enum MazeWasmCellType
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
        private MazeWasmInterop(string wasmPathOrName, ConnectionType connectionType=ConnectionType.Wasmtime, byte[]? wasmBytes = null)
        {
            switch (connectionType)
            {
                case ConnectionType.Wasmtime:
                    connector = new MazeWasmtimeConnector(wasmPathOrName, wasmBytes);
                    break;
                case ConnectionType.Wasmer:
                    connector = new MazeWasmerConnector(wasmPathOrName, wasmBytes);
                    break;
                default:
                    throw new InvalidOperationException($"Unsupported connection type: {connectionType}");
            }
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~MazeWasmInterop()
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
                throw new InvalidOperationException($"Web assembly file '{WASM_FILE_NAME}' not found at path ${wasmExecutionFile}");
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
        /// <param name="connectionType">Type of WebAssembly connection technology to use</param>
        /// <param name="createNew">Create a new instance even if a global one already exists</param>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then at attempt is made to load the WebAssembly from the default location.(</param>
        /// <returns>Interop instance</returns>
        static public MazeWasmInterop GetInstance(ConnectionType connectionType= ConnectionType.Wasmtime, 
            bool createNew = false, byte[]? wasmBytes = null)
        {
            if (instance is null || createNew)
            {
                bool useDefaultName = wasmBytes is not null;
                MazeWasmInterop newInstance = new MazeWasmInterop(GetWasmPathOrName(useDefaultName), connectionType, wasmBytes);
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
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then at attempt is made to load the WebAssembly from the default location.(</param>
        /// <returns>Interop instance</returns>
        static public void Initialize(ConnectionType connectionType = ConnectionType.Wasmtime, 
            bool createNew = false, byte[]? wasmBytes = null)
        {
            if (instance is null || createNew) 
            {
                bool useDefaultName = wasmBytes is not null;
                instance = new MazeWasmInterop(GetWasmPathOrName(useDefaultName), connectionType, wasmBytes);
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
        /// Creates a new, empty `MazeWasm`, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the `MazeWasm`, which should later be freed by calling <see cref="FreeMazeWasm(UInt32)">FreeMazeWasm()</see></returns>
        public UInt32 NewMazeWasm()
        {
            return connector.NewMazeWasm();
        }
        /// <summary>
        /// Frees a `MazeWasm` pointer
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to `MazeWasm`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasm(UInt32 mazeWasmPtr)
        {
            connector.FreeMazeWasm(mazeWasmPtr);
        }
        /// <summary>
        /// Tests whether a `MazeWasm` is empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeWasmIsEmpty(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmIsEmpty(mazeWasmPtr);
        }
        /// <summary>
        /// Resizes a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeWasmResize(UInt32 mazeWasmPtr, UInt32 newRowCount, UInt32 newColCount)
        {
            connector.MazeWasmResize(mazeWasmPtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a `MazeWasm` to empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeWasmReset(UInt32 mazeWasmPtr)
        {
            connector.MazeWasmReset(mazeWasmPtr);
        }
        /// <summary>
        /// Gets the row count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeWasmGetRowCount(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmGetRowCount(mazeWasmPtr);
        }
        /// <summary>
        /// Gets the column count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeWasmGetColCount(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmGetColCount(mazeWasmPtr);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within a `MazeWasm`, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeWasmCellType MazeWasmGetCellType(UInt32 mazeWasmPtr, UInt32 row, UInt32 col)
        {
            return connector.MazeWasmGetCellType(mazeWasmPtr, row, col);
        }
        /// <summary>
        /// Sets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetStartCell(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol)
        {
            connector.MazeWasmSetStartCell(mazeWasmPtr, startRow, startCol);
        }
        /// <summary>
        /// Gets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeWasmPoint MazeWasmGetStartCell(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmGetStartCell(mazeWasmPtr);
        }
        /// <summary>
        /// Sets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetFinishCell(UInt32 mazeWasmPtr, UInt32 finishRow, UInt32 finishCol)
        {
            connector.MazeWasmSetFinishCell(mazeWasmPtr , finishRow , finishCol);
        }
        /// <summary>
        /// Gets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazeWasmPoint MazeWasmGetFinishCell(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmGetFinishCell(mazeWasmPtr);
        }
        /// <summary>
        /// Sets a range of cells to walls within a `MazeWasm`, or will throw an exception
        /// if the walls cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="endRow">Target end row</param>
        /// <param name="endCol">Target end column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetWallCells(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeWasmSetWallCells(mazeWasmPtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Clears a range of wall cells within a `MazeWasm`, or will throw an exception
        /// if the cells cannot be cleared
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="endRow">Target end row</param>
        /// <param name="endCol">Target end column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmClearCells(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            connector.MazeWasmClearCells(mazeWasmPtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Inserts rows into a `MazeWasm`, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertRows(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 count)
        {
            connector.MazeWasmInsertRows(mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Deletes rows from a `MazeWasm`, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteRows(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 count)
        {
            connector.MazeWasmDeleteRows(mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Inserts columns into a `MazeWasm`, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertCols(UInt32 mazeWasmPtr, UInt32 startCol, UInt32 count)
        {
            connector.MazeWasmInsertCols(mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Deletes columns from a `MazeWasm`, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteCols(UInt32 mazeWasmPtr, UInt32 startCol, UInt32 count)
        {
            connector.MazeWasmDeleteCols(mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a `MazeWasm` from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeWasmFromJson(UInt32 mazeWasmPtr, string json)
        {
            connector.MazeWasmFromJson(mazeWasmPtr, json);
        }
        /// <summary>
        /// Converts a `MazeWasm` to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeWasmToJson(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmToJson(mazeWasmPtr);
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails. 
        ///
        /// If successful, use <see cref="MazeWasmSolutionGetPathPoints(UInt32)">MazeWasmSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeWasmSolution(UInt32)">FreeMazeWasmSolution()</see></returns>
        public UInt32 MazeWasmSolve(UInt32 mazeWasmPtr)
        {
            return connector.MazeWasmSolve(mazeWasmPtr);
        }
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeWasmPoint> MazeWasmSolutionGetPathPoints(UInt32 solutionPtr)
        {
            return connector.MazeWasmSolutionGetPathPoints(solutionPtr);
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmSolution(UInt32 solutionPtr)
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
    }
}
