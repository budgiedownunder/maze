using Xunit;
[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Maze.Interop.Tests
{
    using Maze.Interop;
    using static Maze.Interop.MazeInterop;
    using System;

    /// <summary>
    ///  This base class contains the [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Interop.MazeInterop"/> class
    /// </summary>
    public abstract class MazeInteropTestBase
    {
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns>
        /// <see cref="Maze.Interop.MazeInterop"/> instance</returns>
        protected abstract MazeInterop GetInterop();

        private UIntPtr CreateNewMaze(UInt32 numRows, UInt32 numCols)
        {
            UIntPtr mazePtr = GetInterop().NewMaze();
            if (numRows > 0 || numCols > 0)
            {
                GetInterop().MazeResize(mazePtr, numRows, numCols);
            }
            return mazePtr;
        }
        private void FreeMaze(UIntPtr mazePtr)
        {
            GetInterop().FreeMaze(mazePtr);
        }
        private void FreeMazeGame(UIntPtr gamePtr)
        {
            GetInterop().FreeMazeGame(gamePtr);
        }
        private void AssertRowCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected rowCount to be {expected} but got {actual}");
        }
        private void AssertColCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected colCount to be {expected} but got {actual}");
        }
        private void AssertCellType(MazeCellType actual, MazeCellType expected)
        {
            Assert.True(actual == expected, $"Expected cell type to be '{expected}' but got '{actual}'");
        }
        private void AssertPoint(string context, MazePoint actual, MazePoint expected)
        {
            Assert.True(actual.row == expected.row, $"Expected {context} point row to be '{expected.row}' but got '{actual.row}'");
            Assert.True(actual.col == expected.col, $"Expected {context} point column to be '{expected.col}' but got '{actual.col}'");
        }
        private void AssertStartCell(MazePoint actual, MazePoint expected)
        {
            AssertPoint("start", actual, expected);
        }
        private void AssertFinishCell(MazePoint actual, MazePoint expected)
        {
            AssertPoint("finish", actual, expected);
        }
        private void AssertRangeCellType(UIntPtr mazePtr, UInt32 fromRow, UInt32 fromCol, UInt32 toRow, UInt32 toCol, MazeCellType expected, bool freeMazePtrOnFail)
        {
            MazeInterop interop = GetInterop();
            for (UInt32 row = fromRow; row <= toRow; row++)
            {
                for (UInt32 col = fromCol; col <= toCol; col++)
                {
                    MazeCellType cellType = interop.MazeGetCellType(mazePtr, fromRow, fromCol);
                    if (cellType != expected)
                        FreeMaze(mazePtr);
                    Assert.True(cellType == expected, $"Expected cell type at [{row}, {col}] to be '{expected}' but got '{cellType}'");
                }
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewMaze"/> returns a non-zero pointer value
        /// </summary>
        [Fact]
        public void NewMaze_ReturnsNonZeroPointer()
        {
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            if (mazePtr != UIntPtr.Zero) FreeMaze(mazePtr);
            Assert.True(mazePtr != UIntPtr.Zero);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeIsEmpty"/> returns `true` for a newly created maze
        /// </summary>
        [Fact]
        public void MazeIsEmpty_ShouldReturnTrueForNewMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            bool isEmpty = interop.MazeIsEmpty(mazePtr);
            if (mazePtr != UIntPtr.Zero) FreeMaze(mazePtr);
            Assert.True(isEmpty);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeIsEmpty"/> returns `false` for a maze that has size 
        /// </summary>
        [Fact]
        public void MazeIsEmpty_ShouldReturnFalseForMazeWithSize()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 1);
            bool isEmpty = interop.MazeIsEmpty(mazePtr);
            if (mazePtr != UIntPtr.Zero) FreeMaze(mazePtr);
            Assert.False(isEmpty);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetRowCount"/> returns the expected number of rows for a maze that has size 
        /// </summary>
        [Fact]
        public void MazeGetRowCount_ShouldReturnCorrectNumberRows()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UIntPtr mazePtr = CreateNewMaze(targetRowCount, targetColCount);
            var rowCount = GetInterop().MazeGetRowCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, targetRowCount);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetColCount"/> returns the expected number of columns for a maze that has size 
        /// </summary>
        [Fact]
        public void MazeGetColCount_ShouldReturnCorrectNumberCols()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UIntPtr mazePtr = CreateNewMaze(targetRowCount, targetColCount);
            var colCount = GetInterop().MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertColCount(colCount, targetColCount);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeReset"/> removes all rows and columns in a maze 
        /// </summary>
        [Fact]
        public void MazeReset_ShouldSucceed()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            interop.MazeReset(mazePtr);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, 0);
            AssertColCount(colCount, 0);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeClearCells"/> clears cell content in a maze when provided with a valid target cell range
        /// </summary>
        [Fact]
        public void MazeWasmClear_ShouldSucceed()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            interop.MazeClearCells(mazePtr, 1, 2, 3, 4);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, 10);
            AssertColCount(colCount, 5);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeClearCells"/> fails to clear cell content in a maze when provided with an invalid target start row 
        /// </summary>
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeClearCells(mazePtr, 11, 2, 3, 4);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'from' point [11, 2]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeClearCells"/> fails to clear cell content in a maze when provided with an invalid target start column 
        /// </summary>
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeClearCells(mazePtr, 1, 12, 3, 4);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'from' point [1, 12]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeClearCells"/> fails to clear cell content in a maze when provided with an invalid target end row 
        /// </summary>
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidEndRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeClearCells(mazePtr, 1, 2, 11, 4);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'to' point [11, 4]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeClearCells"/> fails to clear cell content in a maze when provided with an invalid target end column 
        /// </summary>
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidEndCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeClearCells(mazePtr, 1, 2, 3, 11);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'to' point [3, 11]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeResize"/> correctly resizes a maze to the expected row and column counts 
        /// </summary>
        [Fact]
        public void MazeResize_ChangesRowAndColumnCounts()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;

            interop.MazeResize(mazePtr, targetRowCount, targetColCount);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, targetRowCount);
            AssertColCount(colCount, targetColCount);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeInsertRows"/> succeeds for a valid start row 
        /// </summary>
        [Fact]
        public void MazeInsertRows_SucceedsForValidStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeInsertRows(mazePtr, 0, 2);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, 2);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeInsertRows"/> fails for an invalid start row 
        /// </summary>
        [Fact]
        public void MazeInsertRows_FailsForInvalidStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeInsertRows(mazePtr, 1, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start_row' index (1)", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteRows"/> fails for an empty maze 
        /// </summary>
        [Fact]
        public void MazeDeleteRows_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteRows(mazePtr, 1, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteRows"/> fails for an invalid start row 
        /// </summary>
        [Fact]
        public void MazeDeleteRows_FailsForInvalidStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteRows(mazePtr, 1, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start_row' index (1)", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteRows"/> fails if the number of rows requested is too large 
        /// </summary>
        [Fact]
        public void MazeDeleteRows_FailsIfCountTooLarge()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(2, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteRows(mazePtr, 1, 3);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'count' (3) - too large", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteRows"/> succeeds for a valid start row and row count
        /// </summary>
        [Fact]
        public void MazeDeleteRows_SucceedsForValidStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(3, 1);
            interop.MazeDeleteRows(mazePtr, 0, 1);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, 2);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeInsertCols"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeInsertCols_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeInsertCols(mazePtr, 0, 0);
            });
            FreeMaze(mazePtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeInsertCols"/> fails for an invalid start column
        /// </summary>
        [Fact]
        public void MazeInsertCols_FailsForInvalidStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeInsertCols(mazePtr, 2, 1);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start_col' index (2)", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeInsertCols"/> succeeds for a valid start column and column count
        /// </summary>
        [Fact]
        public void MazeInsertCols_SucceedsForValidStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 0);
            interop.MazeInsertCols(mazePtr, 0, 3);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertColCount(colCount, 3);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteCols"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeDeleteCols_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteCols(mazePtr, 1, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteCols"/> fails for an invalid start column
        /// </summary>
        [Fact]
        public void MazeDeleteCols_FailsForInvalidStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteCols(mazePtr, 1, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start_col' index (1)", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteCols"/> fails if the number of columns requested is too large
        /// </summary>
        [Fact]
        public void MazeDeleteCols_FailsIfCountTooLarge()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(2, 2);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeDeleteCols(mazePtr, 1, 3);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'count' (3) - too large", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeDeleteCols"/> succeeds for a valid start column and column count
        /// </summary>
        [Fact]
        public void MazeDeleteCols_SucceedsForValidStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 3);
            interop.MazeDeleteCols(mazePtr, 0, 1);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertColCount(colCount, 2);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetCellType"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeGetCellType_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            MazeCellType cellType = MazeCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeGetCellType(mazePtr, 0, 0);
            });
            FreeMaze(mazePtr);
            Assert.Equal("row index (0) out of bounds", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetCellType"/> fails for an invalid target row
        /// </summary>
        [Fact]
        public void MazeGetCellType_FailsForInvalidRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            MazeCellType cellType = MazeCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeGetCellType(mazePtr, 10, 4);
            });
            FreeMaze(mazePtr);
            Assert.Equal("row index (10) out of bounds", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetCellType"/> fails for an invalid target column
        /// </summary>
        [Fact]
        public void MazeGetCellType_FailsForInvalidCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(10, 5);
            MazeCellType cellType = MazeCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeGetCellType(mazePtr, 9, 5);
            });
            FreeMaze(mazePtr);
            Assert.Equal("column index (5) out of bounds", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetCellType"/> succeeds for a valid cell location
        /// </summary>
        [Fact]
        public void MazeGetCellType_SucceedsForValidCellLocation()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(1, 1);
            MazeCellType cellType = interop.MazeGetCellType(mazePtr, 0, 0);
            FreeMaze(mazePtr);
            AssertCellType(cellType, MazeCellType.Empty);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetStartCell"/> fails for an invalid target row
        /// </summary>
        [Fact]
        public void MazeSetStartCell_FailsForInvalidRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetStartCell(mazePtr, 20, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start' point [20, 2]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetStartCell"/> fails for an invalid target column
        /// </summary>
        [Fact]
        public void MazeSetStartCell_FailsForInvalidCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetStartCell(mazePtr, 1, 10);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'start' point [1, 10]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetStartCell"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeGetStartCell_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            MazePoint? start;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                start = interop.MazeGetStartCell(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no start cell defined", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetStartCell"/> fails if a start cell is not defined
        /// </summary>
        [Fact]
        public void MazeGetStartCell_FailsIfNotDefined()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            MazePoint start;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                start = interop.MazeGetStartCell(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no start cell defined", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetStartCell"/> succeeds if a start cell is defined
        /// </summary>
        [Fact]
        public void MazeGetStartCell_SucceedsIfDefined()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            MazePoint start = new MazePoint();
            interop.MazeSetStartCell(mazePtr, 1, 2);
            start = interop.MazeGetStartCell(mazePtr);
            FreeMaze(mazePtr);
            AssertStartCell(start, new MazePoint() { row = 1, col = 2 });
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetFinishCell"/> fails for an invalid target row
        /// </summary>
        [Fact]
        public void MazeSetFinishCell_FailsForInvalidRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetFinishCell(mazePtr, 20, 2);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'finish' point [20, 2]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetFinishCell"/> fails for an invalid target column
        /// </summary>
        [Fact]
        public void MazeSetFinishCell_FailsForInvalidCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetFinishCell(mazePtr, 1, 10);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'finish' point [1, 10]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetFinishCell"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeGetFinishCell_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            MazePoint? finish;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                finish = interop.MazeGetFinishCell(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no finish cell defined", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetFinishCell"/> fails if a finish cell is not defined
        /// </summary>
        [Fact]
        public void MazeGetFinishCell_FailsIfNotDefined()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            MazePoint finish;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                finish = interop.MazeGetFinishCell(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no finish cell defined", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGetFinishCell"/> succeeds if a finish cell is defined
        /// </summary>
        [Fact]
        public void MazeGetFinishCell_SucceedsIfDefined()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 5);
            MazePoint finish = new MazePoint();
            interop.MazeSetFinishCell(mazePtr, 3, 4);
            finish = interop.MazeGetFinishCell(mazePtr);
            FreeMaze(mazePtr);
            AssertFinishCell(finish, new MazePoint() { row = 3, col = 4 });
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetWallCells"/> fails for an empty maze
        /// </summary>
        [Fact]
        public void MazeSetWallCells_FailsForEmptyMaze()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetWallCells(mazePtr, 0, 0, 0, 0);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'from' point [0, 0]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetWallCells"/> fails for an invalid start location
        /// </summary>
        [Fact]
        public void MazeSetWallCells_FailsForInvalidStartLocation()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 10);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetWallCells(mazePtr, 5, 1, 3, 6);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'from' point [5, 1]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetWallCells"/> fails for an invalid end location
        /// </summary>
        [Fact]
        public void MazeSetWallCells_FailsForInvalidEndLocation()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 10);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeSetWallCells(mazePtr, 0, 0, 5, 6);
            });
            FreeMaze(mazePtr);
            Assert.Equal("invalid 'to' point [5, 6]", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSetWallCells"/> succeeds for a valid cell range
        /// </summary>
        [Fact]
        public void MazeSetWallCells_SucceedsForValidCellRange()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(5, 10);
            interop.MazeSetWallCells(mazePtr, 0, 0, 3, 6);
            AssertRangeCellType(mazePtr, 0, 0, 3, 6, MazeCellType.Wall, true);
            FreeMaze(mazePtr);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeToJson"/> succeeds and produces the expected output
        /// </summary>
        [Fact]
        public void MazeToJson_ShouldSucceed()
        {
            MazeInterop interop = GetInterop();
            var expected = @"{""id"":"""",""name"":"""",""definition"":{""grid"":[]}}";
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var json = interop.MazeToJson(mazePtr);
            FreeMaze(mazePtr);
            Assert.Equal(json, expected);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeFromJson"/> fails when presented with invalid JSON
        /// </summary>
        [Fact]
        public void MazeFromJson_ShouldFail()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeFromJson(mazePtr, "{");
            });
            FreeMaze(mazePtr);
            Assert.Equal("EOF while parsing an object at line 1 column 1", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeFromJson"/> succeeds when presented with valid JSON
        /// </summary>
        [Fact]
        public void MazeFromJson_ShouldSucceed()
        {
            var jsonStr = @"
            {
                ""id"":""test"",
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        [""S"", ""W"", "" "", "" "",  ""W""],
                        ["" "", ""W"", "" "", ""W"", "" ""],
                        ["" "", "" "", "" "", ""W"", ""F""],
                        [""W"", "" "", ""W"", "" "", "" ""],
                        ["" "", "" "", "" "", ""W"", "" ""],
                        [""W"", ""W"", "" "", "" "", "" ""],
                        [""W"", ""W"", "" "", ""W"", "" ""]
                    ]
                }
            }
            ";
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeFromJson(mazePtr, jsonStr);
            var rowCount = interop.MazeGetRowCount(mazePtr);
            var colCount = interop.MazeGetColCount(mazePtr);
            FreeMaze(mazePtr);
            AssertRowCount(rowCount, 7);
            AssertColCount(colCount, 5);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSolve"/> fails if a maze does not contain a start cell and returns an error indicating that it is missing
        /// </summary>
        [Fact]
        public void MazeSolve_ShouldFailWithNoStartCell()
        {
            var jsonStr = @"
            {
                ""id"":""test"",
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        ["" "", ""W""],
                        ["" "", ""F""]
                    ]
                }
            }
            ";
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeFromJson(mazePtr, jsonStr);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                UIntPtr solution = interop.MazeSolve(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no start cell found within maze", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSolve"/> fails if a maze does not contain a finish cell and returns an error indicating that it is missing
        /// </summary>
        [Fact]
        public void MazeSolve_ShouldFailWithNoFinishCell()
        {
            var jsonStr = @"
            {
                ""id"":""test"",
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        [""S"", ""W""],
                        ["" "", "" ""]
                    ]
                }
            }
            ";
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeFromJson(mazePtr, jsonStr);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                UIntPtr solution = interop.MazeSolve(mazePtr);
            });
            FreeMaze(mazePtr);
            Assert.Equal("no finish cell found within maze", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSolve"/> succeeds for a valid maze
        /// </summary>
        [Fact]
        public void MazeSolve_ShouldSucceed()
        {
            var jsonStr = @"
            {
                ""id"":""test"",
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        [""S"", ""W"", "" "", "" "",  ""W""],
                        ["" "", ""W"", "" "", ""W"", "" ""],
                        ["" "", "" "", "" "", ""W"", ""F""],
                        [""W"", "" "", ""W"", "" "", "" ""],
                        ["" "", "" "", "" "", ""W"", "" ""],
                        [""W"", ""W"", "" "", "" "", "" ""],
                        [""W"", ""W"", "" "", ""W"", "" ""]
                    ]
                }
            }
            ";
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeFromJson(mazePtr, jsonStr);
            UIntPtr solution = interop.MazeSolve(mazePtr);
            FreeMaze(mazePtr);
            interop.FreeMazeSolution(solution);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeSolutionGetPathPoints"/> succeeds when provided with a valid solution and produces the expected path
        /// </summary>
        [Fact]
        public void MazeSolutionGetPathPoints_ShouldSucceed()
        {
            var jsonStr = @"
                {
                    ""id"":""test"",
                    ""name"":""test"",
                    ""definition"": {
                        ""grid"":[
                            [""S"", ""W"", "" "", "" "",  ""W""],
                            ["" "", ""W"", "" "", ""W"", "" ""],
                            ["" "", "" "", "" "", ""W"", ""F""],
                            [""W"", "" "", ""W"", "" "", "" ""],
                            ["" "", "" "", "" "", ""W"", "" ""],
                            [""W"", ""W"", "" "", "" "", "" ""],
                            [""W"", ""W"", "" "", ""W"", "" ""]
                        ]
                    }
                }
                ";
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            interop.MazeFromJson(mazePtr, jsonStr);
            UIntPtr solution = interop.MazeSolve(mazePtr);
            List<MazePoint> solutionPath = interop.MazeSolutionGetPathPoints(solution);
            FreeMaze(mazePtr);
            interop.FreeMazeSolution(solution);
            Int32 numPointsExpected = 13;
            Assert.True(solutionPath.Count == numPointsExpected, $"Expected {numPointsExpected} points in the solution but got '{solutionPath.Count}'");
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.GetSizedMemoryUsed"/> succeeds and returns zero when expected
        /// </summary>
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldSucceedAndBeZero()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var bytesUsed = interop.GetSizedMemoryUsed();
            FreeMaze(mazePtr);
            var bytesUsedExpected = 0;
            Assert.True(bytesUsed == bytesUsedExpected, $"Expected {bytesUsedExpected} bytes used but got '{bytesUsed}'");
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.GetSizedMemoryUsed"/> succeeds and returns the expected non-zero value when memory is not released
        /// </summary>
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldBeNonZeroAfterAllocate()
        {
            MazeInterop interop = GetInterop();
            UInt32 sizeRequest = 100;
            UInt32 sizedMemoryUsedExpected = sizeRequest + 4;
            var sizedMemoryPtr = interop.AllocateSizedMemory(sizeRequest);
            var bytesUsed = interop.GetSizedMemoryUsed();
            Assert.True(bytesUsed == sizedMemoryUsedExpected, $"Expected {sizedMemoryUsedExpected} used but got '{bytesUsed}'");
            interop.FreeSizedMemory(sizedMemoryPtr);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.GetSizedMemoryUsed"/> succeeds and returns zero after allocating and then freeing sized memory
        /// </summary>
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldBeZeroAfterAllocateAndFree()
        {
            MazeInterop interop = GetInterop();
            var sizedMemoryPtr = interop.AllocateSizedMemory(100);
            Assert.True(sizedMemoryPtr != 0, $"Expected non-zero memory pointer but got '{sizedMemoryPtr}'");
            interop.FreeSizedMemory(sizedMemoryPtr);
            var bytesUsed = interop.GetSizedMemoryUsed();
            Assert.True(bytesUsed == 0, $"Expected zero bytes used but got '{bytesUsed}'");
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.GetNumObjectsAllocated"/> succeeds and returns the expected object count when objects are not freed
        /// </summary>
        [Fact]
        public void MazeWasmGetNumObjectsAllocated_ShouldSucceedAndBeNonZero()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = CreateNewMaze(0, 0);
            var numObjects = interop.GetNumObjectsAllocated();
            FreeMaze(mazePtr);
            var numObjectsExpected = 1;
            Assert.True(numObjects == numObjectsExpected, $"Expected {numObjectsExpected} but got '{numObjects}'");
        }

        // --- Generation tests ---

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewGeneratorOptions"/> succeeds and returns the expected dimensions when provided with valid parameters
        /// </summary>
        [Fact]
        public void MazeGenerate_ShouldSucceedAndReturnCorrectDimensions()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = interop.NewMaze();
            UIntPtr optionsPtr = interop.NewGeneratorOptions(7, 5, MazeGenerationAlgorithm.RecursiveBacktracking, 42);
            try
            {
                interop.MazeGenerate(mazePtr, optionsPtr);
                AssertRowCount(interop.MazeGetRowCount(mazePtr), 7);
                AssertColCount(interop.MazeGetColCount(mazePtr), 5);
            }
            finally
            {
                interop.FreeGeneratorOptions(optionsPtr);
                interop.FreeMaze(mazePtr);
            }
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewGeneratorOptions"/> succeeds and returns the expected output when provided with a specific seed
        /// </summary>
        [Fact]
        public void MazeGenerate_SameSeedProducesDeterministicOutput()
        {
            MazeInterop interop = GetInterop();
            UIntPtr maze1Ptr = interop.NewMaze();
            UIntPtr maze2Ptr = interop.NewMaze();
            UIntPtr opts1Ptr = interop.NewGeneratorOptions(11, 11, MazeGenerationAlgorithm.RecursiveBacktracking, 999);
            UIntPtr opts2Ptr = interop.NewGeneratorOptions(11, 11, MazeGenerationAlgorithm.RecursiveBacktracking, 999);
            try
            {
                interop.MazeGenerate(maze1Ptr, opts1Ptr);
                interop.MazeGenerate(maze2Ptr, opts2Ptr);
                string json1 = interop.MazeToJson(maze1Ptr);
                string json2 = interop.MazeToJson(maze2Ptr);
                Assert.Equal(json1, json2);
            }
            finally
            {
                interop.FreeGeneratorOptions(opts1Ptr);
                interop.FreeGeneratorOptions(opts2Ptr);
                interop.FreeMaze(maze1Ptr);
                interop.FreeMaze(maze2Ptr);
            }
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewGeneratorOptions"/> succeeds and returns the expected dimensions
        /// </summary>
        [Fact]
        public void MazeGenerate_WithExplicitStartFinishAndSpine_ShouldSucceedAndReturnCorrectDimensions()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = interop.NewMaze();
            UIntPtr optionsPtr = interop.NewGeneratorOptions(9, 7, MazeGenerationAlgorithm.RecursiveBacktracking, 1);
            try
            {
                interop.GeneratorOptionsSetStart(optionsPtr, 0, 0);
                interop.GeneratorOptionsSetFinish(optionsPtr, 8, 6);
                interop.GeneratorOptionsSetMinSpineLength(optionsPtr, 5);
                interop.GeneratorOptionsSetMaxRetries(optionsPtr, 50);
                interop.MazeGenerate(mazePtr, optionsPtr);
                AssertRowCount(interop.MazeGetRowCount(mazePtr), 9);
                AssertColCount(interop.MazeGetColCount(mazePtr), 7);
            }
            finally
            {
                interop.FreeGeneratorOptions(optionsPtr);
                interop.FreeMaze(mazePtr);
            }
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewGeneratorOptions"/> fails if an invalid row count is specified
        /// </summary>
        [Fact]
        public void MazeGenerate_WithInvalidRowCount_ShouldThrow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr mazePtr = interop.NewMaze();
            UIntPtr optionsPtr = interop.NewGeneratorOptions(2, 5, MazeGenerationAlgorithm.RecursiveBacktracking, 1);
            try
            {
                var ex = Assert.Throws<Exception>(() => interop.MazeGenerate(mazePtr, optionsPtr));
                Assert.Contains("row_count", ex.Message);
            }
            finally
            {
                interop.FreeGeneratorOptions(optionsPtr);
                interop.FreeMaze(mazePtr);
            }
        }

        // --- MazeGame tests ---

        // Maze layout used for most game tests: 1 row, 3 cols — S[0,0]  [0,1]  F[0,2]
        private const string SimpleGameJson = """{"grid":[["S"," ","F"]]}""";
        // Maze with wall blocking movement: S[0,0]  W[0,1]  F[0,2]
        private const string WalledGameJson = """{"grid":[["S","W","F"]]}""";

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewMazeGame"/> returns a non-zero pointer for valid JSON
        /// </summary>
        [Fact]
        public void NewMazeGame_ShouldReturnNonZeroPointer()
        {
            UIntPtr gamePtr = GetInterop().NewMazeGame(SimpleGameJson);
            if (gamePtr != UIntPtr.Zero) FreeMazeGame(gamePtr);
            Assert.True(gamePtr != UIntPtr.Zero);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.NewMazeGame"/> throws for invalid JSON
        /// </summary>
        [Fact]
        public void NewMazeGame_ShouldThrow_ForInvalidJson()
        {
            var exception = Assert.ThrowsAny<Exception>(() => GetInterop().NewMazeGame("{"));
            Assert.Equal("Failed to create maze game session — invalid JSON or maze has no start cell", exception.Message);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.FreeMazeGame"/> succeeds without throwing
        /// </summary>
        [Fact]
        public void FreeMazeGame_ShouldSucceed()
        {
            UIntPtr gamePtr = GetInterop().NewMazeGame(SimpleGameJson);
            FreeMazeGame(gamePtr);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameMovePlayer"/> returns 0 when direction is None (0)
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturn0_WhenDirectionIsNone()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int result = interop.MazeGameMovePlayer(gamePtr, 0);
            FreeMazeGame(gamePtr);
            Assert.Equal(0, result);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameMovePlayer"/> returns 1 (Moved) for a clear path
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturn1_WhenMoved()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int result = interop.MazeGameMovePlayer(gamePtr, 4); // Right
            FreeMazeGame(gamePtr);
            Assert.Equal(1, result);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameMovePlayer"/> returns 2 (Blocked) when a wall is ahead
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturn2_WhenBlocked()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(WalledGameJson);
            int result = interop.MazeGameMovePlayer(gamePtr, 4); // Right → wall
            FreeMazeGame(gamePtr);
            Assert.Equal(2, result);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameMovePlayer"/> returns 3 (Complete) when the finish cell is reached
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturn3_WhenComplete()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            interop.MazeGameMovePlayer(gamePtr, 4); // Right → [0,1]
            int result = interop.MazeGameMovePlayer(gamePtr, 4); // Right → [0,2] = F
            FreeMazeGame(gamePtr);
            Assert.Equal(3, result);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGamePlayerRow"/> returns the start row initially
        /// </summary>
        [Fact]
        public void MazeGamePlayerRow_ShouldReturnStartRow()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int row = interop.MazeGamePlayerRow(gamePtr);
            FreeMazeGame(gamePtr);
            Assert.Equal(0, row);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGamePlayerCol"/> returns the start column initially
        /// </summary>
        [Fact]
        public void MazeGamePlayerCol_ShouldReturnStartCol()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int col = interop.MazeGamePlayerCol(gamePtr);
            FreeMazeGame(gamePtr);
            Assert.Equal(0, col);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGamePlayerDirection"/> returns 0 (None) before the first move
        /// </summary>
        [Fact]
        public void MazeGamePlayerDirection_ShouldReturn0_Initially()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int dir = interop.MazeGamePlayerDirection(gamePtr);
            FreeMazeGame(gamePtr);
            Assert.Equal(0, dir);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameIsComplete"/> returns 0 before the finish cell is reached
        /// </summary>
        [Fact]
        public void MazeGameIsComplete_ShouldReturn0_Initially()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int complete = interop.MazeGameIsComplete(gamePtr);
            FreeMazeGame(gamePtr);
            Assert.Equal(0, complete);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameVisitedCellCount"/> returns 1 (start cell) before any moves
        /// </summary>
        [Fact]
        public void MazeGameVisitedCellCount_ShouldBe1_Initially()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            int count = interop.MazeGameVisitedCellCount(gamePtr);
            FreeMazeGame(gamePtr);
            Assert.Equal(1, count);
        }

        /// <summary>
        /// Confirms that <see cref="Maze.Interop.MazeInterop.MazeGameGetVisitedCell"/> returns the start cell at index 0
        /// </summary>
        [Fact]
        public void MazeGameGetVisitedCell_ShouldReturnStartCell()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            bool ok = interop.MazeGameGetVisitedCell(gamePtr, 0, out int row, out int col);
            FreeMazeGame(gamePtr);
            Assert.True(ok);
            Assert.Equal(0, row);
            Assert.Equal(0, col);
        }

        /// <summary>
        /// Confirms that <see cref="MazeInterop.MazeGameGetVisitedCell"/> returns false for an out-of-range index
        /// </summary>
        [Fact]
        public void MazeGameGetVisitedCell_ShouldReturnFalse_ForOutOfRangeIndex()
        {
            MazeInterop interop = GetInterop();
            UIntPtr gamePtr = interop.NewMazeGame(SimpleGameJson);
            bool ok = interop.MazeGameGetVisitedCell(gamePtr, 99, out int _, out int _);
            FreeMazeGame(gamePtr);
            Assert.False(ok);
        }
    }
    /// <summary>
    ///  This class contains the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
    ///  tests for the <see cref="Maze.Interop.MazeInterop"/> class, where the interop is initialized using the default WebAssembly file loader mechanism
    /// </summary>
    public class MazeInteropWasmtimeTest : MazeInteropTestBase
    {
        private readonly MazeInterop _interop = null!;
        /// <summary>
        ///  Constructor for [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
        ///  tests, where the interop is initialized using the default WebAssembly file loader mechanism
        /// </summary>
        public MazeInteropWasmtimeTest()
        {
            _interop = MazeInterop.GetInstance(ConnectionType.Wasmtime, true);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Interop.MazeInterop"/> instance</returns>
        protected override MazeInterop GetInterop()
        {
            return _interop;
        }
    }
    /// <summary>
    ///  This class contains the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
    ///  tests for the <see cref="Maze.Interop.MazeInterop"/> class, where the interop is initialized using WebAssembly bytes passed to it in the constructor
    /// </summary>
    public class MazeInteropWasmtimeTestFromBytes : MazeInteropTestBase
    {
        private readonly MazeInterop _interop;
        /// <summary>
        ///  Constructor for the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
        ///  tests, where the interop is initialized from WebAssembly bytes supplied by the caller
        /// </summary>
        public MazeInteropWasmtimeTestFromBytes()
        {
            byte[] wasmBytes = System.IO.File.ReadAllBytes(MazeInterop.GetWasmPath());
            _interop = MazeInterop.GetInstance(ConnectionType.Wasmtime, true, wasmBytes);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Interop.MazeInterop"/> instance</returns>
        protected override MazeInterop GetInterop()
        {
            return _interop;
        }
    }
#if WINDOWS
    /// <summary>
    ///  This class contains the [Wasmer](https://wasmer.io/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests 
    ///  for the <see cref="Maze.Interop.MazeInterop"/> class, where the interop is initialized using the default WebAssembly file loader mechanism
    /// </summary>
    public class MazeInteropWasmerTest : MazeInteropTestBase
    {
        private readonly MazeInterop _interop;
        /// <summary>
        ///  Constructor for the [Wasmer](https://wasmer.io/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit
        ///  tests, where the interop is initialized using the default WebAssembly file loader mechanism
        /// </summary>
        public MazeInteropWasmerTest()
        {
            _interop = MazeInterop.GetInstance(ConnectionType.Wasmer, true);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Interop.MazeInterop"/> instance</returns>
        protected override MazeInterop GetInterop()
        {
            return _interop;
        }
    }
    /// <summary>
    ///  This class contains the [Wasmer](https://wasmer.io/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests 
    ///  for the <see cref="Maze.Interop.MazeInterop"/> class, where the interop is initialized using WebAssembly bytes passed to it in the constructor.
    /// </summary>
    public class MazeInteropWasmerTestFromBytes : MazeInteropTestBase
    {
        private readonly MazeInterop _interop;
        /// <summary>
        ///  This class contains the [Wasmer](https://wasmer.io/) <see cref="Maze.Interop.MazeInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit
        ///  tests, where the interop is initialized using WebAssembly bytes passed to it in the constructor.
        /// </summary>
        public MazeInteropWasmerTestFromBytes()
        {
            byte[] wasmBytes = System.IO.File.ReadAllBytes(MazeInterop.GetWasmPath());
            _interop = MazeInterop.GetInstance(ConnectionType.Wasmer, true, wasmBytes);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Interop.MazeInterop"/> instance</returns>
        protected override MazeInterop GetInterop()
        {
            return _interop;
        }
    }
#endif
#if IOS
    /// <summary>
    ///  This class contains the <see cref="Maze.Interop.MazeInterop.ConnectionType.Native"/> unit tests
    ///  for the <see cref="Maze.Interop.MazeInterop"/> class. Inherits the full base test suite.
    ///  Run manually on iOS simulator or device only — cannot run in CI.
    /// </summary>
    public class MazeInteropNativeConnectorTest : MazeInteropTestBase
    {
        /// <summary>
        /// Returns the <see cref="Maze.Interop.MazeInterop"/> instance backed by <c>MazeNativeConnector</c>
        /// </summary>
        protected override MazeInterop GetInterop() =>
            MazeInterop.GetInstance(MazeInterop.ConnectionType.Native, createNew: true);
    }
#endif
}
