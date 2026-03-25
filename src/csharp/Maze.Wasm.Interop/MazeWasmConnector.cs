using static Maze.Wasm.Interop.MazeWasmInterop;
using static Maze.Wasm.Interop.WasmerInterop;

namespace Maze.Wasm.Interop
{
    internal interface IMazeWasmConnector : IDisposable
    {
        /// <summary>
        /// Creates a new, empty `MazeWasm`, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the `MazeWasm`, which should later be freed by calling <see cref="FreeMazeWasm(UIntPtr)">FreeMazeWasm()</see></returns>
        public UIntPtr NewMazeWasm();
        /// <summary>
        /// Frees a `MazeWasm` pointer
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to `MazeWasm`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasm(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Tests whether a `MazeWasm` is empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeWasmIsEmpty(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Resizes a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeWasmResize(UIntPtr mazeWasmPtr, UInt32 newRowCount, UInt32 newColCount);
        /// <summary>
        /// Resets a `MazeWasm` to empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeWasmReset(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Gets the row count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeWasmGetRowCount(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Gets the column count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeWasmGetColCount(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Gets the cell type associated with a cell within a `MazeWasm`, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeWasmCellType MazeWasmGetCellType(UIntPtr mazeWasmPtr, UInt32 row, UInt32 col);
        /// <summary>
        /// Sets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetStartCell(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol);
        /// <summary>
        /// Gets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeWasmPoint MazeWasmGetStartCell(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Sets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetFinishCell(UIntPtr mazeWasmPtr, UInt32 finishRow, UInt32 finishCol);
        /// <summary>
        /// Gets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazeWasmPoint MazeWasmGetFinishCell(UIntPtr mazeWasmPtr);
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
        public void MazeWasmSetWallCells(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
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
        public void MazeWasmClearCells(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        /// <summary>
        /// Inserts rows into a `MazeWasm`, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertRows(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Deletes rows from a `MazeWasm`, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteRows(UIntPtr mazeWasmPtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Inserts columns into a `MazeWasm`, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertCols(UIntPtr mazeWasmPtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Deletes columns from a `MazeWasm`, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteCols(UIntPtr mazeWasmPtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Reinitialises a `MazeWasm` from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeWasmFromJson(UIntPtr mazeWasmPtr, string json);
        /// <summary>
        /// Converts a `MazeWasm` to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeWasmToJson(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails.
        ///
        /// If successful, use <see cref="MazeWasmSolutionGetPathPoints(UIntPtr)">MazeWasmSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeWasmSolution(UIntPtr)">FreeMazeWasmSolution()</see></returns>
        public UIntPtr MazeWasmSolve(UIntPtr mazeWasmPtr);
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeWasmPoint> MazeWasmSolutionGetPathPoints(UIntPtr solutionPtr);
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmSolution(UIntPtr solutionPtr);
        /// <summary>
        /// Allocates a sized memory block of a given size. A sized memory block is a block of
        /// memory of (`size` + 4) bytes, where the first 4 bytes contain the size of the block (u32)
        /// and then the next `size` bytes is reserved for data use.
        /// </summary>
        /// <param name="size">Number of bytes to allocate</param>
        /// <returns>Pointer to memory</returns>
        public UInt32 AllocateSizedMemory(UInt32 size);
        /// <summary>
        /// Frees the sized memory associated with a given pointer
        /// </summary>
        /// <param name="ptr">Pointer to memory</param>
        /// <returns>Nothing</returns>
        public void FreeSizedMemory(UInt32 ptr);
        /// <summary>
        /// Gets the amount of sized memory currenty allocated
        /// </summary>
        /// <returns>Memory used count</returns>
        public Int64 GetSizedMemoryUsed();
        /// <summary>
        /// Gets the number of objects currenty allocated
        /// </summary>
        /// <returns>Object count</returns>
        public Int64 GetNumObjectsAllocated();
        /// <summary>
        /// Creates a new <c>GeneratorOptionsWasm</c>, or will throw an exception if the operation fails
        /// </summary>
        public UIntPtr NewGeneratorOptionsWasm(UInt32 rowCount, UInt32 colCount, MazeWasmGenerationAlgorithm algorithm, UInt64 seed);
        /// <summary>Sets the start cell on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col);
        /// <summary>Sets the finish cell on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col);
        /// <summary>Sets the minimum spine length on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value);
        /// <summary>Sets the maximum retries on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value);
        /// <summary>Sets the branch_from_finish flag on a <c>GeneratorOptionsWasm</c></summary>
        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value);
        /// <summary>
        /// Generates a maze, populating the given <c>MazeWasm</c>, or will throw an exception if the operation fails
        /// </summary>
        public void MazeWasmGenerate(UIntPtr mazeWasmPtr, UIntPtr optionsPtr);
        /// <summary>Frees a <c>GeneratorOptionsWasm</c> pointer</summary>
        public void FreeGeneratorOptionsWasm(UIntPtr optionsPtr);
    }
}
