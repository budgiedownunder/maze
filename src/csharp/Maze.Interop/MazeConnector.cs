using static Maze.Interop.MazeInterop;
#if !IOS
using static Maze.Interop.WasmerInterop;
#endif

namespace Maze.Interop
{
    internal interface IMazeConnector : IDisposable
    {
        /// <summary>
        /// Creates a new, empty maze, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>Pointer to the maze, which should later be freed by calling <see cref="FreeMaze(UIntPtr)">FreeMaze()</see></returns>
        public UIntPtr NewMaze();
        /// <summary>
        /// Frees a maze pointer
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void FreeMaze(UIntPtr mazePtr);
        /// <summary>
        /// Tests whether a maze is empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool MazeIsEmpty(UIntPtr mazePtr);
        /// <summary>
        /// Resizes a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void MazeResize(UIntPtr mazePtr, UInt32 newRowCount, UInt32 newColCount);
        /// <summary>
        /// Resets a maze to empty
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Nothing</returns>
        public void MazeReset(UIntPtr mazePtr);
        /// <summary>
        /// Gets the row count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Row count</returns>
        public UInt32 MazeGetRowCount(UIntPtr mazePtr);
        /// <summary>
        /// Gets the column count associated with a maze
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Column count</returns>
        public UInt32 MazeGetColCount(UIntPtr mazePtr);
        /// <summary>
        /// Gets the cell type associated with a cell within a maze, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public MazeCellType MazeGetCellType(UIntPtr mazePtr, UInt32 row, UInt32 col);
        /// <summary>
        /// Sets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void MazeSetStartCell(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol);
        /// <summary>
        /// Gets the start cell associated with a maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Start cell point</returns>
        public MazePoint MazeGetStartCell(UIntPtr mazePtr);
        /// <summary>
        /// Sets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void MazeSetFinishCell(UIntPtr mazePtr, UInt32 finishRow, UInt32 finishCol);
        /// <summary>
        /// Gets the finish cell associated with a maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Finish cell point</returns>
        public MazePoint MazeGetFinishCell(UIntPtr mazePtr);
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
        public void MazeSetWallCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
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
        public void MazeClearCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        /// <summary>
        /// Inserts rows into a maze, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void MazeInsertRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Deletes rows from a maze, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void MazeDeleteRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count);
        /// <summary>
        /// Inserts columns into a maze, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void MazeInsertCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Deletes columns from a maze, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void MazeDeleteCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count);
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void MazeFromJson(UIntPtr mazePtr, string json);
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>JSON string</returns>
        public string MazeToJson(UIntPtr mazePtr);
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails.
        ///
        /// If successful, use <see cref="MazeSolutionGetPathPoints(UIntPtr)">MazeSolutionGetPathPoints()</see> to obtain the
        /// solution path.
        /// </summary>
        /// <param name="mazePtr">Pointer to maze</param>
        /// <returns>Solution pointer, which should later be freed by calling <see cref="FreeMazeSolution(UIntPtr)">FreeMazeSolution()</see></returns>
        public UIntPtr MazeSolve(UIntPtr mazePtr);
        /// <summary>
        /// Returns the list of points associated with a solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>List of points</returns>
        public List<MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr);
        /// <summary>
        /// Frees a maze solution pointer
        /// </summary>
        /// <param name="solutionPtr">Pointer to solution</param>
        /// <returns>Nothing</returns>
        public void FreeMazeSolution(UIntPtr solutionPtr);
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
        /// Creates a new <c>GeneratorOptions</c>, or will throw an exception if the operation fails
        /// </summary>
        public UIntPtr NewGeneratorOptions(UInt32 rowCount, UInt32 colCount, MazeGenerationAlgorithm algorithm, UInt64 seed);
        /// <summary>Sets the start cell on a <c>GeneratorOptions</c></summary>
        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col);
        /// <summary>Sets the finish cell on a <c>GeneratorOptions</c></summary>
        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col);
        /// <summary>Sets the minimum spine length on a <c>GeneratorOptions</c></summary>
        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value);
        /// <summary>Sets the maximum retries on a <c>GeneratorOptions</c></summary>
        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value);
        /// <summary>Sets the branch_from_finish flag on a <c>GeneratorOptions</c></summary>
        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value);
        /// <summary>
        /// Generates a maze, populating the given maze, or will throw an exception if the operation fails
        /// </summary>
        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr);
        /// <summary>Frees a <c>GeneratorOptions</c> pointer</summary>
        public void FreeGeneratorOptions(UIntPtr optionsPtr);
    }
}
