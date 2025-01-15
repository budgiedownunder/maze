using static Maze.Wasm.Interop.MazeWasmInterop;
using static Maze.Wasm.Interop.WasmerInterop;

namespace Maze.Wasm.Interop
{
    internal interface IMazeWasmConnector : IDisposable
    {
        /// <summary>
        /// Creates a new, empty `MazeWasm`, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the `MazeWasm`, which should later be freed by calling <see cref="FreeMazeWasm(UInt32)">FreeMazeWasm()</see></returns>
        public UInt32 NewMazeWasm();
        /// <summary>
        /// Frees a `MazeWasm` pointer
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to `MazeWasm`</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasm(UInt32 mazeWasmPtr);
        /// <summary>
        /// Tests whether a `MazeWasm` is empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeWasmIsEmpty(UInt32 mazeWasmPtr);
        /// <summary>
        /// Resizes a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeWasmResize(UInt32 mazeWasmPtr, UInt32 newRowCount, UInt32 newColCount);
        /// <summary>
        /// Resets a `MazeWasm` to empty
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeWasmReset(UInt32 mazeWasmPtr);
        /// <summary>
        /// Gets the row count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeWasmGetRowCount(UInt32 mazeWasmPtr);
        /// <summary>
        /// Gets the column count associated with a `MazeWasm`
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeWasmGetColCount(UInt32 mazeWasmPtr);
        /// <summary>
        /// Gets the cell type associated with a cell within a `MazeWasm`, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeWasmCellType MazeWasmGetCellType(UInt32 mazeWasmPtr, UInt32 row, UInt32 col);
        /// <summary>
        /// Sets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetStartCell(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol);
        /// <summary>
        /// Gets the start cell associated with a `MazeWasm`, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazeWasmPoint MazeWasmGetStartCell(UInt32 mazeWasmPtr);
        /// <summary>
        /// Sets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void MazeWasmSetFinishCell(UInt32 mazeWasmPtr, UInt32 finishRow, UInt32 finishCol);
        /// <summary>
        /// Gets the finish cell associated with a `MazeWasm`, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazeWasmPoint MazeWasmGetFinishCell(UInt32 mazeWasmPtr);
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
        public void MazeWasmSetWallCells(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
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
        public void MazeWasmClearCells(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        /// <summary>
        /// Inserts rows into a `MazeWasm`, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertRows(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Deletes rows from a `MazeWasm`, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteRows(UInt32 mazeWasmPtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Inserts columns into a `MazeWasm`, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void MazeWasmInsertCols(UInt32 mazeWasmPtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Deletes columns from a `MazeWasm`, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void MazeWasmDeleteCols(UInt32 mazeWasmPtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Reinitialises a `MazeWasm` from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeWasmFromJson(UInt32 mazeWasmPtr, string json);
        /// <summary>
        /// Converts a `MazeWasm` to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeWasmToJson(UInt32 mazeWasmPtr);
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails. 
        ///
        /// If successful, use <see cref="MazeWasmSolutionGetPathPoints(UInt32)">MazeWasmSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeWasmSolution(UInt32)">FreeMazeWasmSolution()</see></returns>
        public UInt32 MazeWasmSolve(UInt32 mazeWasmPtr);
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazeWasmPoint> MazeWasmSolutionGetPathPoints(UInt32 solutionPtr);
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeWasmSolution(UInt32 solutionPtr);
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
    }
}
