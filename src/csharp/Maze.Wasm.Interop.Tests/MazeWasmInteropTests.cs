namespace Maze.Wasm.Interop.Tests
{
    using Xunit;
    using Maze.Wasm.Interop;
    using System;
    using static Maze.Wasm.Interop.MazeWasmInterop;
    using System.Xml.Linq;
    using Microsoft.VisualBasic;

    public class MazeWasmInteropTests
    {
        MazeWasmInterop interop = MazeWasmInterop.GetInstance("..\\..\\..\\..\\..\\rust\\target\\wasm32-unknown-unknown\\release\\maze_wasm.wasm");

        private UInt32 CreateNewMazeWasm(UInt32 numRows, UInt32 numCols)
        {
            UInt32 mazeWasmPtr = interop.NewMazeWasm();
            if (numRows > 0 || numCols > 0)
            {
                interop.MazeWasmResize(mazeWasmPtr, numRows, numCols);
            }
            return mazeWasmPtr;
        }
        private void FreeMazeWasm(UInt32 mazeWasmPtr)
        {
            interop.FreeMazeWasm(mazeWasmPtr);
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
        public void MazeWasmGetRowCount_ShouldReturnCorrectNumberRows()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UInt32 mazeWasmPtr = CreateNewMazeWasm(targetRowCount, targetColCount);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, targetRowCount);
        }
        [Fact]
        public void MazeWasmGetColCount_ShouldReturnCorrectNumberCols()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            UInt32 mazeWasmPtr = CreateNewMazeWasm(targetRowCount, targetColCount);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, targetColCount);
        }
        [Fact]
        public void MazeWasmReset_ShouldSucceed()
        {
            UInt32 mazeWasmPtr = CreateNewMazeWasm(10, 5);
            interop.MazeWasmReset(mazeWasmPtr);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 0);
            AssertColCount(colCount, 0);
        }
        [Fact]
        public void MazeWasmResize_ChangesRowAndColumnCounts()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            interop.MazeWasmInsertRows(mazeWasmPtr, 0, 2);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 2);
        }
        [Fact]
        public void MazeWasmInsertRows_FailsForInvalidStartRow()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(3, 1);
            interop.MazeWasmDeleteRows(mazeWasmPtr, 0, 1);
            var rowCount = interop.MazeWasmGetRowCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertRowCount(rowCount, 2);
        }
        [Fact]
        public void MazeWasmInsertCols_FailsForEmptyMaze()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 0);
            interop.MazeWasmInsertCols(mazeWasmPtr, 0, 3);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, 3);
        }
        [Fact]
        public void MazeWasmDeleteCols_FailsForEmptyMaze()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 3);
            interop.MazeWasmDeleteCols(mazeWasmPtr, 0, 1);
            var colCount = interop.MazeWasmGetColCount(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertColCount(colCount, 2);
        }
        [Fact]
        public void MazeWasmGetCellType_FailsForEmptyMaze()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(1, 1);
            MazeWasmCellType cellType = interop.MazeWasmGetCellType(mazeWasmPtr, 0, 0);
            FreeMazeWasm(mazeWasmPtr);
            AssertCellType(cellType, MazeWasmCellType.Empty);
        }
        [Fact]
        public void MazeWasmGetStartCell_FailsForEmptyMaze()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 5);
            MazeWasmPoint start = new MazeWasmPoint();
            interop.MazeWasmSetStartCell(mazeWasmPtr, 1, 2);
            start = interop.MazeWasmGetStartCell(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            AssertStartCell(start, new MazeWasmPoint() { row = 1, col = 2 });
        }
        [Fact]
        public void MazeWasmGetFinishCell_FailsForEmptyMaze()
        {
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(5, 10);
            interop.MazeWasmSetWallCells(mazeWasmPtr, 0, 0, 3, 6);
            AssertRangeCellType(mazeWasmPtr, 0, 0, 3, 6, MazeWasmCellType.Wall, true);
            FreeMazeWasm(mazeWasmPtr);
        }
        [Fact]
        public void MazeWasmToJson_ShouldSucceed()
        {
            var expected = @"{""name"":"""",""definition"":{""grid"":[]}}";
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var json = interop.MazeWasmToJson(mazeWasmPtr);
            FreeMazeWasm(mazeWasmPtr);
            Assert.Equal(json, expected);
        }
        [Fact]
        public void MazeWasmFromJson_ShouldFail()
        {
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
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        ["" "", ""W""],
                        ["" "", ""F""]
                    ]
                }
            }
            ";
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
                ""name"":""test"",
                ""definition"": {
                    ""grid"":[
                        [""S"", ""W""],
                        ["" "", "" ""]
                    ]
                }
            }
            ";
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
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var bytesUsed = interop.GetSizedMemoryUsed();
            FreeMazeWasm(mazeWasmPtr);
            var bytesUsedExpected = 0;
            Assert.True(bytesUsed == bytesUsedExpected, $"Expected {bytesUsedExpected} bytes used but got '{bytesUsed}'");
        }
        [Fact]
        public void MazeWasmGetNumObjectsAllocated_ShouldSucceedAndBeNonZero()
        {
            UInt32 mazeWasmPtr = CreateNewMazeWasm(0, 0);
            var numObjects = interop.GetNumObjectsAllocated();
            FreeMazeWasm(mazeWasmPtr);
            var numObjectsExpected = 1;
            Assert.True(numObjects == numObjectsExpected, $"Expected {numObjectsExpected} but got '{numObjects}'");
        }
    }
}
