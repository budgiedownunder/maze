using Xunit;

[assembly: CollectionBehavior(DisableTestParallelization = true)]
namespace Maze.Api.Tests
{
    using global::Maze.Api;
    using global::Maze.Interop;
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
        /// <summary>
        /// Confirms that <see cref="Maze.RowCount"/> returns the expected number of rows
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.ColCount"/> returns the expected number of columns
        /// </summary>
        [Fact]
        public void MazeColCount_ShouldReturnCorrectNumberCols()
        {
            UInt32 targetRowCount = 10;
            UInt32 targetColCount = 5;
            using (Maze maze = new Maze(targetRowCount, targetColCount))
            {
                AssertColCount(maze.ColCount, targetColCount);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Reset"/> removes all rows and columns
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.Resize"/> correctly adjusts the number of rows and columns
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.InsertRows"/> succeeds and results in the expected number of rows
        /// </summary>
        [Fact]
        public void MazeInsertRows_SucceedsForValidStartRow()
        {
            using (Maze maze = new Maze(0, 0))
            {
                maze.InsertRows(0, 2);
                AssertRowCount(maze.RowCount, 2);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.InsertRows"/> fails for an invalid start row
        /// </summary>
        [Fact]
        public void MazeInsertRows_FailsForInvalidStartRow()
        {
            using (Maze maze = new Maze(0, 0))
            {
                var exception = Assert.ThrowsAny<Exception>(() =>
                {
                    maze.InsertRows(1, 2);
                });
                Assert.Equal("invalid 'start_row' index (1)", exception.Message);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteRows"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteRows"/> fails for an invalid start row
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteRows"/> fails if the number of rows requested is too large
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteRows"/> succeeds for a valid start row and row count
        /// </summary>
        [Fact]
        public void MazeDeleteRows_SucceedsForValidStartRow()
        {
            using (Maze maze = new Maze(3, 1))
            {
                maze.DeleteRows(0, 1);
                AssertRowCount(maze.RowCount, 2);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.InsertCols"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.InsertCols"/> fails for an invalid start column
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.InsertCols"/> succeeds for a valid start column
        /// </summary>
        [Fact]
        public void MazeInsertCols_SucceedsForValidStartCol()
        {
            using (Maze maze = new Maze(1, 0))
            {
                maze.InsertCols(0, 3);
                AssertColCount(maze.ColCount, 3);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteCols"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteCols"/> fails for an invalid start column
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteCols"/> fails if the number of columns requested is too large
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.DeleteCols"/> succeeds for a valid start column and column count
        /// </summary>
        [Fact]
        public void MazeDeleteCols_SucceedsForValidStartCol()
        {
            using (Maze maze = new Maze(1, 3))
            {
                maze.DeleteCols(0, 1);
                AssertColCount(maze.ColCount, 2);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.GetCellType"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetCellType"/> fails for an invalid target row
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetCellType"/> fails for an invalid target column
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetCellType"/> succeeds for valid cell location
        /// </summary>
        [Fact]
        public void MazeGetCellType_SucceedsForValidCellLocation()
        {
            using (Maze maze = new Maze(1, 1))
            {
                Maze.CellType cellType = maze.GetCellType(0, 0);
                AssertCellType(cellType, Maze.CellType.Empty);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.GetStartCell"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetStartCell"/> fails if a start cell is not defined
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetStartCell"/> succeeds if a start cell is defined
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.HasStartCell"/> returns false for an empty maze
        /// </summary>
        [Fact]
        public void MazeHasStartCell_ReturnsFalseForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                Assert.False(maze.HasStartCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.HasStartCell"/> returns false if a start cell is not defined
        /// </summary>
        [Fact]
        public void MazeHasStartCell_ReturnsFalseIfNotDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Assert.False(maze.HasStartCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.HasStartCell"/> returns true if a start cell is defined
        /// </summary>
        [Fact]
        public void MazeHasStartCell_ReturnsTrueIfDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                maze.SetStartCell(1, 2);
                Assert.True(maze.HasStartCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.GetFinishCell"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetFinishCell"/> fails if a finish cell is not defined
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.GetFinishCell"/> succeeds if a finish cell is defined
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.HasFinishCell"/> returns false for an empty maze
        /// </summary>
        [Fact]
        public void MazeHasFinishCell_ReturnsFalseForEmptyMaze()
        {
            using (Maze maze = new Maze(0, 0))
            {
                Assert.False(maze.HasFinishCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.HasFinishCell"/> returns false if a finish cell is not defined
        /// </summary>
        [Fact]
        public void MazeHasFinishCell_ReturnsFalseIfNotDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                Assert.False(maze.HasFinishCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.HasFinishCell"/> returns true if a finish cell is defined
        /// </summary>
        [Fact]
        public void MazeHasFinishCell_ReturnsTrueIfDefined()
        {
            using (Maze maze = new Maze(5, 5))
            {
                maze.SetFinishCell(3, 4);
                Assert.True(maze.HasFinishCell);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.SetWallCells"/> fails for an empty maze
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.SetWallCells"/> fails for an invalid start location
        /// </summary>
        [Fact]
        public void MazeSetWallCells_FailsForInvalidStartLocation()
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
        /// <summary>
        /// Confirms that <see cref="Maze.SetWallCells"/> fails for an invalid end location
        /// </summary>
        [Fact]
        public void MazeSetWallCells_FailsForInvalidEndLocation()
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
        /// <summary>
        /// Confirms that <see cref="Maze.SetWallCells"/> succeeds for a valid cell range
        /// </summary>
        [Fact]
        public void MazeSetWallCells_SucceedsForValidCellRange()
        {
            using (Maze maze = new Maze(5, 10))
            {
                maze.SetWallCells(0, 0, 3, 6);
                AssertRangeCellType(maze, 0, 0, 3, 6, Maze.CellType.Wall);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.ToJson"/> succeeds
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.FromJson"/> fails for invalid JSON
        /// </summary>
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
        /// <summary>
        /// Confirms that <see cref="Maze.FromJson"/> succeeds for valid JSON
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                AssertRowCount(maze.RowCount, 7);
                AssertColCount(maze.ColCount, 5);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Solve"/> fails for a maze that has no start cell defined and with the expected error
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
        /// <summary>
        /// Confirms that <see cref="Maze.Solve"/> fails for a maze that has no finish cell defined and with the expected error
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
        /// <summary>
        /// Confirms that <see cref="Maze.Solve"/> succeeds for a valid maze that has a solution
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
            using (Maze maze = new Maze(0, 0))
            {
                maze.FromJson(jsonStr);
                using (global::Maze.Api.Solution solution = maze.Solve())
                {

                }
            }
        }
        /// <summary>
        /// Confirms that <see cref="Solution.GetPathPoints"/> succeeds for a solution and returns the expected number of points
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
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> succeeds with only the required parameters
        /// </summary>
        [Fact]
        public void MazeGenerate_WithMinimalOptions_ShouldSucceed()
        {
            using (Maze maze = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 7,
                ColCount = 5,
                Seed = 42,
            }))
            {
                AssertRowCount(maze.RowCount, 7);
                AssertColCount(maze.ColCount, 5);
                Assert.False(maze.IsEmpty);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> produces the same maze for the same seed
        /// </summary>
        [Fact]
        public void MazeGenerate_SameSeedProducesDeterministicOutput()
        {
            var options = new Maze.GenerationOptions
            {
                RowCount = 11,
                ColCount = 11,
                Seed = 999,
            };
            using (Maze maze1 = Maze.Generate(options))
            using (Maze maze2 = Maze.Generate(options))
            {
                Assert.Equal(maze1.ToJson(), maze2.ToJson());
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> places start and finish cells at the specified positions
        /// </summary>
        [Fact]
        public void MazeGenerate_WithExplicitStartAndFinish_ShouldPlaceCellsCorrectly()
        {
            using (Maze maze = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 9,
                ColCount = 9,
                Seed = 1,
                StartRow = 0,
                StartCol = 0,
                FinishRow = 8,
                FinishCol = 8,
            }))
            {
                AssertRowCount(maze.RowCount, 9);
                AssertColCount(maze.ColCount, 9);
                AssertStartCell(maze.GetStartCell(), new Maze.Point { Row = 0, Column = 0 });
                AssertFinishCell(maze.GetFinishCell(), new Maze.Point { Row = 8, Column = 8 });
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> succeeds when all options are provided
        /// </summary>
        [Fact]
        public void MazeGenerate_WithAllOptions_ShouldSucceed()
        {
            using (Maze maze = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 11,
                ColCount = 11,
                Seed = 7,
                StartRow = 0,
                StartCol = 0,
                FinishRow = 10,
                FinishCol = 10,
                MinSpineLength = 5,
                MaxRetries = 50,
                BranchFromFinish = false,
            }))
            {
                AssertRowCount(maze.RowCount, 11);
                AssertColCount(maze.ColCount, 11);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails with the expected message when row count is less than 3
        /// </summary>
        [Fact]
        public void MazeGenerate_WithRowCountLessThan3_ShouldFail()
        {
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 2,
                    ColCount = 5,
                    Seed = 1,
                });
            });
            Assert.Equal("row_count must be at least 3", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails with the expected message when column count is less than 3
        /// </summary>
        [Fact]
        public void MazeGenerate_WithColCountLessThan3_ShouldFail()
        {
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 2,
                    Seed = 1,
                });
            });
            Assert.Equal("col_count must be at least 3", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails with the expected message when the start cell is out of bounds
        /// </summary>
        [Fact]
        public void MazeGenerate_WithStartOutOfBounds_ShouldFail()
        {
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 5,
                    Seed = 1,
                    StartRow = 10,
                    StartCol = 0,
                });
            });
            Assert.Equal("start is out of bounds", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails with the expected message when the finish cell is out of bounds
        /// </summary>
        [Fact]
        public void MazeGenerate_WithFinishOutOfBounds_ShouldFail()
        {
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 5,
                    Seed = 1,
                    FinishRow = 0,
                    FinishCol = 10,
                });
            });
            Assert.Equal("finish is out of bounds", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails with the expected message when start and finish are the same cell
        /// </summary>
        [Fact]
        public void MazeGenerate_WithStartEqualToFinish_ShouldFail()
        {
            var exception = Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 5,
                    Seed = 1,
                    StartRow = 2,
                    StartCol = 2,
                    FinishRow = 2,
                    FinishCol = 2,
                });
            });
            Assert.Equal("start and finish must be different cells", exception.Message);
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> succeeds when a valid MinSpineLength is provided
        /// </summary>
        [Fact]
        public void MazeGenerate_WithValidMinSpineLength_ShouldSucceed()
        {
            using (Maze maze = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 9,
                ColCount = 9,
                Seed = 42,
                MinSpineLength = 3,
            }))
            {
                AssertRowCount(maze.RowCount, 9);
                AssertColCount(maze.ColCount, 9);
                Assert.False(maze.IsEmpty);
            }
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> fails when MinSpineLength is impossible to satisfy within MaxRetries attempts
        /// </summary>
        [Fact]
        public void MazeGenerate_WithImpossibleMinSpineLength_ShouldFail()
        {
            Assert.ThrowsAny<Exception>(() =>
            {
                Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 5,
                    Seed = 1,
                    MinSpineLength = 1000,
                    MaxRetries = 2,
                });
            });
        }
        /// <summary>
        /// Confirms that <see cref="Maze.Generate"/> produces different mazes for different seeds on a
        /// 5x5 grid with corner-to-corner start/finish (the smallest realistic user configuration).
        /// </summary>
        [Fact]
        public void MazeGenerate_DifferentSeedsProduce_DifferentMazes_5x5_CornerToCorner()
        {
            int differentCount = 0;
            string? firstJson = null;
            for (ulong seed = 1; seed <= 20; seed++)
            {
                using Maze maze = Maze.Generate(new Maze.GenerationOptions
                {
                    RowCount = 5,
                    ColCount = 5,
                    StartRow = 0,
                    StartCol = 0,
                    FinishRow = 4,
                    FinishCol = 4,
                    MinSpineLength = 5,
                    Seed = seed,
                });
                string json = maze.ToJson();
                if (firstJson is null)
                    firstJson = json;
                else if (json != firstJson)
                    differentCount++;
            }
            Assert.True(differentCount > 0, "All 20 seeds produced identical 5x5 mazes — seed is not affecting generation");
        }
        /// <summary>
        /// Regression test: consecutive large seed values (~1.77×10^12) must produce different mazes.
        /// These values are identical when truncated to float32 (precision ~131,072 at this scale),
        /// so this would have caught the ulong→float precision loss bug in MazeWasmtimeConnector.ToValueBox.
        /// </summary>
        [Fact]
        public void MazeGenerate_DifferentSeedsProduce_DifferentMazes_LargeSeeds()
        {
            // Two consecutive seeds at ~1.77×10^12 scale — identical as float32, distinct as int64
            const ulong seedA = 1_768_464_000_000UL;
            const ulong seedB = 1_768_464_000_001UL;

            using Maze mazeA = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 5,
                ColCount = 5,
                StartRow = 0,
                StartCol = 0,
                FinishRow = 4,
                FinishCol = 4,
                MinSpineLength = 5,
                Seed = seedA,
            });
            using Maze mazeB = Maze.Generate(new Maze.GenerationOptions
            {
                RowCount = 5,
                ColCount = 5,
                StartRow = 0,
                StartCol = 0,
                FinishRow = 4,
                FinishCol = 4,
                MinSpineLength = 5,
                Seed = seedB,
            });

            Assert.True(mazeA.ToJson() != mazeB.ToJson(),
                $"Seeds {seedA} and {seedB} produced identical mazes — seed precision may be lost (ulong→float truncation)");
        }

        // --- MazeGame tests ---

        // 1 row, 3 cols: S[0,0]  [0,1]  F[0,2]
        private const string SimpleGameJson = """{"grid":[["S"," ","F"]]}""";
        // Wall blocks move right: S[0,0]  W[0,1]  F[0,2]
        private const string WalledGameJson = """{"grid":[["S","W","F"]]}""";

        /// <summary>
        /// Confirms that <see cref="MazeGame.Create"/> throws for invalid JSON
        /// </summary>
        [Fact]
        public void MazeGameCreate_ShouldThrow_ForInvalidJson()
        {
            var exception = Assert.ThrowsAny<Exception>(() => MazeGame.Create("{"));
            Assert.Equal("Failed to create maze game session — invalid JSON or maze has no start cell", exception.Message);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.Create"/> throws when the maze has no start cell
        /// </summary>
        [Fact]
        public void MazeGameCreate_ShouldThrow_WhenNoStartCell()
        {
            var exception = Assert.ThrowsAny<Exception>(() => MazeGame.Create("""{"grid":[[" "," ","F"]]}"""));
            Assert.Equal("Failed to create maze game session — invalid JSON or maze has no start cell", exception.Message);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.Create"/> succeeds for valid maze JSON
        /// </summary>
        [Fact]
        public void MazeGameCreate_ShouldSucceed()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.NotNull(game);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.MovePlayer"/> returns <see cref="MazeGameMoveResult.None"/> when direction is <see cref="MazeGameDirection.None"/>
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturnNone_WhenDirectionIsNone()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.Equal(MazeGameMoveResult.None, game.MovePlayer(MazeGameDirection.None));
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.MovePlayer"/> returns <see cref="MazeGameMoveResult.Moved"/> when the path is clear
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturnMoved_WhenPathIsClear()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.Equal(MazeGameMoveResult.Moved, game.MovePlayer(MazeGameDirection.Right));
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.MovePlayer"/> returns <see cref="MazeGameMoveResult.Blocked"/> when a wall is ahead
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturnBlocked_WhenWallAhead()
        {
            using MazeGame game = MazeGame.Create(WalledGameJson);
            Assert.Equal(MazeGameMoveResult.Blocked, game.MovePlayer(MazeGameDirection.Right));
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.MovePlayer"/> returns <see cref="MazeGameMoveResult.Complete"/> when the finish cell is reached
        /// </summary>
        [Fact]
        public void MazeGameMovePlayer_ShouldReturnComplete_WhenFinishReached()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            game.MovePlayer(MazeGameDirection.Right); // → [0,1]
            Assert.Equal(MazeGameMoveResult.Complete, game.MovePlayer(MazeGameDirection.Right)); // → [0,2] = F
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.PlayerRow"/> and <see cref="MazeGame.PlayerCol"/> are at the start cell initially
        /// </summary>
        [Fact]
        public void MazeGamePlayerPosition_ShouldBeAtStartCell_Initially()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.Equal(0, game.PlayerRow);
            Assert.Equal(0, game.PlayerCol);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.PlayerDirection"/> is <see cref="MazeGameDirection.None"/> before the first move
        /// </summary>
        [Fact]
        public void MazeGamePlayerDirection_ShouldBeNone_Initially()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.Equal(MazeGameDirection.None, game.PlayerDirection);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.IsComplete"/> is false before the finish cell is reached
        /// </summary>
        [Fact]
        public void MazeGameIsComplete_ShouldBeFalse_Initially()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            Assert.False(game.IsComplete);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.IsComplete"/> is true after the finish cell is reached
        /// </summary>
        [Fact]
        public void MazeGameIsComplete_ShouldBeTrue_AfterReachingFinish()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            game.MovePlayer(MazeGameDirection.Right);
            game.MovePlayer(MazeGameDirection.Right);
            Assert.True(game.IsComplete);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.VisitedCells"/> contains only the start cell before any moves
        /// </summary>
        [Fact]
        public void MazeGameVisitedCells_ShouldContainStartCell_Initially()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            var visited = game.VisitedCells;
            Assert.Equal(1, visited.Count);
            Assert.Equal(0, visited[0].Row);
            Assert.Equal(0, visited[0].Col);
        }

        /// <summary>
        /// Confirms that <see cref="MazeGame.VisitedCells"/> grows as the player moves through the maze
        /// </summary>
        [Fact]
        public void MazeGameVisitedCells_ShouldGrowAsPlayerMoves()
        {
            using MazeGame game = MazeGame.Create(SimpleGameJson);
            game.MovePlayer(MazeGameDirection.Right); // [0,1]
            Assert.Equal(2, game.VisitedCells.Count);
            game.MovePlayer(MazeGameDirection.Right); // [0,2] = F
            Assert.Equal(3, game.VisitedCells.Count);
        }
    }
    /// <summary>
    ///  This class defines the [Wasmtime](https://docs.wasmtime.dev/) text fixture used by the [Maze.Api.Tests.MazeApiWasmtimeTest_Static](xref:Maze.Api.Tests.MazeApiWasmtimeTest_Static) and
    ///  [Maze.Api.Tests.MazeApiWasmtimeTest_NonStatic](xref:Maze.Api.Tests.MazeApiWasmtimeTest_NonStatic) classes
    /// </summary>
    public class WasmtimeTestFixture : IDisposable
    {
        /// <summary>
        ///  Constructor for the [Wasmtime](https://docs.wasmtime.dev/) test fixture
        /// </summary>
        public WasmtimeTestFixture()
        {
            MazeInterop.Disconnect();
            MazeInterop.Initialize(MazeInterop.ConnectionType.Wasmtime, true);
        }
        /// <summary>Explicitly disconnects the interop instance after all tests in this collection complete.</summary>
        public void Dispose() { MazeInterop.Disconnect(); GC.SuppressFinalize(this); }
    }
    /// <summary>
    ///  This class is used to apply [Wasmtime](https://docs.wasmtime.dev/) `[CollectionDefinition]` and `ICollectionFixture` to the [Maze.Api.Tests.MazeApiWasmtimeTest_Static](xref:Maze.Api.Tests.MazeApiWasmtimeTest_Static) and
    ///  [Maze.Api.Tests.MazeApiWasmtimeTest_NonStatic](xref:Maze.Api.Tests.MazeApiWasmtimeTest_NonStatic) classes
    /// </summary>
    [CollectionDefinition("WasmtimeTestFixtureCollection")]
    public class WasmtimeTestFixtureCollection : ICollectionFixture<WasmtimeTestFixture>
    {
        // This class is intentionally left empty
        // It is used to apply [CollectionDefinition] and ICollectionFixture
    }
    /// <summary>
    ///  This class contains the static [Wasmtime](https://docs.wasmtime.dev/) [Maze.Interop.MazeInterop.ConnectionType](xref:Maze.Interop.MazeInterop.ConnectionType) [`xUnit`](https://xunit.net/)
    ///  unit tests for the [Maze.Api](xref:Maze.Api) class
    /// </summary>
    [Collection("WasmtimeTestFixtureCollection")]
    public class MazeApiWasmtimeTest_Static : MazeApiTestBase
    {
        private readonly WasmtimeTestFixture _fixture;
        /// <summary>
        ///  Constructor for the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Api"/> tests that use a statically allocated <see cref="MazeInterop"/>
        /// </summary>
        public MazeApiWasmtimeTest_Static(WasmtimeTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = true;
            Solution.UseStaticInterop = true;
            MazeGame.UseStaticInterop = true;
        }
    }
    /// <summary>
    ///  This class contains the non-static [Wasmtime](https://docs.wasmtime.dev/) [Maze.Interop.MazeInterop.ConnectionType](xref:Maze.Interop.MazeInterop.ConnectionType) [`xUnit`](https://xunit.net/) unit tests for the [Maze.Api](xref:Maze.Api) class
    /// </summary>
    [Collection("WasmtimeTestFixtureCollection")]
    public class MazeApiWasmtimeTest_NonStatic : MazeApiTestBase
    {
        private readonly WasmtimeTestFixture _fixture;
        /// <summary>
        ///  Constructor for the [Wasmtime](https://docs.wasmtime.dev/) <see cref="Api"/> tests that use a dynamically allocated <see cref="MazeInterop"/>
        /// </summary>
        public MazeApiWasmtimeTest_NonStatic(WasmtimeTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = false;
            Solution.UseStaticInterop = false;
            MazeGame.UseStaticInterop = false;
        }
    }
#if WINDOWS
    /// <summary>
    ///  This class defines the [Wasmer](https://wasmer.io/) text fixture used by the [Maze.Api.Tests.MazeApiWasmerTest_Static](xref:Maze.Api.Tests.MazeApiWasmerTest_Static) and
    ///  [Maze.Api.Tests.MazeApiWasmerTest_NonStatic](xref:Maze.Api.Tests.MazeApiWasmerTest_NonStatic) classes
    /// </summary>
    public class WasmerTestFixture : IDisposable
    {
        /// <summary>
        ///  Constructor for the [Wasmer](https://wasmer.io/) test fixture
        /// </summary>
        public WasmerTestFixture()
        {
            MazeInterop.Disconnect();
            MazeInterop.Initialize(MazeInterop.ConnectionType.Wasmer, true);
        }
        /// <summary>Explicitly disconnects the interop instance after all tests in this collection complete.</summary>
        public void Dispose() { MazeInterop.Disconnect(); GC.SuppressFinalize(this); }
    }
    /// <summary>
    ///  This class is used to apply [Wasmer](https://wasmer.io/) `[CollectionDefinition]` and `ICollectionFixture` to  the[Maze.Api.Tests.MazeApiWasmerTest_Static](xref:Maze.Api.Tests.MazeApiWasmerTest_Static) and
    ///  [Maze.Api.Tests.MazeApiWasmerTest_NonStatic](xref:Maze.Api.Tests.MazeApiWasmerTest_NonStatic) classes
    /// </summary>
    [CollectionDefinition("WasmerTestFixtureCollection")]
    public class WasmerTestFixtureCollection : ICollectionFixture<WasmerTestFixture>
    {
        // This class is intentionally left empty
        // It is used to apply [CollectionDefinition] and ICollectionFixture
    }
    /// <summary>
    ///  This class contains the static [Wasmer](https://wasmer.io/) [Maze.Interop.MazeInterop.ConnectionType](xref:Maze.Interop.MazeInterop.ConnectionType) [`xUnit`](https://xunit.net/) unit tests for the [Maze.Api](xref:Maze.Api) class
    /// </summary>
    [Collection("WasmerTestFixtureCollection")]
    public class MazeApiWasmerTest_Static : MazeApiTestBase
    {
        private readonly WasmerTestFixture _fixture;
        /// <summary>
        ///  Constructor for the [Wasmer](https://wasmer.io/) <see cref="Api"/> tests that use a statically allocated <see cref="MazeInterop"/>
        /// </summary>
        public MazeApiWasmerTest_Static(WasmerTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = true;
            Solution.UseStaticInterop = true;
            MazeGame.UseStaticInterop = true;
        }
    }
    /// <summary>
    ///  This class contains the non-static [Wasmer](https://wasmer.io/) [Maze.Interop.MazeInterop.ConnectionType](xref:Maze.Interop.MazeInterop.ConnectionType) [`xUnit`](https://xunit.net/) unit tests for the [Maze.Api](xref:Maze.Api) class
    /// </summary>
    [Collection("WasmerTestFixtureCollection")]
    public class MazeApiWasmerTest_NonStatic : MazeApiTestBase
    {
        private readonly WasmerTestFixture _fixture;
        /// <summary>
        ///  Constructor for the [Wasmer](https://wasmer.io/) <see cref="Api"/> tests that use a dynamically allocated <see cref="MazeInterop"/>
        /// </summary>
        public MazeApiWasmerTest_NonStatic(WasmerTestFixture fixture)
        {
            _fixture = fixture;
            Maze.UseStaticInterop = false;
            Solution.UseStaticInterop = false;
            MazeGame.UseStaticInterop = false;
        }
    }
#endif
}
