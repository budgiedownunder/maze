using Xunit;

[assembly: CollectionBehavior(DisableTestParallelization = true)]
namespace Maze.Api.Tests
{
    using global::Maze.Api;
    using global::Maze.Wasm.Interop;
    using Microsoft.VisualStudio.TestPlatform.Utilities;
    using System;
    using System.Diagnostics;
    using System.Diagnostics.Metrics;
    using Xunit.Abstractions;

    /// <summary>
    ///  This base class contains the [`xUnit`](https://xunit.net/) unit tests for the [Maze.Api](xref:Maze.Api) .NET class library
    /// </summary>
    public abstract class MazeApiTestBase()
    {
        private void AssertRowCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected rowCount to be {expected} but got {actual}");
        }
        private void AssertColCount(UInt32 actual, UInt32 expected)
        {
            Assert.True(actual == expected, $"Expected colCount to be {expected} but got {actual}");
        }
        private void AssertCellType(Maze.CellType actual, Maze.CellType expected)
        {
            Assert.True(actual == expected, $"Expected cell type to be '{expected}' but got '{actual}'");
        }
        private void AssertPoint(string context, Maze.Point actual, Maze.Point expected)
        {
            Assert.True(actual.Row == expected.Row, $"Expected {context} point row to be '{expected.Row}' but got '{actual.Row}'");
            Assert.True(actual.Column == expected.Column, $"Expected {context} point column to be '{expected.Column}' but got '{actual.Column}'");
        }
        private void AssertStartCell(Maze.Point actual, Maze.Point expected)
        {
            AssertPoint("start", actual, expected);
        }
        private void AssertFinishCell(Maze.Point actual, Maze.Point expected)
        {
            AssertPoint("finish", actual, expected);
        }
        private void AssertRangeCellType(Maze maze, UInt32 fromRow, UInt32 fromCol, UInt32 toRow, UInt32 toCol, Maze.CellType expected)
        {
            for (UInt32 row = fromRow; row <= toRow; row++)
            {
                for (UInt32 col = fromCol; col <= toCol; col++)
                {
                    Maze.CellType cellType = maze.GetCellType(fromRow, fromCol);
                    Assert.True(cellType == expected, $"Expected cell type at [{row}, {col}] to be '{expected}' but got '{cellType}'");
                }
            }
        }
        [Fact]
        public void MazeRowCount_ShouldReturnCorrectNumberRows()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            using (Maze maze = new Maze(targetRowCount, targetColCount))
            {
                AssertRowCount(maze.RowCount, targetRowCount);
            }
        }
        [Fact]
        public void MazeColCount_ShouldReturnCorrectNumberCols()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            Maze maze = new Maze(targetRowCount, targetColCount);
            AssertColCount(maze.ColCount, targetColCount);
        }
        [Fact]
        public void MazeReset_ShouldSucceed()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            using (Maze maze = new Maze(targetRowCount, targetColCount))
            {
                AssertRowCount(maze.RowCount, targetRowCount);
                AssertColCount(maze.ColCount, targetColCount);
                maze.Reset();
                AssertRowCount(maze.RowCount, 0);
                AssertColCount(maze.RowCount, 0);
            }
        }
        [Fact]
        public void MazeResize_ChangesRowAndColumnCounts()
        {
            using (Maze maze = new Maze(0, 0))
            {
                UInt32 targetRowCount = 10;
                UInt32 targetColCount = 5;

                maze.Resize(targetRowCount, targetColCount);
                AssertRowCount(maze.RowCount, targetRowCount);
                AssertColCount(maze.ColCount, targetColCount);
            }
        }
        [Fact]
        public void MazeInsertRows_SucceedsForValidStartRow()
        {
            using (Maze maze = new Maze(0, 0))
            {
                maze.InsertRows(0, 2);
                AssertRowCount(maze.RowCount, 2);
            }
        }
        [Fact]
        public void MazeInsertRows_FailsForInvalidStartRow()
        {
            Maze maze = new Maze(0, 0);
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                maze.InsertRows(1, 2);
            });
            Assert.Equal("invalid 'start_row' index (1)", exception.Message);
        }
        [Fact]
        public void MazeDeleteRows_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteRows(1, 2);
                });
                Assert.Equal("definition is empty", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteRows_FailsForInvalidStartRow()
        {
            using (Maze maze = new Maze(1, 1))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteRows(1, 2);
                });
                Assert.Equal("invalid 'start_row' index (1)", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteRows_FailsIfCountTooLarge()
        {
            using (Maze maze = new Maze(2, 1))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteRows(1, 3);
                });
                Assert.Equal("invalid 'count' (3) - too large", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteRows_SucceedsForValidStartRow()
        {
            using (Maze maze = new Maze(3, 1))
            {
                maze.DeleteRows(0, 1);
                AssertRowCount(maze.RowCount, 2);
            }
        }
        [Fact]
        public void MazeInsertCols_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.InsertCols(0, 0);
                });
                Assert.Equal("definition is empty", exception.Message);
            }
        }
        [Fact]
        public void MazeInsertCols_FailsForInvalidStartCol()
        {
            using (Maze maze = new Maze(1, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.InsertCols(2, 1);
                });
                Assert.Equal("invalid 'start_col' index (2)", exception.Message);
            }
        }
        [Fact]
        public void MazeInsertCols_SucceedsForValidStartCol()
        {
            using (Maze maze = new Maze(1, 0))
            {
                maze.InsertCols(0, 3);
                AssertColCount(maze.ColCount, 3);
            }
        }
        [Fact]
        public void MazeDeleteCols_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteCols(1, 2);
                });
                Assert.Equal("definition is empty", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteCols_FailsForInvalidStartCol()
        {
            using (Maze maze = new Maze(1, 1))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteCols(1, 2);
                });
                Assert.Equal("invalid 'start_col' index (1)", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteCols_FailsIfCountTooLarge()
        {
            using (Maze maze = new Maze(2, 2))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.DeleteCols(1, 3);
                });
                Assert.Equal("invalid 'count' (3) - too large", exception.Message);
            }
        }
        [Fact]
        public void MazeDeleteCols_SucceedsForValidStartCol()
        {
            using (Maze maze = new Maze(1, 3))
            {
                maze.DeleteCols(0, 1);
                AssertColCount(maze.ColCount, 2);
            }
        }
        [Fact]
        public void MazeGetCellType_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                Maze.CellType cellType = Maze.CellType.Empty;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    cellType = maze.GetCellType(0, 0);
                });
                Assert.Equal("row index (0) out of bounds", exception.Message);
            }
        }
        [Fact]
        public void MazeGetCellType_FailsForInvalidRow()
        {
            using (Maze maze = new Maze(10, 5))
            {
                Maze.CellType cellType = Maze.CellType.Empty;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    cellType = maze.GetCellType(10, 4);
                });
                Assert.Equal("row index (10) out of bounds", exception.Message);
            }
        }
        [Fact]
        public void MazeGetCellType_FailsForInvalidCol()
        {
            using (Maze maze = new Maze(10, 5))
            {
                Maze.CellType cellType = Maze.CellType.Empty;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    cellType = maze.GetCellType(9, 5);
                });
                Assert.Equal("column index (5) out of bounds", exception.Message);
            }
        }
        [Fact]
        public void MazeGetCellType_SucceedsForValidCellLocation()
        {
            using (Maze maze = new Maze(1, 1))
            {
                Maze.CellType cellType = maze.GetCellType(0, 0);
                AssertCellType(cellType, Maze.CellType.Empty);
            }
        }
        [Fact]
        public void MazeGetStartCell_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                Maze.Point? start;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    start = maze.GetStartCell();
                });
                Assert.Equal("no start cell defined", exception.Message);
            }
        }
        [Fact]
        public void MazeGetStartCell_FailsIfNotDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Maze.Point? start;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    start = maze.GetStartCell();
                });
                Assert.Equal("no start cell defined", exception.Message);
            }
        }
        [Fact]
        public void MazeGetStartCell_SucceedsIfDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Maze.Point start = new Maze.Point();
                maze.SetStartCell(1, 2);
                start = maze.GetStartCell();
                AssertStartCell(start, new Maze.Point() { Row = 1, Column = 2 });
            }
        }
        [Fact]
        public void MazeGetFinishCell_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                Maze.Point? finish;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    finish = maze.GetFinishCell();
                });
                Assert.Equal("no finish cell defined", exception.Message);
            }
        }
        [Fact]
        public void MazeGetFinishCell_FailsIfNotDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Maze.Point finish;
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    finish = maze.GetFinishCell();
                });
                Assert.Equal("no finish cell defined", exception.Message);
            }
        }
        [Fact]
        public void MazeGetFinishCell_SucceedsIfDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Maze.Point finish = new Maze.Point();
                maze.SetFinishCell(3, 4);
                finish = maze.GetFinishCell();
                AssertFinishCell(finish, new Maze.Point() { Row = 3, Column = 4 });
            }
        }
        [Fact]
        public void MazeSetWallCells_FailsForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.SetWallCells(0, 0, 0, 0);
                });
                Assert.Equal("invalid 'from' point [0, 0]", exception.Message);
            }
        }
        [Fact]
        public void MazeSetWallCells_FailsForInvalidFromLocation()
        {
            using (Maze maze = new Maze(5, 10))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.SetWallCells(5, 1, 3, 6);
                });
                Assert.Equal("invalid 'from' point [5, 1]", exception.Message);
            }
        }
        [Fact]
        public void MazeSetWallCells_FailsForInvalidToLocation()
        {
            using (Maze maze = new Maze(5, 10))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.SetWallCells(0, 0, 5, 6);
                });
                Assert.Equal("invalid 'to' point [5, 6]", exception.Message);
            }
        }
        [Fact]
        public void MazeSetWallCells_SucceedsForValidCellRange()
        {
            using (Maze maze = new Maze(5, 10))
            {
                maze.SetWallCells(0, 0, 3, 6);
                AssertRangeCellType(maze, 0, 0, 3, 6, Maze.CellType.Wall);
            }
        }
        [Fact]
        public void MazeToJson_ShouldSucceed()
        {
            var expected = @"{""id"":"""",""name"":"""",""definition"":{""grid"":[]}}";
            using (Maze maze = new Maze(0, 0))
            {
                var json = maze.ToJson();
                Assert.Equal(json, expected);
            }
        }
        [Fact]
        public void MazeFromJson_ShouldFail()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.FromJson("{");
                });
                Assert.Equal("EOF while parsing an object at line 1 column 1", exception.Message);
            }
        }
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                AssertRowCount(maze.RowCount, 7);
                AssertColCount(maze.ColCount, 5);
            }
        }
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    global::Maze.Api.Solution solution = maze.Solve();
                });
                Assert.Equal("no start cell found within maze", exception.Message);
            }
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    global::Maze.Api.Solution solution = maze.Solve();
                });
                Assert.Equal("no finish cell found within maze", exception.Message);
            }
        }
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                using (global::Maze.Api.Solution solution = maze.Solve())
                {

                }
            }
        }
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                using (global::Maze.Api.Solution solution = maze.Solve())
                {
                    List<Maze.Point> solutionPath = solution.GetPathPoints();
                    Int32 numPointsExpected = 13;
                    Assert.True(solutionPath.Count == numPointsExpected, $"Expected {numPointsExpected} points in the solution but got '{solutionPath.Count}'");
                }

            }
        }
    }
    /// <summary>
    ///  This class defines the [Wasmtime](https://docs.wasmtime.dev/) text fixture used by the <see cref="Maze.Api.Tests.MazeApiWasmtimeTest"/> class
    /// </summary>
    public class WasmtimeTestFixture
    {
        public WasmtimeTestFixture()
        {
            MazeWasmInterop.Disconnect();
            MazeWasmInterop.Initialize(MazeWasmInterop.ConnectionType.Wasmtime, true);
        }
    }
    /// <summary>
    ///  This class is used to apply [Wasmtime](https://docs.wasmtime.dev/) `[CollectionDefinition]` and `ICollectionFixture` to the <see cref="Maze.Api.Tests.MazeApiWasmtimeTest"/> class
    /// </summary>
    [CollectionDefinition("WasmtimeTestFixtureCollection")]
    public class WasmtimeTestFixtureCollection : ICollectionFixture<WasmtimeTestFixture>
    {
        // This class is intentionally left empty
        // It is used to apply [CollectionDefinition] and ICollectionFixture
    }
    /// <summary>
    ///  This class contains the static [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Api"/> class
    /// </summary>
    [Collection("WasmtimeTestFixtureCollection")]
    public class MazeApiWasmtimeTest_Static: MazeApiTestBase
    {
        private readonly WasmtimeTestFixture _fixture;
        public MazeApiWasmtimeTest_Static(WasmtimeTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = true;
            Solution.UseStaticInterop = true;
        }
    }
    /// <summary>
    ///  This class contains the non-static [Wasmtime](https://docs.wasmtime.dev/) <see cref="Maze.Wasm.Interop.MazeWasmInterop.ConnectionType"/> [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Api"/> class
    /// </summary>
    [Collection("WasmtimeTestFixtureCollection")]
    public class MazeApiWasmtimeTest_NonStatic : MazeApiTestBase
    {
        private readonly WasmtimeTestFixture _fixture;
        public MazeApiWasmtimeTest_NonStatic(WasmtimeTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = false;
            Solution.UseStaticInterop = false;
        }
    }
