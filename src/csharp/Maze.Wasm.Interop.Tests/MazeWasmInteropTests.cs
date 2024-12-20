using Xunit;
[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Maze.Wasm.Interop.Tests
{
    using Maze.Wasm.Interop;
    using static Maze.Wasm.Interop.MazeWasmInterop;
    using System;

    /// <summary>
    ///  This base class contains the [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> class
    /// </summary>
    public abstract class MazeWasmInteropTestBase
    {
        /// <summary>
        /// Returns the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns>
        /// <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance</returns>
        protected abstract MazeWasmInterop GetInterop();

        private UInt32 CreateNewMazeWasm(UInt32 numRows, UInt32 numCols)
        {
            UInt32 mazeWasmPtr = GetInterop().NewMazeWasm();
            if (numRows > 0 || numCols > 0)
            {
                GetInterop().MazeWasmResize(mazeWasmPtr, numRows, numCols);
            }
            return mazeWasmPtr;
        }
        private void FreeMazeWasm(UInt32 mazeWasmPtr)
        {
            GetInterop().FreeMazeWasm(mazeWasmPtr);
        }
        private void AssertRowCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected rowCount to be {expected} but got {actual}");
        }
        private void AssertColCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected colCount to be {expected} but got {actual}");
        }
        private void AssertCellType(MazeWasmCellType actual, MazeWasmCellType expected)
        {
            Assert.True(actual == expected, $"Expected cell type to be '{expected}' but got '{actual}'");
        }
        private void AssertPoint(string context, MazeWasmPoint actual, MazeWasmPoint expected)
        {
            Assert.True(actual.row == expected.row, $"Expected {context} point row to be '{expected.row}' but got '{actual.row}'");
            Assert.True(actual.col == expected.col, $"Expected {context} point column to be '{expected.col}' but got '{actual.col}'");
        }
        private void AssertStartCell(MazeWasmPoint actual, MazeWasmPoint expected)
        {
            AssertPoint("start", actual, expected);
        }
        private void AssertFinishCell(MazeWasmPoint actual, MazeWasmPoint expected)
        {
            AssertPoint("finish", actual, expected);
        }
        private void AssertRangeCellType(UInt32 mazeWasmPtr, UInt32 fromRow, UInt32 fromCol, UInt32 toRow, UInt32 toCol, MazeWasmCellType expected, bool freeMazePtrOnFail)
        {
            MazeWasmInterop interop = GetInterop();
            for (UInt32 row = fromRow; row <= toRow; row++)
            {
                for (UInt32 col = fromCol; col <= toCol; col++)
                {
                    MazeWasmCellType cellType = interop.MazeWasmGetCellType(mazeWasmPtr, fromRow, fromCol);
                    if (cellType != expected)
                        FreeMazeWasm(mazeWasmPtr);
                    Assert.True(cellType == expected, $"Expected cell type at [{row}, {col}] to be '{expected}' but got '{cellType}'");
                }
            }
        }
        [Fact]
        public void NewMazeWasm_ReturnsNonZeroPointer()
        {
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            if (mazeWasmPtr != 0) FreeMazeWasm(mazeWasmPtr);
            Assert.True(mazeWasmPtr != 0);
        }
        [Fact]
        public void MazeWasmIsEmpty_ShouldReturnTrueForNewMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            bool isEmpty = interop.MazeWasmIsEmpty(mazeWasmPtr);
            if (mazeWasmPtr != 0) FreeMazeWasm(mazeWasmPtr);
            Assert.True(isEmpty);
        }
        [Fact]
        public void MazeWasmIsEmpty_ShouldReturnFalseForMazeWithSize()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 1);
            bool isEmpty = interop.MazeWasmIsEmpty(mazeWasmPtr);
            if (mazeWasmPtr != 0) FreeMazeWasm(mazeWasmPtr);
            Assert.False(isEmpty);
        }
        [Fact]
        public void MazeWasmGetRowCount_ShouldReturnCorrectNumberRows()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UInt32 mazeWasmPtr = CreateNewMazeWasm(targetRowCount, targetColCount);
            var rowCount = GetInterop().MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, targetRowCount);
        }
        [Fact]
        public void MazeWasmGetColCount_ShouldReturnCorrectNumberCols()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UInt32 mazeWasmPtr = CreateNewMazeWasm(targetRowCount, targetColCount);
            var colCount = GetInterop().MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, targetColCount);
        }
        [Fact]
        public void MazeWasmReset_ShouldSucceed()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            interop.MazeWasmReset(mazeWasmPtr);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 0);
            AssertColCount(colCount, 0);
        }
        [Fact]
        public void MazeWasmClear_ShouldSucceed()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            interop.MazeWasmClearCells(mazeWasmPtr, 1, 2, 3, 4);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 10);
            AssertColCount(colCount, 5);
        }
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidStartRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmClearCells(mazeWasmPtr, 11, 2, 3, 4);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'from' point [11, 2]", exception.Message);
        }
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidStartCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmClearCells(mazeWasmPtr, 1, 12, 3, 4);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'from' point [1, 12]", exception.Message);
        }
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidEndRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmClearCells(mazeWasmPtr, 1, 2, 11, 4);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'to' point [11, 4]", exception.Message);
        }
        [Fact]
        public void MazeWasmClear_ShouldFailForInvalidEndCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmClearCells(mazeWasmPtr, 1, 2, 3, 11);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'to' point [3, 11]", exception.Message);
        }
        [Fact]
        public void MazeWasmResize_ChangesRowAndColumnCounts()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;

            interop.MazeWasmResize(mazeWasmPtr, targetRowCount, targetColCount);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, targetRowCount);
            AssertColCount(colCount, targetColCount);
        }
        [Fact]
        public void MazeWasmInsertRows_SucceedsForValidStartRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmInsertRows(mazeWasmPtr, 0, 2);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 2);
        }
        [Fact]
        public void MazeWasmInsertRows_FailsForInvalidStartRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmInsertRows(mazeWasmPtr, 1, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start_row' index (1)", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteRows_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteRows(mazeWasmPtr, 1, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteRows_FailsForInvalidStartRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteRows(mazeWasmPtr, 1, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start_row' index (1)", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteRows_FailsIfCountTooLarge()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(2, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteRows(mazeWasmPtr, 1, 3);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'count' (3) - too large", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteRows_SucceedsForValidStartRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(3, 1);
            interop.MazeWasmDeleteRows(mazeWasmPtr, 0, 1);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 2);
        }
        [Fact]
        public void MazeWasmInsertCols_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmInsertCols(mazeWasmPtr, 0, 0);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        [Fact]
        public void MazeWasmInsertCols_FailsForInvalidStartCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmInsertCols(mazeWasmPtr, 2, 1);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start_col' index (2)", exception.Message);
        }
        [Fact]
        public void MazeWasmInsertCols_SucceedsForValidStartCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 0);
            interop.MazeWasmInsertCols(mazeWasmPtr, 0, 3);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, 3);
        }
        [Fact]
        public void MazeWasmDeleteCols_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteCols(mazeWasmPtr, 1, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("definition is empty", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteCols_FailsForInvalidStartCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 1);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteCols(mazeWasmPtr, 1, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start_col' index (1)", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteCols_FailsIfCountTooLarge()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(2, 2);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmDeleteCols(mazeWasmPtr, 1, 3);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'count' (3) - too large", exception.Message);
        }
        [Fact]
        public void MazeWasmDeleteCols_SucceedsForValidStartCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 3);
            interop.MazeWasmDeleteCols(mazeWasmPtr, 0, 1);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, 2);
        }
        [Fact]
        public void MazeWasmGetCellType_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            MazeWasmCellType cellType = MazeWasmCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeWasmGetCellType(mazeWasmPtr, 0, 0);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("row index (0) out of bounds", exception.Message);
        }
        [Fact]
        public void MazeWasmGetCellType_FailsForInvalidRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            MazeWasmCellType cellType = MazeWasmCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeWasmGetCellType(mazeWasmPtr, 10, 4);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("row index (10) out of bounds", exception.Message);
        }
        [Fact]
        public void MazeWasmGetCellType_FailsForInvalidCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            MazeWasmCellType cellType = MazeWasmCellType.Empty;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                cellType = interop.MazeWasmGetCellType(mazeWasmPtr, 9, 5);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("column index (5) out of bounds", exception.Message);
        }
        [Fact]
        public void MazeWasmGetCellType_SucceedsForValidCellLocation()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 1);
            MazeWasmCellType cellType = interop.MazeWasmGetCellType(mazeWasmPtr, 0, 0);
            FreeMazeWasm(mazeWasmPtr);
            AssertCellType(cellType, MazeWasmCellType.Empty);
        }
        [Fact]
        public void MazeWasmSetStartCell_FailsForInvalidRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetStartCell(mazeWasmPtr, 20, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start' point [20, 2]", exception.Message);
        }
        [Fact]
        public void MazeWasmSetStartCell_FailsForInvalidCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetStartCell(mazeWasmPtr, 1, 10);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'start' point [1, 10]", exception.Message);
        }
        [Fact]
        public void MazeWasmGetStartCell_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            MazeWasmPoint? start;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                start = interop.MazeWasmGetStartCell(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no start cell defined", exception.Message);
        }
        [Fact]
        public void MazeWasmGetStartCell_FailsIfNotDefined()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            MazeWasmPoint start;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                start = interop.MazeWasmGetStartCell(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no start cell defined", exception.Message);
        }
        [Fact]
        public void MazeWasmGetStartCell_SucceedsIfDefined()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            MazeWasmPoint start = new MazeWasmPoint();
            interop.MazeWasmSetStartCell(mazeWasmPtr, 1, 2);
            start = interop.MazeWasmGetStartCell(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertStartCell(start, new MazeWasmPoint() { row = 1, col = 2 });
        }
        [Fact]
        public void MazeWasmSetFinishCell_FailsForInvalidRow()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetFinishCell(mazeWasmPtr, 20, 2);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'finish' point [20, 2]", exception.Message);
        }
        [Fact]
        public void MazeWasmSetFinishCell_FailsForInvalidCol()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetFinishCell(mazeWasmPtr, 1, 10);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'finish' point [1, 10]", exception.Message);
        }
        [Fact]
        public void MazeWasmGetFinishCell_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            MazeWasmPoint? finish;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                finish = interop.MazeWasmGetFinishCell(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no finish cell defined", exception.Message);
        }
        [Fact]
        public void MazeWasmGetFinishCell_FailsIfNotDefined()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            MazeWasmPoint finish;
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                finish = interop.MazeWasmGetFinishCell(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no finish cell defined", exception.Message);
        }
        [Fact]
        public void MazeWasmGetFinishCell_SucceedsIfDefined()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            MazeWasmPoint finish = new MazeWasmPoint();
            interop.MazeWasmSetFinishCell(mazeWasmPtr, 3, 4);
            finish = interop.MazeWasmGetFinishCell(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertFinishCell(finish, new MazeWasmPoint() { row = 3, col = 4 });
        }
        [Fact]
        public void MazeWasmSetWallCells_FailsForEmptyMaze()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetWallCells(mazeWasmPtr, 0, 0, 0, 0);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'from' point [0, 0]", exception.Message);
        }
        [Fact]
        public void MazeWasmSetWallCells_FailsForInvalidFromLocation()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 10);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetWallCells(mazeWasmPtr, 5, 1, 3, 6);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'from' point [5, 1]", exception.Message);
        }
        [Fact]
        public void MazeWasmSetWallCells_FailsForInvalidToLocation()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 10);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmSetWallCells(mazeWasmPtr, 0, 0, 5, 6);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("invalid 'to' point [5, 6]", exception.Message);
        }
        [Fact]
        public void MazeWasmSetWallCells_SucceedsForValidCellRange()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 10);
            interop.MazeWasmSetWallCells(mazeWasmPtr, 0, 0, 3, 6);
            AssertRangeCellType(mazeWasmPtr, 0, 0, 3, 6, MazeWasmCellType.Wall, true);
            FreeMazeWasm(mazeWasmPtr);
        }
        [Fact]
        public void MazeWasmToJson_ShouldSucceed()
        {
            MazeWasmInterop interop = GetInterop();
            var expected = @"{""id"":"""",""name"":"""",""definition"":{""grid"":[]}}";
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var json = interop.MazeWasmToJson(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal(json, expected);
        }
        [Fact]
        public void MazeWasmFromJson_ShouldFail()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                interop.MazeWasmFromJson(mazeWasmPtr, "{");
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("EOF while parsing an object at line 1 column 1", exception.Message);
        }
        [Fact]
        public void MazeWasmFromJson_ShouldSucceed()
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
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmFromJson(mazeWasmPtr, jsonStr);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 7);
            AssertColCount(colCount, 5);
        }
        [Fact]
        public void MazeWasmSolve_ShouldFailWithNoStartCell()
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
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmFromJson(mazeWasmPtr, jsonStr);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                UInt32 solution = interop.MazeWasmSolve(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no start cell found within maze", exception.Message);
        }
        [Fact]
        public void MazeWasmSolve_ShouldFailWithNoFinishCell()
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
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmFromJson(mazeWasmPtr, jsonStr);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                UInt32 solution = interop.MazeWasmSolve(mazeWasmPtr);
            });
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal("no finish cell found within maze", exception.Message);
        }
        [Fact]
        public void MazeWasmSolve_ShouldSucceed()
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
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmFromJson(mazeWasmPtr, jsonStr);
            UInt32 solution = interop.MazeWasmSolve(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            interop.FreeMazeWasmSolution(solution);
        }
        [Fact]
        public void MazeWasmSolutionGetPathPoints_ShouldSucceed()
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
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmFromJson(mazeWasmPtr, jsonStr);
            UInt32 solution = interop.MazeWasmSolve(mazeWasmPtr);
            List<MazeWasmPoint> solutionPath = interop.MazeWasmSolutionGetPathPoints(solution);
            FreeMazeWasm(mazeWasmPtr);
            interop.FreeMazeWasmSolution(solution);
            Int32 numPointsExpected = 13;
            Assert.True(solutionPath.Count == numPointsExpected, $"Expected {numPointsExpected} points in the solution but got '{solutionPath.Count}'");
        }
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldSucceedAndBeZero()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var bytesUsed = interop.GetSizedMemoryUsed();
            FreeMazeWasm(mazeWasmPtr);
            var bytesUsedExpected = 0;
            Assert.True(bytesUsed == bytesUsedExpected, $"Expected {bytesUsedExpected} bytes used but got '{bytesUsed}'");
        }
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldBeNonZeroAfterAllocate()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 sizeRequest = 100;
            UInt32 sizedMemoryUsedExpected = sizeRequest + 4;
            var sizedMemoryPtr = interop.AllocateSizedMemory(sizeRequest);
            var bytesUsed = interop.GetSizedMemoryUsed();
            Assert.True(bytesUsed == sizedMemoryUsedExpected, $"Expected {sizedMemoryUsedExpected} used but got '{bytesUsed}'");
            interop.FreeSizedMemory(sizedMemoryPtr);
        }
        [Fact]
        public void MazeWasmGetSizedMemoryUsed_ShouldBeZeroAfterAllocateAndFree()
        {
            MazeWasmInterop interop = GetInterop();
            var sizedMemoryPtr = interop.AllocateSizedMemory(100);
            Assert.True(sizedMemoryPtr != 0, $"Expected non-zero memory pointer but got '{sizedMemoryPtr}'");
            interop.FreeSizedMemory(sizedMemoryPtr);
            var bytesUsed = interop.GetSizedMemoryUsed();
            Assert.True(bytesUsed == 0, $"Expected zero bytes used but got '{bytesUsed}'");
        }
        [Fact]
        public void MazeWasmGetNumObjectsAllocated_ShouldSucceedAndBeNonZero()
        {
            MazeWasmInterop interop = GetInterop();
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var numObjects = interop.GetNumObjectsAllocated();
            FreeMazeWasm(mazeWasmPtr);
            var numObjectsExpected = 1;
            Assert.True(numObjects == numObjectsExpected, $"Expected {numObjectsExpected} but got '{numObjects}'");
        }
    }
    /// <summary>
    ///  This class contains the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
    ///  tests for the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> class, where the interop is initialized using the default WebAssembly file loader mechanism.
    /// </summary>
    public class MazeWasmInteropWasmtimeTest : MazeWasmInteropTestBase
    {
        private readonly MazeWasmInterop _interop = null!;

        public MazeWasmInteropWasmtimeTest()
        {
            _interop = MazeWasmInterop.GetInstance(ConnectionType.Wasmtime, true);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance</returns>
        protected override MazeWasmInterop GetInterop()
        {
            return _interop;
        }
    }
    /// <summary>
    ///  This class contains the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit 
    ///  tests for the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> class, where the interop is initialized using WebAssembly bytes passed to it in the constructor.
    /// </summary>
    public class MazeWasmInteropWasmtimeTestFromBytes : MazeWasmInteropTestBase
    {
        private readonly MazeWasmInterop _interop;

        public MazeWasmInteropWasmtimeTestFromBytes()
        {
            byte[] wasmBytes = System.IO.File.ReadAllBytes(MazeWasmInterop.GetWasmPath());
            _interop = MazeWasmInterop.GetInstance(ConnectionType.Wasmtime, true, wasmBytes);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance</returns>
        protected override MazeWasmInterop GetInterop()
        {
            return _interop;
        }
    }
#if WINDOWS
    /// <summary>
    ///  This class contains the [Wasmer](https://wasmer.io/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests 
    ///  for the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> class, where the interop is initialized using the default WebAssembly file loader mechanism.
    /// </summary>
    public class MazeWasmInteropWasmerTest : MazeWasmInteropTestBase
    {
        private readonly MazeWasmInterop _interop;

        public MazeWasmInteropWasmerTest()
        {
            _interop = MazeWasmInterop.GetInstance(ConnectionType.Wasmer, true);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance</returns>
        protected override MazeWasmInterop GetInterop()
        {
            return _interop;
        }
    }
    /// <summary>
    ///  This class contains the [Wasmer](https://wasmer.io/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests 
    ///  for the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> class, where the interop is initialized using WebAssembly bytes passed to it in the constructor.
    /// </summary>
    public class MazeWasmInteropWasmerTestFromBytes : MazeWasmInteropTestBase
    {
        private readonly MazeWasmInterop _interop;

        public MazeWasmInteropWasmerTestFromBytes()
        {
            byte[] wasmBytes = System.IO.File.ReadAllBytes(MazeWasmInterop.GetWasmPath());
            _interop = MazeWasmInterop.GetInstance(ConnectionType.Wasmer, true, wasmBytes);
        }
        /// <summary>
        /// Returns the <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance to be used for the tests
        /// </summary>
        /// <returns> <see cref="Maze.Wasm.Interop.MazeWasmInterop"/> instance</returns>
        protected override MazeWasmInterop GetInterop()
        {
            return _interop;
        }
    }
#endif
}
