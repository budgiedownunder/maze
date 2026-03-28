#if !IOS && !ANDROID
namespace Maze.Interop
{
    using System.Text;
    using Wasmtime;

    /// <summary>
    /// Provides a wrapper to [Wasmtime](https://docs.wasmtime.dev/) WebAssembly memory
    /// </summary>
    internal class MazeWasmtimeMemory : IMemory
    {
        private Wasmtime.Memory _memory = null!;
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
        /// Reads a `MazeWasmPoint` pointer into a `MazeWasmPoint`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to point</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazeWasmPoint ReadMazeWasmPoint(UInt32 ptrOffset)
        {
            return _memory.Read<MazeInterop.MazeWasmPoint>(ptrOffset);
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
    class MazeWasmtimeFunction : IFunction
    {
        Wasmtime.Function _func;
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
        private ValueBox[] ToValueBoxArray(params object[] values)
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
        private ValueBox ToValueBox(object value) 
        {
            ValueBox result= new ValueBox();
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
                    throw new Exception($"unable to convert argument value to Wasmtime ValueBox - unsupported type {value.GetType().ToString()}");
            }
            return result;
        }
    }
    /// <summary>
    ///  This class provides a C# connector to the `maze_wasm` web assembly via [Wasmtime](https://docs.wasmtime.dev/), insulating the
    ///  calling application from the specifics of the underlying Web Assembly interop operations.
    ///  
    /// Developers can use <see cref="MazeWasmConnectorBase.NewMazeWasm()">NewMazeWasm()</see> to create
    /// a pointer to a maze object within Web Assembly and then other `MazeWasm` functions, such as 
    ///  <see cref="MazeWasmConnectorBase.MazeWasmInsertRows(UIntPtr,uint,uint)">MazeWasmInsertRows()</see>,
    ///  <see cref="MazeWasmConnectorBase.MazeWasmGenerate(UIntPtr,UIntPtr)">MazeWasmGenerate()</see>, and
    ///  <see cref="MazeWasmConnectorBase.MazeWasmSolve(UIntPtr)">MazeWasmSolve()</see>, to interact with the maze.
    ///
    /// Once finished with, a maze should be destroyed using <see cref="MazeWasmConnectorBase.FreeMazeWasm(UIntPtr)">FreeMazeWasm()</see>
    /// to prevent memory leaks within Web Assembly.
    /// </summary>
    class MazeWasmtimeConnector : MazeWasmConnectorBase, IMazeWasmConnector
    {
        private bool _disposed = false;

        // Wasmtime Store and Instance
        private string wasmPathOrName = null!;
        private Store store = null!;
        private Instance instanceWasm= null!;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="wasmPathOrName">WebAssembly path or name. WebAssembly is loaded from this location if `wasmBytes` is `null`.</param>
        /// <param name="wasmBytes">WebAssembly bytes(</param>
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
        /// defined by `wasmPathOrName`.(</param>
        /// <returns>Nothing</returns>
        private void InitializeModule(byte[]? wasmBytes)
        {
            var engine = new Engine();
            var module = wasmBytes is not null 
                ? Wasmtime.Module.FromBytes(engine, "maze_wasm", wasmBytes)
                : Wasmtime.Module.FromFile(engine, wasmPathOrName);
            using var linker = new Linker(engine);
            store = new Store(engine);

            instanceWasm = new Instance(store, module);
            if (instanceWasm is null)
                throw new Exception("Failed to create Wasmtime instance");
        }
        /// <summary>
        /// Initializes the WebAssembly memory
        /// </summary>
        /// <returns>Nothing</returns>
        private void InitializeMemory()
        {
            var wasmtimeMemory = instanceWasm.GetMemory("memory");
            if (wasmtimeMemory is null)
                throw new Exception("Failed to locate memory in WebAssembly");
            base.memory = new MazeWasmtimeMemory(wasmtimeMemory);
        }
        /// <summary>
        /// Initializes the WebAssembly functions
        /// </summary>
        /// <returns>Nothing</returns>
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
            newGeneratorOptionsWasm = ResolveFunction("new_generator_options_wasm");
            generatorOptionsSetStart = ResolveFunction("generator_options_set_start");
            generatorOptionsSetFinish = ResolveFunction("generator_options_set_finish");
            generatorOptionsSetMinSpineLength = ResolveFunction("generator_options_set_min_spine_length");
            generatorOptionsSetMaxRetries = ResolveFunction("generator_options_set_max_retries");
            generatorOptionsSetBranchFromFinish = ResolveFunction("generator_options_set_branch_from_finish");
            mazeWasmGenerate = ResolveFunction("maze_wasm_generate");
            freeGeneratorOptionsWasm = ResolveFunction("free_generator_options_wasm");
        }
        /// <summary>
        /// Locates a WebAssembly function. Will throw an exception if the function is not found.
        /// </summary>
        /// <returns>Function</returns>
        private IFunction ResolveFunction(string functionName)
        {
            Wasmtime.Function? func = instanceWasm?.GetFunction(functionName);
            if (func is null)
            {
                throw new Exception($"Failed to load the WebAssembly function: {functionName} from {wasmPathOrName}.");
            }
            return new MazeWasmtimeFunction(func);
        }
     }
}
#endif
