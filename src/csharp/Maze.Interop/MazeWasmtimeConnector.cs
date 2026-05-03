#if !IOS && !ANDROID
namespace Maze.Interop
{
    using System.Text;
    using Wasmtime;

    /// <summary>
    /// Provides a wrapper to [Wasmtime](https://docs.wasmtime.dev/) WebAssembly memory
    /// </summary>
    internal class MazeWasmtimeMemory : IWebAssemblyMemory
    {
        private readonly Wasmtime.Memory _memory = null!;
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="memory">WebAssembly memory</param>
        internal MazeWasmtimeMemory(Wasmtime.Memory memory)
        {
            if (memory is null)
            {
                throw new Exception("Null wasmMemory supplied to MazeWasmerMemory constructor");
            }
            _memory = memory;
        }
        /// <summary>
        /// Reads an unsigned integer from unmanaged memory
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to value</param>
        /// <returns>Value</returns>
        public UInt32 ReadUInt32(UInt32 ptrOffset)
        {
            return _memory.Read<UInt32>(ptrOffset);
        }
        /// <summary>
        /// Writes an array of bytes to a give target unmanaged memory offset,
        /// which is assumed to have sufficient space
        /// </summary>
        /// <param name="ptrTargetOffset">Target memory pointer offset to write to</param>
        /// <param name="bytes">Byte array</param>
        /// <returns>Value</returns>
        public void WriteBytes(UInt32 ptrTargetOffset, byte[] bytes)
        {
            Span<byte> memory = _memory.GetSpan((int)ptrTargetOffset, bytes.Length);
            bytes.CopyTo(memory);
        }
        /// <summary>
        /// Reads a `MazeWasmResult` pointer into a `MazeWasmResult`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to result</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazeWasmResult ReadMazeWasmResult(UInt32 ptrOffset)
        {
            return _memory.Read<MazeInterop.MazeWasmResult>(ptrOffset);
        }
        /// <summary>
        /// Reads a `MazePoint` pointer into a `MazePoint`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to point</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazePoint ReadMazePoint(UInt32 ptrOffset)
        {
            return _memory.Read<MazeInterop.MazePoint>(ptrOffset);
        }
        /// <summary>
        /// Reads a `MazeWasmError` pointer into a `MazeWasmError`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to error</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazeWasmError ReadMazeWasmError(UInt32 ptrOffset)
        {
            return _memory.Read<MazeInterop.MazeWasmError>(ptrOffset);
        }
        /// <summary>
        /// Extracts the string value from a string pointer, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="ptrOffset">Memory offset pointer to string</param>
        /// <returns>String value if successful</returns>
        public string StringPtrToString(UInt32 ptrOffset)
        {
            Span<byte> sizeBytes = _memory.GetSpan(ptrOffset, 4);
            UInt32 length = BitConverter.ToUInt32(sizeBytes);
            return Encoding.UTF8.GetString(_memory.GetSpan(ptrOffset + 4, (int)length));
        }
    };
    /// <summary>
    ///  This class provides a C# wrapper to a [Wasmtime](https://docs.wasmtime.dev/) WebAssembly
    ///  function
    /// </summary>
    class MazeWasmtimeFunction : IWebAssemblyFunction
    {
        readonly Wasmtime.Function _func;
        /// <summary>
        /// Constructor
        /// </summary>
        public MazeWasmtimeFunction(Wasmtime.Function func)
        {
            this._func = func;
        }
        /// <summary>
        /// Invokes the given function with the given arguments
        /// </summary>
        /// <param name="args">Function arguments</param>
        /// <returns>Result (will be `null` if the function has no result)</returns>
        public object? Invoke(params object[] args)
        {
            ValueBox[] wasmerArgs = ToValueBoxArray(args);
            return this._func?.Invoke(wasmerArgs);
        }
        /// <summary>
        /// Converts an array of object values to an array of [Wasmtime](https://docs.wasmtime.dev/) `ValueBox`
        /// values
        /// </summary>
        /// <returns>Array result</returns>
        private static ValueBox[] ToValueBoxArray(params object[] values)
        {
            if (values.Length == 0)
                return [];
            ValueBox[] arr = new ValueBox[values.Length];
            for (int i = 0; i < values.Length; i++)
            {
                arr[i] = ToValueBox(values[i]);
            }
            return arr;
        }
        /// <summary>
        /// Converts an object value to a [Wasmtime](https://docs.wasmtime.dev/) `ValueBox`
        /// value
        /// </summary>
        /// <returns>Result</returns>
        private static ValueBox ToValueBox(object value)
        {
            ValueBox result;
            switch (value)
            {
                case int intValue:
                    result = intValue;
                    break;
                case uint uintValue:
                    result = (long)uintValue;
                    break;
                case long longValue:
                    result = longValue;
                    break;
                case ulong ulongValue:
                    result = (long)ulongValue;
                    break;
                case float floatValue:
                    result = floatValue;
                    break;
                case double doubleValue:
                    result = doubleValue;
                    break;
                default:
                    throw new Exception($"unable to convert argument value to Wasmtime ValueBox - unsupported type {value.GetType()}");
            }
            return result;
        }
    }
    /// <summary>
    ///  This class provides a C# connector to the <c>maze_wasm</c> WebAssembly module via
    ///  [Wasmtime](https://docs.wasmtime.dev/), insulating the calling application from the
    ///  specifics of the underlying interop operations.
    ///
    /// Developers can use <see cref="MazeWebAssemblyConnectorBase.NewMaze()">NewMaze()</see> to create
    /// a pointer to a maze object and then other maze functions, such as
    ///  <see cref="MazeWebAssemblyConnectorBase.MazeInsertRows(UIntPtr,uint,uint)">MazeInsertRows()</see>,
    ///  <see cref="MazeWebAssemblyConnectorBase.MazeGenerate(UIntPtr,UIntPtr)">MazeGenerate()</see>, and
    ///  <see cref="MazeWebAssemblyConnectorBase.MazeSolve(UIntPtr)">MazeSolve()</see>, to interact with the maze.
    ///
    /// Once finished with, a maze should be destroyed using <see cref="MazeWebAssemblyConnectorBase.FreeMaze(UIntPtr)">FreeMaze()</see>
    /// to prevent memory leaks.
    /// </summary>
    class MazeWasmtimeConnector : MazeWebAssemblyConnectorBase, IMazeConnector
    {
        private bool _disposed = false;