#if WINDOWS
    /// <summary>
    ///  This class defines the [Wasmer](https://wasmer.io/) text fixture used by the <see cref="Maze.Api.Tests.MazeApiWasmerTest"/> class
    /// </summary>
    public class WasmerTestFixture
    {
        public WasmerTestFixture()
        {
            MazeWasmInterop.Disconnect();
            MazeWasmInterop.Initialize(MazeWasmInterop.ConnectionType.Wasmer, true);
        }
    }
    /// <summary>
    ///  This class is used to apply [Wasmer](https://wasmer.io/) `[CollectionDefinition]` and `ICollectionFixture` to the <see cref="Maze.Api.Tests.MazeApiWasmerTest"/> class
    /// </summary>
    [CollectionDefinition("WasmerTestFixtureCollection")]
    public class WasmerTestFixtureCollection : ICollectionFixture<WasmerTestFixture>
    {
        // This class is intentionally left empty
        // It is used to apply [CollectionDefinition] and ICollectionFixture
    }
    /// <summary>
    ///  This class contains the static [Wasmer](https://wasmer.io/) <see cref = "Maze.Wasm.Interop.MazeWasmInterop.ConnectionType" /> [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Api"/> class
    //// </ summary >
    [Collection("WasmerTestFixtureCollection")]
    public class MazeApiWasmerTest_Static : MazeApiTestBase
    {
        private readonly WasmerTestFixture _fixture;
        public MazeApiWasmerTest_Static(WasmerTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = true;
            Solution.UseStaticInterop = true;
        }
    }
    /// <summary>
    ///  This class contains the non-static [Wasmer](https://wasmer.io/) <see cref = "Maze.Wasm.Interop.MazeWasmInterop.ConnectionType" /> [`xUnit`](https://xunit.net/) unit tests for the <see cref="Maze.Api"/> class
    //// </ summary >
    [Collection("WasmerTestFixtureCollection")]
    public class MazeApiWasmerTest_NonStatic : MazeApiTestBase
    {
        private readonly WasmerTestFixture _fixture;
        public MazeApiWasmerTest_NonStatic(WasmerTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = false;
            Solution.UseStaticInterop = false;
        }
    }
#endif
}
