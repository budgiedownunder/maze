namespace Maze.Interop
{
    using System;
    using System.Collections.Generic;
    using System.Threading;
    using static Maze.Interop.MazeInterop;

    /// <summary>
    /// Serialises every call to an <see cref="IMazeConnector"/>.
    /// </summary>
    /// <remarks>
    /// One connector — one WebAssembly runtime, one linear memory, one set of
    /// allocator counters — is shared by every <c>Maze</c> and <c>MazeGame</c>, and
    /// none of the runtimes tolerates two threads inside it at once. Calls do not
    /// only arrive from the thread doing the work: an undisposed <c>Maze</c> or
    /// <c>MazeGame</c> releases its handle from a finalizer, which the garbage
    /// collector runs on its own thread whenever it chooses. Wrapping the connector
    /// rather than locking inside each implementation keeps that guarantee true for
    /// all three (Wasmtime, Wasmer and the native library), and a member added to the
    /// interface later cannot quietly skip it — this class fails to compile until it
    /// is forwarded too. Calls are short, so serialising them costs nothing worth
    /// measuring.
    /// </remarks>
    internal sealed class SynchronizedMazeConnector : IMazeConnector
    {
        private readonly IMazeConnector _inner;
        private readonly Lock _gate = new();

        internal SynchronizedMazeConnector(IMazeConnector inner)
        {
            _inner = inner ?? throw new ArgumentNullException(nameof(inner));
        }

        public UIntPtr NewMaze()
        {
            lock (_gate) { return _inner.NewMaze(); }
        }

        public void FreeMaze(UIntPtr mazePtr)
        {
            lock (_gate) { _inner.FreeMaze(mazePtr); }
        }

        public bool MazeIsEmpty(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeIsEmpty(mazePtr); }
        }

        public void MazeResize(UIntPtr mazePtr, UInt32 newRowCount, UInt32 newColCount)
        {
            lock (_gate) { _inner.MazeResize(mazePtr, newRowCount, newColCount); }
        }

        public void MazeReset(UIntPtr mazePtr)
        {
            lock (_gate) { _inner.MazeReset(mazePtr); }
        }

        public UInt32 MazeGetRowCount(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeGetRowCount(mazePtr); }
        }

        public UInt32 MazeGetColCount(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeGetColCount(mazePtr); }
        }

        public MazeCellType MazeGetCellType(UIntPtr mazePtr, UInt32 row, UInt32 col)
        {
            lock (_gate) { return _inner.MazeGetCellType(mazePtr, row, col); }
        }

        public void MazeSetStartCell(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol)
        {
            lock (_gate) { _inner.MazeSetStartCell(mazePtr, startRow, startCol); }
        }

        public MazePoint MazeGetStartCell(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeGetStartCell(mazePtr); }
        }

        public void MazeSetFinishCell(UIntPtr mazePtr, UInt32 finishRow, UInt32 finishCol)
        {
            lock (_gate) { _inner.MazeSetFinishCell(mazePtr, finishRow, finishCol); }
        }

        public MazePoint MazeGetFinishCell(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeGetFinishCell(mazePtr); }
        }

        public void MazeSetWallCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetWallCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeSetKeyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetKeyCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeSetDoorCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetDoorCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeSetEnemyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetEnemyCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeSetHealthCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetHealthCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeSetTreasureCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeSetTreasureCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeClearCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            lock (_gate) { _inner.MazeClearCells(mazePtr, startRow, startCol, endRow, endCol); }
        }

        public void MazeInsertRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            lock (_gate) { _inner.MazeInsertRows(mazePtr, startRow, count); }
        }

        public void MazeDeleteRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            lock (_gate) { _inner.MazeDeleteRows(mazePtr, startRow, count); }
        }

        public void MazeInsertCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            lock (_gate) { _inner.MazeInsertCols(mazePtr, startCol, count); }
        }

        public void MazeDeleteCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            lock (_gate) { _inner.MazeDeleteCols(mazePtr, startCol, count); }
        }

        public void MazeFromJson(UIntPtr mazePtr, string json)
        {
            lock (_gate) { _inner.MazeFromJson(mazePtr, json); }
        }

        public string MazeToJson(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeToJson(mazePtr); }
        }

        public string? MazeGetCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            lock (_gate) { return _inner.MazeGetCellEntity(mazePtr, row, col); }
        }

        public void MazeSetCellEntity(UIntPtr mazePtr, uint row, uint col, string json)
        {
            lock (_gate) { _inner.MazeSetCellEntity(mazePtr, row, col, json); }
        }

        public void MazeClearCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            lock (_gate) { _inner.MazeClearCellEntity(mazePtr, row, col); }
        }

        public UIntPtr MazeSolve(UIntPtr mazePtr)
        {
            lock (_gate) { return _inner.MazeSolve(mazePtr); }
        }

        public List<MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            lock (_gate) { return _inner.MazeSolutionGetPathPoints(solutionPtr); }
        }

        public void FreeMazeSolution(UIntPtr solutionPtr)
        {
            lock (_gate) { _inner.FreeMazeSolution(solutionPtr); }
        }

        public UInt32 AllocateSizedMemory(UInt32 size)
        {
            lock (_gate) { return _inner.AllocateSizedMemory(size); }
        }

        public void FreeSizedMemory(UInt32 ptr)
        {
            lock (_gate) { _inner.FreeSizedMemory(ptr); }
        }

        public Int64 GetSizedMemoryUsed()
        {
            lock (_gate) { return _inner.GetSizedMemoryUsed(); }
        }

        public Int64 GetNumObjectsAllocated()
        {
            lock (_gate) { return _inner.GetNumObjectsAllocated(); }
        }

        public UIntPtr NewGeneratorOptions(UInt32 rowCount, UInt32 colCount, MazeGenerationAlgorithm algorithm, UInt64 seed)
        {
            lock (_gate) { return _inner.NewGeneratorOptions(rowCount, colCount, algorithm, seed); }
        }

        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            lock (_gate) { _inner.GeneratorOptionsSetStart(optionsPtr, row, col); }
        }

        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            lock (_gate) { _inner.GeneratorOptionsSetFinish(optionsPtr, row, col); }
        }

        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetMinSpineLength(optionsPtr, value); }
        }

        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetMaxRetries(optionsPtr, value); }
        }

        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetBranchFromFinish(optionsPtr, value); }
        }

        public void GeneratorOptionsSetDoorCount(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetDoorCount(optionsPtr, value); }
        }

        public void GeneratorOptionsSetSpareDoors(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetSpareDoors(optionsPtr, value); }
        }

        public void GeneratorOptionsSetSpareKeys(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetSpareKeys(optionsPtr, value); }
        }

        public void GeneratorOptionsSetEnemyCount(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetEnemyCount(optionsPtr, value); }
        }

        public void GeneratorOptionsSetHealthCount(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetHealthCount(optionsPtr, value); }
        }

        public void GeneratorOptionsSetTreasureCount(UIntPtr optionsPtr, UInt32 value)
        {
            lock (_gate) { _inner.GeneratorOptionsSetTreasureCount(optionsPtr, value); }
        }

        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            lock (_gate) { _inner.MazeGenerate(mazePtr, optionsPtr); }
        }

        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            lock (_gate) { _inner.FreeGeneratorOptions(optionsPtr); }
        }

        public UIntPtr NewMazeGame(string definitionJson)
        {
            lock (_gate) { return _inner.NewMazeGame(definitionJson); }
        }

        public void FreeMazeGame(UIntPtr gamePtr)
        {
            lock (_gate) { _inner.FreeMazeGame(gamePtr); }
        }

        public int MazeGameMovePlayer(UIntPtr gamePtr, int dir)
        {
            lock (_gate) { return _inner.MazeGameMovePlayer(gamePtr, dir); }
        }

        public int MazeGamePlayerRow(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGamePlayerRow(gamePtr); }
        }

        public int MazeGamePlayerCol(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGamePlayerCol(gamePtr); }
        }

        public int MazeGamePlayerDirection(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGamePlayerDirection(gamePtr); }
        }

        public int MazeGameIsComplete(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameIsComplete(gamePtr); }
        }

        public int MazeGameIsLost(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameIsLost(gamePtr); }
        }

        public int MazeGameLoseReason(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameLoseReason(gamePtr); }
        }

        public bool MazeGamePickup(UIntPtr gamePtr, out MazeBagItem item)
        {
            lock (_gate) { return _inner.MazeGamePickup(gamePtr, out item); }
        }

        public int MazeGameBagCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameBagCount(gamePtr); }
        }

        public bool MazeGameGetBagItem(UIntPtr gamePtr, int index, out MazeBagItem item)
        {
            lock (_gate) { return _inner.MazeGameGetBagItem(gamePtr, index, out item); }
        }

        public int MazeGameDoorCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameDoorCount(gamePtr); }
        }

        public bool MazeGameGetDoor(UIntPtr gamePtr, int index, out MazeDoor door)
        {
            lock (_gate) { return _inner.MazeGameGetDoor(gamePtr, index, out door); }
        }

        public int MazeGameTick(UIntPtr gamePtr, float dtMs)
        {
            lock (_gate) { return _inner.MazeGameTick(gamePtr, dtMs); }
        }

        public int MazeGameTickEventCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameTickEventCount(gamePtr); }
        }

        public bool MazeGameGetTickEvent(UIntPtr gamePtr, int index, out MazeGameEvent evt)
        {
            lock (_gate) { return _inner.MazeGameGetTickEvent(gamePtr, index, out evt); }
        }

        public int MazeGameKeyCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameKeyCount(gamePtr); }
        }

        public bool MazeGameGetKey(UIntPtr gamePtr, int index, out MazeKey key)
        {
            lock (_gate) { return _inner.MazeGameGetKey(gamePtr, index, out key); }
        }

        public uint MazeGameHp(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameHp(gamePtr); }
        }

        public uint MazeGameMaxHp(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameMaxHp(gamePtr); }
        }

        public int MazeGameEnemyCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameEnemyCount(gamePtr); }
        }

        public bool MazeGameGetEnemy(UIntPtr gamePtr, int index, out MazeEnemy enemy)
        {
            lock (_gate) { return _inner.MazeGameGetEnemy(gamePtr, index, out enemy); }
        }

        public int MazeGameHealthPickupCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameHealthPickupCount(gamePtr); }
        }

        public bool MazeGameGetHealthPickup(UIntPtr gamePtr, int index, out MazeHealthPickup pickup)
        {
            lock (_gate) { return _inner.MazeGameGetHealthPickup(gamePtr, index, out pickup); }
        }

        public int MazeGameTreasureCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameTreasureCount(gamePtr); }
        }

        public bool MazeGameGetTreasure(UIntPtr gamePtr, int index, out MazeTreasure treasure)
        {
            lock (_gate) { return _inner.MazeGameGetTreasure(gamePtr, index, out treasure); }
        }

        public int MazeGameCollectedTreasureCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameCollectedTreasureCount(gamePtr); }
        }

        public bool MazeGameGetCollectedTreasure(UIntPtr gamePtr, int index, out MazeCollectedTreasure collected)
        {
            lock (_gate) { return _inner.MazeGameGetCollectedTreasure(gamePtr, index, out collected); }
        }

        public int MazeGameVisitedCellCount(UIntPtr gamePtr)
        {
            lock (_gate) { return _inner.MazeGameVisitedCellCount(gamePtr); }
        }

        public bool MazeGameGetVisitedCell(UIntPtr gamePtr, int index, out int row, out int col)
        {
            lock (_gate) { return _inner.MazeGameGetVisitedCell(gamePtr, index, out row, out col); }
        }

        public void Dispose()
        {
            lock (_gate) { _inner.Dispose(); }
        }
    }
}
