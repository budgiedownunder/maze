#if !IOS
using System.Text;
using static Maze.Interop.MazeInterop;

namespace Maze.Interop
{
    /// <summary>
    /// WebAssembly memory wrapper interface
    /// </summary>
    internal interface IWebAssemblyMemory
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
        public MazeInterop.MazeWasmResult ReadMazeWasmResult(UInt32 ptrOffset);
        /// <summary>
        /// Reads a `MazePoint` pointer into a `MazePoint`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to point</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazePoint ReadMazePoint(UInt32 ptrOffset);
        /// <summary>
        /// Reads a `MazeWasmError` pointer into a `MazeWasmError`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to error</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeInterop.MazeWasmError ReadMazeWasmError(UInt32 ptrOffset);
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
    internal interface IWebAssemblyFunction
    {
        /// <summary>
        /// Executes the WebAssembly function
        /// </summary>
        /// <param name="args">Arguments list</param>
        /// <returns>Result value</returns>
        public object? Invoke(params object[] args);
    };
    /// <summary>
    /// Base class for a WebAssembly maze connector, containing wrapper functions
    /// for each WebAssembly function.
    /// </summary>
    internal abstract class MazeWebAssemblyConnectorBase
    {
        // Memory
        protected IWebAssemblyMemory memory = null!;
        // WebAssembly functions
        protected IWebAssemblyFunction? newMaze;
        protected IWebAssemblyFunction? freeMaze;
        protected IWebAssemblyFunction? mazeIsEmpty;
        protected IWebAssemblyFunction? mazeResize;
        protected IWebAssemblyFunction? mazeReset;
        protected IWebAssemblyFunction? mazeGetRowCount;
        protected IWebAssemblyFunction? mazeGetColCount;
        protected IWebAssemblyFunction? mazeGetCellType;
        protected IWebAssemblyFunction? mazeSetStartCell;
        protected IWebAssemblyFunction? mazeGetStartCell;
        protected IWebAssemblyFunction? mazeSetFinishCell;
        protected IWebAssemblyFunction? mazeGetFinishCell;
        protected IWebAssemblyFunction? mazeSetWallCells;
        protected IWebAssemblyFunction? mazeClearCells;
        protected IWebAssemblyFunction? mazeInsertRows;
        protected IWebAssemblyFunction? mazeDeleteRows;
        protected IWebAssemblyFunction? mazeInsertCols;
        protected IWebAssemblyFunction? mazeDeleteCols;
        protected IWebAssemblyFunction? mazeFromJson;
        protected IWebAssemblyFunction? mazeToJson;
        protected IWebAssemblyFunction? mazeSolve;
        protected IWebAssemblyFunction? mazeSolutionGetPathPoints;
        protected IWebAssemblyFunction? freeMazeResult;
        protected IWebAssemblyFunction? freeMazeSolution;
        protected IWebAssemblyFunction? freeMazeError;
        protected IWebAssemblyFunction? allocateSizedMemory;
        protected IWebAssemblyFunction? freeSizedMemory;
        protected IWebAssemblyFunction? getSizedMemoryUsed;
        protected IWebAssemblyFunction? getNumObjectsAllocated;
        protected IWebAssemblyFunction? newGeneratorOptions;
        protected IWebAssemblyFunction? generatorOptionsSetStart;
        protected IWebAssemblyFunction? generatorOptionsSetFinish;
        protected IWebAssemblyFunction? generatorOptionsSetMinSpineLength;
        protected IWebAssemblyFunction? generatorOptionsSetMaxRetries;
        protected IWebAssemblyFunction? generatorOptionsSetBranchFromFinish;
        protected IWebAssemblyFunction? mazeGenerate;
        protected IWebAssemblyFunction? freeGeneratorOptions;
        protected IWebAssemblyFunction? newMazeGame;
        protected IWebAssemblyFunction? freeMazeGame;
        protected IWebAssemblyFunction? mazeGameMovePlayer;
        protected IWebAssemblyFunction? mazeGamePlayerRow;
        protected IWebAssemblyFunction? mazeGamePlayerCol;
        protected IWebAssemblyFunction? mazeGamePlayerDirection;
        protected IWebAssemblyFunction? mazeGameIsComplete;
        protected IWebAssemblyFunction? mazeGameVisitedCellCount;
        protected IWebAssemblyFunction? mazeGameGetVisitedCell;
        /// <summary>
        /// Creates a new, empty maze, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the maze, which should later be freed by calling <see cref="FreeMaze(UIntPtr)">FreeMaze()</see></returns>
        public UIntPtr NewMaze()
        {
            UIntPtr mazePtr = (UIntPtr)(uint)(int)(newMaze?.Invoke() ?? 0);
            if (mazePtr == UIntPtr.Zero)
            {
                throw new Exception("newMazeWasm() failed (returned zero), possibly due to low memory");
            }
            return mazePtr;
        }
        /// <summary>
        /// Frees a maze pointer
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void FreeMaze(UIntPtr mazePtr)
        {
            freeMaze?.Invoke((long)(uint)mazePtr);
        }
        /// <summary>
        /// Tests whether a maze is empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeIsEmpty(UIntPtr mazePtr)
        {
            return (Int32)(mazeIsEmpty?.Invoke((long)(uint)mazePtr) ?? 0) != 0;
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
            mazeResize?.Invoke((long)(uint)mazePtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets a maze to empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeReset(UIntPtr mazePtr)
        {
            mazeReset?.Invoke((long)(uint)mazePtr);
        }
        /// <summary>
        /// Gets the row count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeGetRowCount(UIntPtr mazePtr)
        {
            return (UInt32)(Int32)(mazeGetRowCount?.Invoke((long)(uint)mazePtr) ?? 0);
        }
        /// <summary>
        /// Gets the column count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeGetColCount(UIntPtr mazePtr)
        {
            return (UInt32)(Int32)(mazeGetColCount?.Invoke((long)(uint)mazePtr) ?? 0);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within a maze, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeInterop.MazeCellType MazeGetCellType(UIntPtr mazePtr, UInt32 row, UInt32 col)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeGetCellType?.Invoke((long)(uint)mazePtr, row, col) ?? 0);
            MazeInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            MazeInterop.MazeCellType cellType = result.value_ptr != 0 ? (MazeInterop.MazeCellType)(result.value_ptr) : MazeInterop.MazeCellType.Empty;
            FreeMazeResult(resultPtr, true);
            return cellType;
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeSetStartCell?.Invoke((long)(uint)mazePtr, startRow, startCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeInterop.MazePoint MazeGetStartCell(UIntPtr mazePtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeGetStartCell?.Invoke((long)(uint)mazePtr) ?? 0);
            return MazeResultGetPoint(resultPtr, true);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeSetFinishCell?.Invoke((long)(uint)mazePtr, finishRow, finishCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Gets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazeInterop.MazePoint MazeGetFinishCell(UIntPtr mazePtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeGetFinishCell?.Invoke((long)(uint)mazePtr) ?? 0);
            return MazeResultGetPoint(resultPtr, true);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeSetWallCells?.Invoke((long)(uint)mazePtr, startRow, startCol, endRow, endCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeClearCells?.Invoke((long)(uint)mazePtr, startRow, startCol, endRow, endCol) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeInsertRows?.Invoke((long)(uint)mazePtr, startRow, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeDeleteRows?.Invoke((long)(uint)mazePtr, startRow, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeInsertCols?.Invoke((long)(uint)mazePtr, startCol, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
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
            UInt32 errorPtr = (UInt32)(Int32)(mazeDeleteCols?.Invoke((long)(uint)mazePtr, startCol, count) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeFromJson(UIntPtr mazePtr, string json)
        {
            var jsonStrPtr = ToStringPtr(json);
            UInt32 errorPtr = (UInt32)(Int32)(mazeFromJson?.Invoke((long)(uint)mazePtr, jsonStrPtr) ?? 0);
            FreeStringPtr(jsonStrPtr);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeToJson(UIntPtr mazePtr)
        {
            UInt32 resultPtr = (UInt32)(Int32)(mazeToJson?.Invoke((long)(uint)mazePtr) ?? 0);
            MazeInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeInterop.MazeWasmResultValueType)(result.value_type) != MazeInterop.MazeWasmResultValueType.String)
            {
                FreeMazeResult(resultPtr, true);
                throw new Exception("Result value is not a string");
            }
            string json = "";
            if (result.value_ptr != 0)
                json = memory.StringPtrToString(result.value_ptr);
            FreeMazeResult(resultPtr, true);
            return json;
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
            UInt32 resultPtr = (UInt32)(Int32)(mazeSolve?.Invoke((long)(uint)mazePtr) ?? 0);
            MazeInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtr);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                FreeMazeResult(resultPtr, true);
                throw new Exception(errorMessage);
            }
            if ((MazeInterop.MazeWasmResultValueType)(result.value_type) != MazeInterop.MazeWasmResultValueType.Solution)
            {
                FreeMazeResult(resultPtr, true);
                throw new Exception("Result value is not a solution");
            }
            UIntPtr solutionPtr = UIntPtr.Zero;
            if (result.value_ptr != 0)
                solutionPtr = (UIntPtr)result.value_ptr;
            FreeMazeResult(resultPtr, false);
            return solutionPtr;
        }
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeSolution(UIntPtr solutionPtr)
        {
            freeMazeSolution?.Invoke((long)(uint)solutionPtr);
        }
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeInterop.MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            if (solutionPtr == UIntPtr.Zero) throw new Exception("solutionPtr is zero");
            UInt32 pathPointsPtr = (UInt32)(Int32)(mazeSolutionGetPathPoints?.Invoke((long)(uint)solutionPtr) ?? 0);
            List<MazeInterop.MazePoint> points = ReadPathPoints(pathPointsPtr);
            FreeSizedMemory(pathPointsPtr);
            return points;
        }
        /// <summary>
        /// Extracts the path points from a points pointer
        /// </summary>
        /// <param name="pathPointsPtrOffset">Points pointer offset</param>
        /// <returns>List of points</returns>
        protected List<MazeInterop.MazePoint> ReadPathPoints(UInt32 pathPointsPtrOffset)
        {
            UInt32 dataPtrOffset = pathPointsPtrOffset + 4;
            UInt32 numPoints = memory.ReadUInt32(dataPtrOffset);
            dataPtrOffset += 4;
            List<MazeInterop.MazePoint> points = new List<MazeInterop.MazePoint>();
            for (UInt32 i = 0; i < numPoints; i++)
            {
                MazeInterop.MazePoint point = memory.ReadMazePoint(dataPtrOffset);
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
        MazeInterop.MazePoint MazeResultGetPoint(UInt32 resultPtrOffset, bool freeResultPtr)
        {
            MazeInterop.MazeWasmResult result = memory.ReadMazeWasmResult(resultPtrOffset);
            if (result.error_ptr != 0)
            {
                string errorMessage = GetErrorMessage(result.error_ptr);
                if (freeResultPtr) FreeMazeResult(resultPtrOffset, true);
                throw new Exception(errorMessage);
            }
            if ((MazeInterop.MazeWasmResultValueType)(result.value_type) != MazeInterop.MazeWasmResultValueType.Point)
            {
                if (freeResultPtr) FreeMazeResult(resultPtrOffset, true);
                throw new Exception("Result value is not a Point");
            }
            MazeInterop.MazePoint point = memory.ReadMazePoint(result.value_ptr + 4);
            if (freeResultPtr) FreeMazeResult(resultPtrOffset, true);
            return point;
        }
        /// <summary>
        /// Frees a `MazeWasmResult` pointer
        /// </summary>
        /// <param name="resultPtrOffset">Memory pointer offset to result</param>
        /// <param name="freeResultValue">Flag indicating whether to free the result value pointed to by `value_ptr` in the `MazeWasmResult`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeResult(UInt32 resultPtrOffset, bool freeResultValue)
        {
            freeMazeResult?.Invoke(resultPtrOffset, freeResultValue ? 1 : 0);
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
        /// Creates a new <c>GeneratorOptions</c>, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="rowCount">Number of rows</param>
        /// <param name="colCount">Number of columns</param>
        /// <param name="algorithm">Generation algorithm to use</param>
        /// <param name="seed">Random seed for generation</param>
        /// <returns>Pointer to the new generator options</returns>
        public UIntPtr NewGeneratorOptions(UInt32 rowCount, UInt32 colCount, MazeGenerationAlgorithm algorithm, UInt64 seed)
        {
            UIntPtr optionsPtr = (UIntPtr)(uint)(int)(newGeneratorOptions?.Invoke(rowCount, colCount, (int)algorithm, seed) ?? 0);
            if (optionsPtr == UIntPtr.Zero)
                throw new Exception("new_generator_options_wasm() failed (returned zero), possibly due to low memory");
            return optionsPtr;
        }
        /// <summary>Sets the start cell on a <c>GeneratorOptions</c></summary>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="col">Column index (zero-based)</param>
        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            generatorOptionsSetStart?.Invoke((long)(uint)optionsPtr, row, col);
        }
        /// <summary>Sets the finish cell on a <c>GeneratorOptions</c></summary>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="col">Column index (zero-based)</param>
        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            generatorOptionsSetFinish?.Invoke((long)(uint)optionsPtr, row, col);
        }
        /// <summary>Sets the minimum spine length on a <c>GeneratorOptions</c></summary>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        /// <param name="value">Minimum spine length</param>
        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value)
        {
            generatorOptionsSetMinSpineLength?.Invoke((long)(uint)optionsPtr, value);
        }
        /// <summary>Sets the maximum retries on a <c>GeneratorOptions</c></summary>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        /// <param name="value">Maximum number of retries</param>
        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value)
        {
            generatorOptionsSetMaxRetries?.Invoke((long)(uint)optionsPtr, value);
        }
        /// <summary>Sets the branch_from_finish flag on a <c>GeneratorOptions</c></summary>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        /// <param name="value">Non-zero to enable branching from the finish cell</param>
        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value)
        {
            generatorOptionsSetBranchFromFinish?.Invoke((long)(uint)optionsPtr, (int)value);
        }
        /// <summary>
        /// Generates a maze, populating the given maze, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to the maze to populate</param>
        /// <param name="optionsPtr">Pointer to the generator options</param>
        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            UInt32 errorPtr = (UInt32)(Int32)(mazeGenerate?.Invoke((long)(uint)mazePtr, (long)(uint)optionsPtr) ?? 0);
            if (errorPtr != 0)
                TidyAndThrowError(errorPtr);
        }
        /// <summary>Frees a <c>GeneratorOptions</c> pointer</summary>
        /// <param name="optionsPtr">Pointer to the generator options to free</param>
        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            freeGeneratorOptions?.Invoke((long)(uint)optionsPtr);
        }
        /// <summary>
        /// Tides an error pointer and then throws an exception containing the error message associated with that pointer
        /// </summary>
        /// <param name="errorPtr">Pointer to error</param>
        /// <returns>Nothing</returns>
        protected void TidyAndThrowError(UInt32 errorPtr)
        {
            string message = GetErrorMessage(errorPtr);
            FreeMazeError(errorPtr);
            throw new Exception(message);
        }
        /// <summary>
        /// Extracts the error message associated with an error pointer or throws an exception of the operation cannot be performed
        /// </summary>
        /// <param name="errorPtr">Pointer to error</param>
        /// <returns>Error message</returns>
        protected string GetErrorMessage(UInt32 errorPtr)
        {
            MazeInterop.MazeWasmError mazeWasmError = memory.ReadMazeWasmError(errorPtr);
            return memory.StringPtrToString(mazeWasmError.message_ptr);
        }
        /// <summary>
        /// Frees a `MazeWasmError` pointer
        /// </summary>
        /// <param name="mazeErrorPtr">Pointer to `MazeWasmError`</param>
        /// <returns>Nothing</returns>
        protected void FreeMazeError(UInt32 mazeErrorPtr)
        {
            freeMazeError?.Invoke(mazeErrorPtr);
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
        /// <summary>
        /// Creates a new maze game session from a maze definition JSON string,
        /// or will throw an exception if the operation fails.
        /// </summary>
        /// <param name="definitionJson">Maze definition JSON string ({"grid":[...]})</param>
        /// <returns>Opaque game session pointer. Free with <see cref="FreeMazeGame(UIntPtr)">FreeMazeGame()</see> when done.</returns>
        public UIntPtr NewMazeGame(string definitionJson)
        {
            var jsonStrPtr = ToStringPtr(definitionJson);
            UIntPtr gamePtr = (UIntPtr)(uint)(int)(newMazeGame?.Invoke(jsonStrPtr) ?? 0);
            FreeStringPtr(jsonStrPtr);
            if (gamePtr == UIntPtr.Zero)
                throw new Exception("Failed to create maze game session — invalid JSON or maze has no start cell");
            return gamePtr;
        }
        /// <summary>
        /// Frees a game session pointer returned by <see cref="NewMazeGame(string)">NewMazeGame()</see>
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        public void FreeMazeGame(UIntPtr gamePtr)
        {
            freeMazeGame?.Invoke((long)(uint)gamePtr);
        }
        /// <summary>
        /// Moves the player one cell in the given direction
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <param name="dir">Direction: 0=None 1=Up 2=Down 3=Left 4=Right</param>
        /// <returns>0=None 1=Moved 2=Blocked 3=Complete</returns>
        public int MazeGameMovePlayer(UIntPtr gamePtr, int dir)
        {
            return (int)(mazeGameMovePlayer?.Invoke((long)(uint)gamePtr, dir) ?? 0);
        }
        /// <summary>
        /// Gets the player's current row (zero-based)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Row index</returns>
        public int MazeGamePlayerRow(UIntPtr gamePtr)
        {
            return (int)(mazeGamePlayerRow?.Invoke((long)(uint)gamePtr) ?? 0);
        }
        /// <summary>
        /// Gets the player's current column (zero-based)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Column index</returns>
        public int MazeGamePlayerCol(UIntPtr gamePtr)
        {
            return (int)(mazeGamePlayerCol?.Invoke((long)(uint)gamePtr) ?? 0);
        }
        /// <summary>
        /// Gets the player's current facing direction
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>0=None 1=Up 2=Down 3=Left 4=Right</returns>
        public int MazeGamePlayerDirection(UIntPtr gamePtr)
        {
            return (int)(mazeGamePlayerDirection?.Invoke((long)(uint)gamePtr) ?? 0);
        }
        /// <summary>
        /// Returns whether the player has reached the finish cell
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>1 if complete, 0 otherwise</returns>
        public int MazeGameIsComplete(UIntPtr gamePtr)
        {
            return (int)(mazeGameIsComplete?.Invoke((long)(uint)gamePtr) ?? 0);
        }
        /// <summary>
        /// Returns the number of cells visited by the player (including the start cell)
        /// </summary>
        /// <param name="gamePtr">Pointer to game session</param>
        /// <returns>Visited cell count</returns>
        public int MazeGameVisitedCellCount(UIntPtr gamePtr)
        {
            return (int)(mazeGameVisitedCellCount?.Invoke((long)(uint)gamePtr) ?? 0);
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
            // Allocate 4-byte output slots in WASM linear memory.
            // Pass offset+4 to skip the sized-memory header so the function writes into the data region.
            UInt32 rowOutPtr = AllocateSizedMemory(4);
            UInt32 colOutPtr = AllocateSizedMemory(4);
            int result = (int)(mazeGameGetVisitedCell?.Invoke(
                (long)(uint)gamePtr, index,
                (long)(uint)(rowOutPtr + 4),
                (long)(uint)(colOutPtr + 4)) ?? -1);
            row = (int)memory.ReadUInt32(rowOutPtr + 4);
            col = (int)memory.ReadUInt32(colOutPtr + 4);
            FreeSizedMemory(rowOutPtr);
            FreeSizedMemory(colOutPtr);
            return result == 0;
        }
    }
}
#endif // !IOS