        // Wasmtime Store and Instance

        private readonly string wasmPathOrName = null!;
        private Store store = null!;
        private Instance instanceWasm = null!;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="wasmPathOrName">WebAssembly path or name. WebAssembly is loaded from this location if `wasmBytes` is `null`.</param>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null` then an attempt is made to load WebAssembly from the path
        /// defined by `wasmPathOrName`.</param>
        public MazeWasmtimeConnector(string wasmPathOrName, byte[]? wasmBytes = null)
        {
            if (wasmPathOrName is null)
                throw new Exception("WebAssembly path or name is not defined");
            this.wasmPathOrName = wasmPathOrName;
            Initialize(wasmBytes);
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~MazeWasmtimeConnector()
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
                _disposed = true;
            }
        }
        /// <summary>
        /// Initializes the object
        /// </summary>
        /// <returns>Nothing</returns>
        private void Initialize(byte[]? wasmBytes)
        {
            InitializeModule(wasmBytes);
            InitializeMemory();
            InitializeFunctions();
        }
        /// <summary>
        /// Initializes the WebAssembly module
        /// </summary>
        /// <param name="wasmBytes">WebAssembly bytes. If this is `null`, then the WebAssembly will be loaded from the path
        /// defined by `wasmPathOrName`.</param>
        /// <returns>Nothing</returns>
        private void InitializeModule(byte[]? wasmBytes)
        {
            var engine = new Engine();
            var module = wasmBytes is not null
                ? Wasmtime.Module.FromBytes(engine, "maze_wasm", wasmBytes)
                : Wasmtime.Module.FromFile(engine, wasmPathOrName);
            using var linker = new Linker(engine);
            store = new Store(engine);

            instanceWasm = new Instance(store, module) ?? throw new Exception("Failed to create Wasmtime instance");
        }
        /// <summary>
        /// Initializes the WebAssembly memory
        /// </summary>
        /// <returns>Nothing</returns>
        private void InitializeMemory()
        {
            var wasmtimeMemory = instanceWasm.GetMemory("memory") ?? throw new Exception("Failed to locate memory in WebAssembly");
            base.memory = new MazeWasmtimeMemory(wasmtimeMemory);
        }
        /// <summary>
        /// Initializes the WebAssembly functions
        /// </summary>
        /// <returns>Nothing</returns>
        private void InitializeFunctions()
        {
            newMaze = ResolveFunction("new_maze_wasm");
            freeMaze = ResolveFunction("free_maze_wasm");
            mazeIsEmpty = ResolveFunction("maze_wasm_is_empty");
            mazeResize = ResolveFunction("maze_wasm_resize");
            mazeReset = ResolveFunction("maze_wasm_reset");
            mazeGetRowCount = ResolveFunction("maze_wasm_get_row_count");
            mazeGetColCount = ResolveFunction("maze_wasm_get_col_count");
            mazeGetCellType = ResolveFunction("maze_wasm_get_cell_type");
            mazeSetStartCell = ResolveFunction("maze_wasm_set_start_cell");
            mazeGetStartCell = ResolveFunction("maze_wasm_get_start_cell");
            mazeSetFinishCell = ResolveFunction("maze_wasm_set_finish_cell");
            mazeGetFinishCell = ResolveFunction("maze_wasm_get_finish_cell");
            mazeSetWallCells = ResolveFunction("maze_wasm_set_wall_cells");
            mazeClearCells = ResolveFunction("maze_wasm_clear_cells");
            mazeInsertRows = ResolveFunction("maze_wasm_insert_rows");
            mazeDeleteRows = ResolveFunction("maze_wasm_delete_rows");
            mazeInsertCols = ResolveFunction("maze_wasm_insert_cols");
            mazeDeleteCols = ResolveFunction("maze_wasm_delete_cols");
            mazeFromJson = ResolveFunction("maze_wasm_from_json");
            mazeToJson = ResolveFunction("maze_wasm_to_json");
            mazeSolve = ResolveFunction("maze_wasm_solve");
            mazeSolutionGetPathPoints = ResolveFunction("maze_wasm_solution_get_path_points");
            freeMazeResult = ResolveFunction("free_maze_wasm_result");
            freeMazeSolution = ResolveFunction("free_maze_wasm_solution");
            freeMazeError = ResolveFunction("free_maze_wasm_error");
            allocateSizedMemory = ResolveFunction("allocate_sized_memory");
            freeSizedMemory = ResolveFunction("free_sized_memory");
            getSizedMemoryUsed = ResolveFunction("get_sized_memory_used");
            getNumObjectsAllocated = ResolveFunction("get_num_objects_allocated");
            newGeneratorOptions = ResolveFunction("new_generator_options_wasm");
            generatorOptionsSetStart = ResolveFunction("generator_options_set_start");
            generatorOptionsSetFinish = ResolveFunction("generator_options_set_finish");
            generatorOptionsSetMinSpineLength = ResolveFunction("generator_options_set_min_spine_length");
            generatorOptionsSetMaxRetries = ResolveFunction("generator_options_set_max_retries");
            generatorOptionsSetBranchFromFinish = ResolveFunction("generator_options_set_branch_from_finish");
            mazeGenerate = ResolveFunction("maze_wasm_generate");
            freeGeneratorOptions = ResolveFunction("free_generator_options_wasm");
            newMazeGame = ResolveFunction("new_maze_game_wasm");
            freeMazeGame = ResolveFunction("free_maze_game_wasm");
            mazeGameMovePlayer = ResolveFunction("maze_game_wasm_move_player");
            mazeGamePlayerRow = ResolveFunction("maze_game_wasm_player_row");
            mazeGamePlayerCol = ResolveFunction("maze_game_wasm_player_col");
            mazeGamePlayerDirection = ResolveFunction("maze_game_wasm_player_direction");
            mazeGameIsComplete = ResolveFunction("maze_game_wasm_is_complete");
            mazeGameVisitedCellCount = ResolveFunction("maze_game_wasm_visited_cell_count");
            mazeGameGetVisitedCell = ResolveFunction("maze_game_wasm_get_visited_cell");
        }
        /// <summary>
        /// Locates a WebAssembly function. Will throw an exception if the function is not found.
        /// </summary>
        /// <returns>Function</returns>
        private IWebAssemblyFunction ResolveFunction(string functionName)
        {
            Wasmtime.Function? func = (instanceWasm?.GetFunction(functionName)) ?? throw new Exception($"Failed to load the WebAssembly function: {functionName} from {wasmPathOrName}.");
            return new MazeWasmtimeFunction(func);
        }
    }
}
#endif
