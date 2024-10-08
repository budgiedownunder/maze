namespace Maze.Wasm.Interop
{
    using System.Reflection;
    using System.Runtime.InteropServices;
    using Microsoft.Extensions.Configuration;
    using System.Text;
    using Wasmtime;

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
    public class MazeWasmInterop
    {
        // Singleton instance
        private static MazeWasmInterop? instance = null;

        private static Wasmtime.Memory? wasmMemory;

        // Wasmtime Store and Instance
        private static string? instanceWasmPath;
        private static Store? store;
        private static Instance? instanceWasm;

        // WebAssembly functions
        private static Function? newMazeWasm;
        private Function? freeMazeWasm;
        private Function? mazeWasmIsEmpty;
        private Function? mazeWasmResize;
        private Function? mazeWasmReset;
        private Function? mazeWasmGetRowCount;
        private Function? mazeWasmGetColCount;
        private Function? mazeWasmGetCellType;
        private Function? mazeWasmSetStartCell;
        private Function? mazeWasmGetStartCell;
        private Function? mazeWasmSetFinishCell;
        private Function? mazeWasmGetFinishCell;
        private Function? mazeWasmSetWallCells;
        private Function? mazeWasmClearCells;
        private Function? mazeWasmInsertRows;
        private Function? mazeWasmDeleteRows;
        private Function? mazeWasmInsertCols;
        private Function? mazeWasmDeleteCols;
        private Function? mazeWasmFromJson;
        private Function? mazeWasmToJson;
        private Function? mazeWasmSolve;
        private Function? mazeWasmSolutionGetPathPoints;
        private Function? freeMazeWasmResult;
        private Function? freeMazeWasmSolution;

        private Function? freeMazeWasmError;
        private Function? allocateSizedMemory;
        private Function? freeSizedMemory;
        private Function? getSizedMemoryUsed;
        private Function? getNumObjectsAllocated;

        [StructLayout(LayoutKind.Sequential)]
        struct MazeWasmError
        {
            public UInt32 message_ptr;
        }

        [StructLayout(LayoutKind.Sequential)]
        struct MazeWasmResult
        {
            public byte value_type;
            public UInt32 value_ptr;
            public UInt32 error_ptr;
        }

        enum MazeWasmResultValueType
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
            /// Row associated with the point (zero-based)
            /// </summary>
            public UInt32 row;
            /// <summary>
            /// Column associated with the point (zero-based)
            /// </summary>
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

        // Private constructor (singleton pattern)
        private MazeWasmInterop(string wasmPath)
        {
            var engine = new Engine();
            var module = Wasmtime.Module.FromFile(engine, wasmPath);
            using var linker = new Linker(engine);
            store = new Store(engine);

            instanceWasmPath = wasmPath;
            instanceWasm = new Instance(store, module);
            wasmMemory = instanceWasm?.GetMemory("memory");

            InitializeFunctions();
        }
        private void InitializeFunctions()
        {
            newMazeWasm = ResolveFunction("new_maze_wasm");
            freeMazeWasm = ResolveFunction("free_maze_wasm");
            mazeWasmIsEmpty = ResolveFunction("maze_wasm_is_empty");
            mazeWasmResize = ResolveFunction("maze_wasm_resize");
            mazeWasmReset = ResolveFunction("maze_wasm_reset");
            mazeWasmGetRowCount = ResolveFunction("maze_wasm_get_row_count");
            mazeWasmGetColCount = ResolveFunction("maze_wasm_get_col_count");
            mazeWasmGetCellType = ResolveFunction("maze_wasm_get_cell_type");
            mazeWasmSetStartCell = ResolveFunction("maze_wasm_set_start_cell");
            mazeWasmGetStartCell = ResolveFunction("maze_wasm_get_start_cell");
            mazeWasmSetFinishCell = ResolveFunction("maze_wasm_set_finish_cell");
            mazeWasmGetFinishCell = ResolveFunction("maze_wasm_get_finish_cell");
            mazeWasmSetWallCells = ResolveFunction("maze_wasm_set_wall_cells");
            mazeWasmClearCells = ResolveFunction("maze_wasm_clear_cells");
            mazeWasmInsertRows = ResolveFunction("maze_wasm_insert_rows");
            mazeWasmDeleteRows = ResolveFunction("maze_wasm_delete_rows");
            mazeWasmInsertCols = ResolveFunction("maze_wasm_insert_cols");
            mazeWasmDeleteCols = ResolveFunction("maze_wasm_delete_cols");
            mazeWasmFromJson = ResolveFunction("maze_wasm_from_json");
            mazeWasmToJson = ResolveFunction("maze_wasm_to_json");
            mazeWasmSolve = ResolveFunction("maze_wasm_solve");
            mazeWasmSolutionGetPathPoints = ResolveFunction("maze_wasm_solution_get_path_points");

            freeMazeWasmResult = ResolveFunction("free_maze_wasm_result");
            freeMazeWasmSolution = ResolveFunction("free_maze_wasm_solution");
            freeMazeWasmError = ResolveFunction("free_maze_wasm_error");

            allocateSizedMemory = ResolveFunction("allocate_sized_memory");
            freeSizedMemory = ResolveFunction("free_sized_memory");
            getSizedMemoryUsed = ResolveFunction("get_sized_memory_used");
            getNumObjectsAllocated = ResolveFunction("get_num_objects_allocated");
        }
        private Function ResolveFunction(string functionName)
        {
            Function? func = instanceWasm?.GetFunction(functionName);
            if (func == null)
            {
                throw new Exception($"Failed to load the WebAssembly function: {functionName} from {instanceWasmPath}.");
            }
            return func;
        }
        /// <summary>
        /// Returns the path to the 'maze_wasm' Web Assembly
        /// </summary>
        /// <returns>Web Assembly path</returns>
        static private string GetWasmPath()
        {
            const string WASM_FILE_NAME = "maze_wasm.wasm";
            const string APP_SETTINGS_FILE_NAME = "appsettings.json";
            var executionPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
            if (string.IsNullOrEmpty(executionPath))
            {
                throw new InvalidOperationException("Could not determine execution directory");
            }
            string wasmExecutionFile = Path.Combine(executionPath, WASM_FILE_NAME);
            if (File.Exists(wasmExecutionFile))
            {
                return wasmExecutionFile;
            }
            string appsettingsFile = Path.Combine(executionPath, APP_SETTINGS_FILE_NAME);
            if (!File.Exists(appsettingsFile))
            {
                throw new InvalidOperationException($"Web Assembly file path cannot be determined - no web assembly file found at execution path: '{wasmExecutionFile}' and no application settings file found at: {appsettingsFile}");
            }
            var configuration = new ConfigurationBuilder()
            .SetBasePath(executionPath)
            .AddJsonFile(APP_SETTINGS_FILE_NAME)
            .AddEnvironmentVariables()
            .Build();

            string? path = configuration["MAZE_WASM_PATH"];
            if (string.IsNullOrEmpty(path))
            {
                throw new InvalidOperationException($"MAZE_WASM_PATH environment variable is not set in {APP_SETTINGS_FILE_NAME}");
            }
            return path;
        }
        /// <summary>
        /// Returns the instance for the interop (creating if needed)
        /// </summary>
        /// <returns>Interop instance</returns>
        static public MazeWasmInterop GetInstance()
        {
            if (instance == null)
            {
                instance = new MazeWasmInterop(GetWasmPath());
            }
            return instance;
        }
        /// <summary>
        /// Creates a new, empty `MazeWasm`, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the `MazeWasm`, which should later be freed by calling <see cref="FreeMazeWasm(UInt32)">FreeMazeWasm()</see></returns>
        public UInt32 NewMazeWasm()
        {
            UInt32 mazeWasmPtr = (UInt32)(Int32)(newMazeWasm?.Invoke() ?? 0);
            if (mazeWasmPtr == 0)
            {
                throw new Exception("newMazeWasm() failed (returned zero), possibly due to low memory");
            }
            return mazeWasmPtr;
        }
        /// <summary>
        /// Frees a `MazeWasm` pointer
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to `MazeWasm`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasm(UInt32 mazeWasmPtr)
        {
            freeMazeWasm?.Invoke(mazeWasmPtr);
        }
        /// <summary>
        /// Tests whether a `MazeWasm` is empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeWasmIsEmpty(UInt32 mazeWasmPtr)
        {
            return (Int32)(mazeWasmIsEmpty?.Invoke(mazeWasmPtr) ?? 0) != 0;
        }
        /// <summary>
        /// Resizes a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Zero if successful, as a pointer to an error</returns>
        public void MazeWasmResize(UInt32 mazeWasmPtr, UInt32 newRowCount, UInt32 newColCount)
        {
            mazeWasmResize?.Invoke(mazeWasmPtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a `MazeWasm` to empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeWasmReset(UInt32 mazeWasmPtr)
        {
            mazeWasmReset?.Invoke(mazeWasmPtr);
        }
        /// <summary>
        /// Gets the row count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeWasmGetRowCount(UInt32 mazeWasmPtr)
        {
            return (UInt32)(Int32)(mazeWasmGetRowCount?.Invoke(mazeWasmPtr) ?? 0);
        }
        /// <summary>
        /// Gets the column count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeWasmGetColCount(UInt32 mazeWasmPtr)
        {
            return (UInt32)(Int32)(mazeWasmGetColCount?.Invoke(mazeWasmPtr) ?? 0);
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
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetCellType?.Invoke(mazeWasmPtr, row, col) ?? 0);
            MazeWasmResult result = wasmMemory.Read<MazeWasmResult>(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            MazeWasmCellType cellType = result.value_ptr != 0 ? (MazeWasmCellType)(result.value_ptr) : MazeWasmCellType.Empty;
            FreeMazeWasmResult(resultPtr, true);
            return cellType;
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetStartCell?.Invoke(mazeWasmPtr, startRow, startCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeWasmPoint MazeWasmGetStartCell(UInt32 mazeWasmPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetStartCell?.Invoke(mazeWasmPtr) ?? 0);
            return MazeWasmResultGetPoint(resultPtr, true);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetFinishCell?.Invoke(mazeWasmPtr, finishRow, finishCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>FInish cell point</returns>
        public MazeWasmPoint MazeWasmGetFinishCell(UInt32 mazeWasmPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetFinishCell?.Invoke(mazeWasmPtr) ?? 0);
            return MazeWasmResultGetPoint(resultPtr, true);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetWallCells?.Invoke(mazeWasmPtr, startRow, startCol, endRow, endCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmClearCells?.Invoke(mazeWasmPtr, startRow, startCol, endRow, endCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmInsertRows?.Invoke(mazeWasmPtr, startRow, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmDeleteRows?.Invoke(mazeWasmPtr, startRow, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmInsertCols?.Invoke(mazeWasmPtr, startCol, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmDeleteCols?.Invoke(mazeWasmPtr, startCol, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Reinitialises a `MazeWasm` from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeWasmFromJson(UInt32 mazeWasmPtr, string json)
        {
            var jsonStrPtr = ToStringPtr(json);
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmFromJson?.Invoke(mazeWasmPtr, jsonStrPtr) ?? 0);
            FreeStringPtr(jsonStrPtr);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Converts a `MazeWasm` to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeWasmToJson(UInt32 mazeWasmPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmToJson?.Invoke(mazeWasmPtr) ?? 0);
            MazeWasmResult result = wasmMemory.Read<MazeWasmResult>(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmResultValueType)(result.value_type) != MazeWasmResultValueType.String)
            {
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception("Result value is not a string");
            }
            string json = "";
            if (result.value_ptr != 0)
                json = StringPtrToString(result.value_ptr);
            FreeMazeWasmResult(resultPtr, true);
            return json;
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
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmSolve?.Invoke(mazeWasmPtr) ?? 0);
            MazeWasmResult result = wasmMemory.Read<MazeWasmResult>(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmResultValueType)(result.value_type) != MazeWasmResultValueType.Solution)
            {
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception("Result value is not a solution");
            }
            UInt32 solutionPtr = 0;
            if (result.value_ptr != 0)
                solutionPtr = result.value_ptr;
            FreeMazeWasmResult(resultPtr, false);
            return solutionPtr;
        }
        /// <summary>
        /// Returns the list of points associated a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeWasmPoint> MazeWasmSolutionGetPathPoints(UInt32 solutionPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            if (solutionPtr == 0) throw new Exception("solutionPtr is zero");
            UInt32 pathPointsPtr = (UInt32)(Int32)(mazeWasmSolutionGetPathPoints?.Invoke(solutionPtr) ?? 0);
            UInt32 dataPtr = pathPointsPtr + 4;
            UInt32 numPoints = wasmMemory.Read<UInt32>(dataPtr);
            dataPtr += 4;
            List<MazeWasmPoint> points = new List<MazeWasmPoint>();
            for (UInt32 i = 0; i < numPoints; i++)
            {
                MazeWasmPoint point = wasmMemory.Read<MazeWasmPoint>(dataPtr);
                points.Add(point);
                dataPtr += 8;
            }
            FreeSizedMemory(pathPointsPtr);
            return points;
        }
        /// <summary>
        /// Tides an error pointer and then throws an exception containing the error message associated with that pointer
        /// </summary>
        /// <param name="errorPtr">Pointer to error</param>
        /// <returns>Nothing</returns>
        private void TidyAndThrowError(UInt32 errorPtr)
        {
            string message = GetErrorMessage(errorPtr);
            FreeMazeWasmError(errorPtr);
            throw new Exception(message);
        }
        /// <summary>
        /// Extracts the error message associated with an error pointer or throws an exception of the operation cannot be performed
        /// </summary>
        /// <param name="errorPtr">Pointer to error</param>
        /// <returns>Error message</returns>
        private string GetErrorMessage(UInt32 errorPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            MazeWasmError mazeWasmError = wasmMemory.Read<MazeWasmError>(errorPtr);
            return StringPtrToString(mazeWasmError.message_ptr);
        }
        /// <summary>
        /// Frees a `MazeWasmError` pointer
        /// </summary>
        /// <param name="mazeErrorPtr">Pointer to `MazeWasmError`</param>
        /// <returns>Nothing</returns>
        void FreeMazeWasmError(UInt32 mazeErrorPtr)
        {
            freeMazeWasmError?.Invoke(mazeErrorPtr);
        }
        /// <summary>
        /// Frees a `MazeWasmResult` pointer
        /// </summary>
        /// <param name="resultPtr">Pointer to result</param>
        /// <param name="freeResultValue">Flag indicating whether to free the result value pointed to by `value_ptr` in the `MazeWasmResult`</param>
        /// <returns>Nothing</returns>
        void FreeMazeWasmResult(UInt32 resultPtr, bool freeResultValue)
        {
            freeMazeWasmResult?.Invoke(resultPtr, freeResultValue ? 1 : 0);
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmSolution(UInt32 solutionPtr)
        {
            freeMazeWasmSolution?.Invoke(solutionPtr);
        }
        /// <summary>
        /// Extracts the point value from a result pointer if it exists, else throws
        /// an exception if the result contains an error. In all cases, the result pointer is
        /// free'd if requested.
        /// </summary>
        /// <param name="resultPtr">Pointer to result</param>
        /// <param name="freeResultPtr">Flag indicating whether to free the result</param>
        /// <returns>Point value if successful</returns>
        MazeWasmPoint MazeWasmResultGetPoint(UInt32 resultPtr, bool freeResultPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            MazeWasmResult result = wasmMemory.Read<MazeWasmResult>(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                if (freeResultPtr) FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmResultValueType)(result.value_type) != MazeWasmResultValueType.Point)
            {
                if (freeResultPtr) FreeMazeWasmResult(resultPtr, true);
                throw new Exception("Result value is not a Point");
            }
            MazeWasmPoint point = wasmMemory.Read<MazeWasmPoint>(result.value_ptr + 4);
            if (freeResultPtr) FreeMazeWasmResult(resultPtr, true);
            return point;
        }
        /// <summary>
        /// Allocates a sized memory block of a given size. A sized memory block is a block of 
        /// memory of (`size` + 4) bytes, where the first 4 bytes contain the size of the block (u32)
        /// and then the next `size` bytes is reserved for data use.
        /// </summary>
        /// <param name="size">Number of bytes to allocate</param>
        /// <returns>Pointer to memory</returns>
        UInt32 AllocateSizedMemory(UInt32 size)
        {
            return (UInt32)(Int32)(allocateSizedMemory?.Invoke(size) ?? 0);
        }
        /// <summary>
        /// Frees the sized memory associated with a given pointer
        /// </summary>
        /// <param name="ptr">Pointer to memory</param>
        /// <returns>Nothing</returns>
        void FreeSizedMemory(UInt32 ptr)
        {
            freeSizedMemory?.Invoke(ptr);
        }
        /// <summary>
        /// Gets the amount of sized memory currenty allocated
        /// </summary>
        /// <returns>Memory used count</returns>
        public Int64 GetSizedMemoryUsed()
        {
            return (Int64)(getSizedMemoryUsed?.Invoke() ?? 0);
        }
        /// <summary>
        /// Gets the number of objects currenty allocated
        /// </summary>
        /// <returns>Object count</returns>
        public Int64 GetNumObjectsAllocated()
        {
            return (Int64)(getNumObjectsAllocated?.Invoke() ?? 0);
        }
        /// <summary>
        /// Extracts the string value from a string pointer, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="strPtr">Memory pointer to string</param>
        /// <returns>String value if successful</returns>
        string StringPtrToString(UInt32 strPtr)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            Span<byte> sizeBytes = wasmMemory.GetSpan(strPtr, 4);
            uint length = BitConverter.ToUInt32(sizeBytes);
            return Encoding.UTF8.GetString(wasmMemory.GetSpan(strPtr + 4, (int)length));
        }
        /// <summary>
        /// Converts a string value to a string pointer, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="value">String value to convert</param>
        /// <returns>String pointer if successful</returns>
        UInt32 ToStringPtr(string value)
        {
            if (wasmMemory == null) throw new Exception("wasmMemory is not initialized");
            var strPtr = AllocateSizedMemory((UInt32)value.Length);
            byte[] utf8Bytes = Encoding.UTF8.GetBytes(value);
            Span<byte> memory = wasmMemory.GetSpan((int)strPtr + 4, utf8Bytes.Length);
            utf8Bytes.CopyTo(memory);
            return strPtr;
        }
        /// <summary>
        /// Frees a string pointer
        /// </summary>
        /// <param name="strPtr">Memory pointer to string</param>
        /// <returns>Nothing</returns>
        void FreeStringPtr(UInt32 strPtr)
        {
            FreeSizedMemory(strPtr);
        }
    }
}
