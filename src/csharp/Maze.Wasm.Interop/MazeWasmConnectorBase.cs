using System.Text;
using static Maze.Wasm.Interop.MazeWasmInterop;

namespace Maze.Wasm.Interop
{
    /// <summary>
    /// WebAssembly memory wrapper interface
    /// </summary>
    internal interface IMemory
    {
        /// <summary>
        /// Reads an unsigned integer from unmanaged memory
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to value</param>
        /// <returns>Value</returns>
        public UInt32 ReadUInt32(UInt32 ptrOffset);
        /// <summary>
        /// Writes an array of bytes to a give target unmanaged memory offset,
        /// which is assumed to have sufficient space
        /// </summary>
        /// <param name="ptrTargetOffset">Target memory pointer offset to write to</param>
        /// <param name="bytes">Byte array</param>
        /// <returns>Value</returns>
        public void WriteBytes(UInt32 ptrTargetOffset, byte[] bytes);
        /// <summary>
        /// Reads a `MazeWasmResult` pointer into a `MazeWasmResult`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to result</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmResult ReadMazeWasmResult(UInt32 ptrOffset);
        /// <summary>
        /// Reads a `MazeWasmPoint` pointer into a `MazeWasmPoint`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to point</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmPoint ReadMazeWasmPoint(UInt32 ptrOffset);
        /// <summary>
        /// Reads a `MazeWasmError` pointer into a `MazeWasmError`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to error</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmError ReadMazeWasmError(UInt32 ptrOffset);
        /// <summary>
        /// Extracts the string value from a string pointer, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="ptrOffset">Memory offset pointer to string</param>
        /// <returns>String value if successful</returns>
        public string StringPtrToString(UInt32 ptrOffset);
    };
    /// <summary>
    /// WebAssembly function wrapper interface
    /// </summary>
    internal interface IFunction
    {
        /// <summary>
        /// Executes the WebAssembly function
        /// </summary>
        /// <param name="args">Arguments list</param>
        /// <returns>Result value</returns>
        public object? Invoke(params object[] args);
    };
    /// <summary>
    /// Base class for MazeWasm connector, containing wrapper functions
    /// for each MazeWasm WebAssembly function.
    /// </summary>
    internal abstract class MazeWasmConnectorBase
    {
        // Memory
        protected IMemory memory = null!;
        // WebAssembly functions
        protected IFunction? newMazeWasm;
        protected IFunction? freeMazeWasm;
        protected IFunction? mazeWasmIsEmpty;
        protected IFunction? mazeWasmResize;
        protected IFunction? mazeWasmReset;
        protected IFunction? mazeWasmGetRowCount;
        protected IFunction? mazeWasmGetColCount;
        protected IFunction? mazeWasmGetCellType;
        protected IFunction? mazeWasmSetStartCell;
        protected IFunction? mazeWasmGetStartCell;
        protected IFunction? mazeWasmSetFinishCell;
        protected IFunction? mazeWasmGetFinishCell;
        protected IFunction? mazeWasmSetWallCells;
        protected IFunction? mazeWasmClearCells;
        protected IFunction? mazeWasmInsertRows;
        protected IFunction? mazeWasmDeleteRows;
        protected IFunction? mazeWasmInsertCols;
        protected IFunction? mazeWasmDeleteCols;
        protected IFunction? mazeWasmFromJson;
        protected IFunction? mazeWasmToJson;
        protected IFunction? mazeWasmSolve;
        protected IFunction? mazeWasmSolutionGetPathPoints;
        protected IFunction? freeMazeWasmResult;
        protected IFunction? freeMazeWasmSolution;
        protected IFunction? freeMazeWasmError;
        protected IFunction? allocateSizedMemory;
        protected IFunction? freeSizedMemory;
        protected IFunction? getSizedMemoryUsed;
        protected IFunction? getNumObjectsAllocated;
        protected IFunction? newGeneratorOptionsWasm;
        protected IFunction? generatorOptionsSetStart;
        protected IFunction? generatorOptionsSetFinish;
        protected IFunction? generatorOptionsSetMinSpineLength;
        protected IFunction? generatorOptionsSetMaxRetries;
        protected IFunction? generatorOptionsSetBranchFromFinish;
        protected IFunction? mazeWasmGenerate;
        protected IFunction? freeGeneratorOptionsWasm;
        /// <summary>
        /// Creates a new, empty `MazeWasm`, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the `MazeWasm`, which should later be freed by calling <see cref="FreeMazeWasm(UIntPtr)">FreeMazeWasm()</see></returns>
        public UIntPtr NewMazeWasm()
        {
            UIntPtr mazeWasmPtr = (UIntPtr)(uint)(int)(newMazeWasm?.Invoke() ?? 0);
            if (mazeWasmPtr == UIntPtr.Zero)
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
        public void FreeMazeWasm(UIntPtr mazeWasmPtr)
        {
            freeMazeWasm?.Invoke((long)(uint)mazeWasmPtr);
        }
        /// <summary>
        /// Tests whether a `MazeWasm` is empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeWasmIsEmpty(UIntPtr mazeWasmPtr)
        {
            return (Int32)(mazeWasmIsEmpty?.Invoke((long)(uint)mazeWasmPtr) ?? 0) != 0;
        }
        /// <summary>
        /// Resizes a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeWasmResize(UIntPtr mazeWasmPtr, UInt32 newRowCount, UInt32 newColCount)
        {
            mazeWasmResize?.Invoke((long)(uint)mazeWasmPtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a `MazeWasm` to empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeWasmReset(UIntPtr mazeWasmPtr)
        {
            mazeWasmReset?.Invoke((long)(uint)mazeWasmPtr);
        }
        /// <summary>
        /// Gets the row count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeWasmGetRowCount(UIntPtr mazeWasmPtr)
        {
            return (UInt32)(Int32)(mazeWasmGetRowCount?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
        }
        /// <summary>
        /// Gets the column count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeWasmGetColCount(UIntPtr mazeWasmPtr)
        {
            return (UInt32)(Int32)(mazeWasmGetColCount?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within a `MazeWasm`, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeWasmInterop.MazeWasmCellType MazeWasmGetCellType(UIntPtr mazeWasmPtr, UInt32 row, UInt32 col)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetCellType?.Invoke((long)(uint)mazeWasmPtr, row, col) ?? 0);
            MazeWasmInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            MazeWasmInterop.MazeWasmCellType cellType = result.value_ptr != 0 ? (MazeWasmInterop.MazeWasmCellType)(result.value_ptr) : MazeWasmInterop.MazeWasmCellType.Empty;
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
        public void MazeWasmSetStartCell(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetStartCell?.Invoke((long)(uint)mazeWasmPtr, startRow, startCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeWasmInterop.MazeWasmPoint MazeWasmGetStartCell(UIntPtr mazeWasmPtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetStartCell?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
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
        public void MazeWasmSetFinishCell(UIntPtr mazeWasmPtr, UInt32 finishRow, UInt32 finishCol)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetFinishCell?.Invoke((long)(uint)mazeWasmPtr, finishRow, finishCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazeWasmInterop.MazeWasmPoint MazeWasmGetFinishCell(UIntPtr mazeWasmPtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmGetFinishCell?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
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
        public void MazeWasmSetWallCells(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmSetWallCells?.Invoke((long)(uint)mazeWasmPtr, startRow, startCol, endRow, endCol) ?? 0);
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
        public void MazeWasmClearCells(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmClearCells?.Invoke((long)(uint)mazeWasmPtr, startRow, startCol, endRow, endCol) ?? 0);
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
        public void MazeWasmInsertRows(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 count)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmInsertRows?.Invoke((long)(uint)mazeWasmPtr, startRow, count) ?? 0);
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
        public void MazeWasmDeleteRows(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 count)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmDeleteRows?.Invoke((long)(uint)mazeWasmPtr, startRow, count) ?? 0);
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
        public void MazeWasmInsertCols(UIntPtr mazeWasmPtr, UInt32 startCol, UInt32 count)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmInsertCols?.Invoke((long)(uint)mazeWasmPtr, startCol, count) ?? 0);
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
        public void MazeWasmDeleteCols(UIntPtr mazeWasmPtr, UInt32 startCol, UInt32 count)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmDeleteCols?.Invoke((long)(uint)mazeWasmPtr, startCol, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Reinitialises a `MazeWasm` from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeWasmFromJson(UIntPtr mazeWasmPtr, string json)
        {
            var jsonStrPtr = ToStringPtr(json);
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmFromJson?.Invoke((long)(uint)mazeWasmPtr, jsonStrPtr) ?? 0);
            FreeStringPtr(jsonStrPtr);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Converts a `MazeWasm` to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeWasmToJson(UIntPtr mazeWasmPtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmToJson?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
            MazeWasmInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmInterop.MazeWasmResultValueType)(result.value_type) != MazeWasmInterop.MazeWasmResultValueType.String)
            {
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception("Result value is not a string");
            }
            string json = "";
            if (result.value_ptr != 0)
                json = memory.StringPtrToString(result.value_ptr);
            FreeMazeWasmResult(resultPtr, true);
            return json;
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails.
        ///
        /// If successful, use <see cref="MazeWasmSolutionGetPathPoints(UIntPtr)">MazeWasmSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeWasmSolution(UIntPtr)">FreeMazeWasmSolution()</see></returns>
        public UIntPtr MazeWasmSolve(UIntPtr mazeWasmPtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeWasmSolve?.Invoke((long)(uint)mazeWasmPtr) ?? 0);
            MazeWasmInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmInterop.MazeWasmResultValueType)(result.value_type) != MazeWasmInterop.MazeWasmResultValueType.Solution)
            {
                FreeMazeWasmResult(resultPtr, true);
                throw new Exception("Result value is not a solution");
            }
            UIntPtr solutionPtr = UIntPtr.Zero;
            if (result.value_ptr != 0)
                solutionPtr = (UIntPtr)result.value_ptr;
            FreeMazeWasmResult(resultPtr, false);
            return solutionPtr;
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmSolution(UIntPtr solutionPtr)
        {
            freeMazeWasmSolution?.Invoke((long)(uint)solutionPtr);
        }
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeWasmInterop.MazeWasmPoint> MazeWasmSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            if (solutionPtr == UIntPtr.Zero) throw new Exception("solutionPtr is zero");
            UInt32 pathPointsPtr = (UInt32)(Int32)(mazeWasmSolutionGetPathPoints?.Invoke((long)(uint)solutionPtr) ?? 0);
            List<MazeWasmInterop.MazeWasmPoint> points = ReadPathPoints(pathPointsPtr);
            FreeSizedMemory(pathPointsPtr);
            return points;
        }
        /// <summary>
        /// Extracts the path points from a points pointer
        /// </summary>
        /// <param name="pathPointsPtrOffset">Points pointer offset</param>
        /// <returns>List of points</returns>
        protected List<MazeWasmInterop.MazeWasmPoint> ReadPathPoints(UInt32 pathPointsPtrOffset)
        {
            UInt32 dataPtrOffset = pathPointsPtrOffset + 4;
            UInt32 numPoints = memory.ReadUInt32(dataPtrOffset);
            dataPtrOffset += 4;
            List<MazeWasmInterop.MazeWasmPoint> points = new List<MazeWasmInterop.MazeWasmPoint>();
            for (UInt32 i = 0; i < numPoints; i++)
            {
                MazeWasmInterop.MazeWasmPoint point = memory.ReadMazeWasmPoint(dataPtrOffset);
                points.Add(point);
                dataPtrOffset += 8;
            }
            return points;
        }
        /// <summary>
        /// Extracts the point value from a result pointer if it exists, else throws
        /// an exception if the result contains an error. In all cases, the result pointer is
        /// free'd if requested.
        /// </summary>
        /// <param name="resultPtrOffset">Memory pointer offset to result</param>
        /// <param name="freeResultPtr">Flag indicating whether to free the result</param>
        /// <returns>Point value if successful</returns>
        MazeWasmInterop.MazeWasmPoint MazeWasmResultGetPoint(UInt32 resultPtrOffset, bool freeResultPtr)
        {
            MazeWasmInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtrOffset);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                if (freeResultPtr) FreeMazeWasmResult(resultPtrOffset, true);
                throw new Exception(errorMessage);
            }
            if ((MazeWasmInterop.MazeWasmResultValueType)(result.value_type) != MazeWasmInterop.MazeWasmResultValueType.Point)
            {
                if (freeResultPtr) FreeMazeWasmResult(resultPtrOffset, true);
                throw new Exception("Result value is not a Point");
            }
            MazeWasmInterop.MazeWasmPoint point = memory.ReadMazeWasmPoint(result.value_ptr + 4);
            if (freeResultPtr) FreeMazeWasmResult(resultPtrOffset, true);
            return point;
        }
        /// <summary>
        /// Frees a `MazeWasmResult` pointer
        /// </summary>
        /// <param name="resultPtrOffset">Memory pointer offset to result</param>
        /// <param name="freeResultValue">Flag indicating whether to free the result value pointed to by `value_ptr` in the `MazeWasmResult`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmResult(UInt32 resultPtrOffset, bool freeResultValue)
        {
            freeMazeWasmResult?.Invoke(resultPtrOffset, freeResultValue ? 1 : 0);
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
            return (UInt32)(Int32)(allocateSizedMemory?.Invoke(size) ?? 0);
        }
        /// <summary>
        /// Frees the sized memory associated with a given pointer
        /// </summary>
        /// <param name="ptr">Pointer to memory</param>
        /// <returns>Nothing</returns>
        public void FreeSizedMemory(UInt32 ptr)
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
        /// Creates a new <c>GeneratorOptionsWasm</c>, or will throw an exception if the operation fails
        /// </summary>
        public UIntPtr NewGeneratorOptionsWasm(UInt32 rowCount, UInt32 colCount, MazeWasmGenerationAlgorithm algorithm, UInt64 seed)
        {
            UIntPtr optionsPtr = (UIntPtr)(uint)(int)(newGeneratorOptionsWasm?.Invoke(rowCount, colCount, (int)algorithm, seed) ?? 0);
            if (optionsPtr == UIntPtr.Zero)
                throw new Exception("new_generator_options_wasm() failed (returned zero), possibly due to low memory");
            return optionsPtr;
        }
        /// <summary>Sets the start cell on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            generatorOptionsSetStart?.Invoke((long)(uint)optionsPtr, row, col);
        }
        /// <summary>Sets the finish cell on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            generatorOptionsSetFinish?.Invoke((long)(uint)optionsPtr, row, col);
        }
        /// <summary>Sets the minimum spine length on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value)
        {
            generatorOptionsSetMinSpineLength?.Invoke((long)(uint)optionsPtr, value);
        }
        /// <summary>Sets the maximum retries on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value)
        {
            generatorOptionsSetMaxRetries?.Invoke((long)(uint)optionsPtr, value);
        }
        /// <summary>Sets the branch_from_finish flag on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value)
        {
            generatorOptionsSetBranchFromFinish?.Invoke((long)(uint)optionsPtr, (int)value);
        }
        /// <summary>
        /// Generates a maze, populating the given <c>MazeWasm</c>, or will throw an exception if the operation fails
        /// </summary>
        public void MazeWasmGenerate(UIntPtr mazeWasmPtr, UIntPtr optionsPtr)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeWasmGenerate?.Invoke((long)(uint)mazeWasmPtr, (long)(uint)optionsPtr) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>Frees a <c>GeneratorOptionsWasm</c> pointer</summary>
        public void FreeGeneratorOptionsWasm(UIntPtr optionsPtr)
        {
            freeGeneratorOptionsWasm?.Invoke((long)(uint)optionsPtr);
        }
        /// <summary>
        /// Tides an error pointer and then throws an exception containing the error message associated with that pointer
        /// </summary>
        /// <param name="errorPtr">Pointer to error</param>
        /// <returns>Nothing</returns>
        protected void TidyAndThrowError(UInt32 errorPtr)
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
        protected string GetErrorMessage(UInt32 errorPtr)
        {
            MazeWasmInterop.MazeWasmError mazeWasmError = memory.ReadMazeWasmError(errorPtr);
            return memory.StringPtrToString(mazeWasmError.message_ptr);
        }
        /// <summary>
        /// Frees a `MazeWasmError` pointer
        /// </summary>
        /// <param name="mazeErrorPtr">Pointer to `MazeWasmError`</param>
        /// <returns>Nothing</returns>
        protected void FreeMazeWasmError(UInt32 mazeErrorPtr)
        {
            freeMazeWasmError?.Invoke(mazeErrorPtr);
        }
        /// <summary>
        /// Frees a string memory pointer offset
        /// </summary>
        /// <param name="strPtrOffset">String memory pointer offset</param>
        /// <returns>Nothing</returns>
        void FreeStringPtr(UInt32 strPtrOffset)
        {
            FreeSizedMemory(strPtrOffset);
        }
        /// <summary>
        /// Converts a string value to a string memory pointer offset, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="value">String value to convert</param>
        /// <returns>String memory pointer offset if successful</returns>
        protected UInt32 ToStringPtr(string value)
        {
            UInt32 strPtrOffset = AllocateSizedMemory((UInt32)value.Length);
            byte[] utf8Bytes = Encoding.UTF8.GetBytes(value);
            memory.WriteBytes(strPtrOffset + 4, utf8Bytes);
            return strPtrOffset;
        }
     }
}
