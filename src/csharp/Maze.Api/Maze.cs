using Maze.Interop;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using static Maze.Interop.MazeInterop;

namespace Maze.Api
{
    /// <summary>
    /// The `Maze` class represents a maze
    /// </summary>
    public class Maze : IDisposable
    {
        // Private data
        static MazeInterop _interop = MazeInterop.GetInstance(); // Used when UseStaticInterop = true
        private bool _disposed = false;
        private UIntPtr _mazePtr = default;
        /// <summary>
        /// Controls whether the object uses a statically defined [Maze.Interop](xref:Maze.Interop) instance (default = `true`). If
        /// `false`, then the maze determines the current instance on a per-API call basis.
        /// </summary>
        /// <returns>Boolean</returns>
        public static bool UseStaticInterop { get; set; } = true;
        /// <summary>
        /// The current [Maze.Interop](xref:Maze.Interop) associated with the object
        /// </summary>
        /// <returns>[Maze.Interop](xref:Maze.Interop) instance</returns>
        public MazeInterop Interop
        {
            get
            {
                return UseStaticInterop ? _interop : MazeInterop.GetInstance();
            }
        }
        /// <summary>
        /// Defines the type of a maze cell
        /// </summary>
        public enum CellType
        {
            /// <summary>
            /// An empty cell
            /// </summary>
            Empty = 0,
            /// <summary>
            /// A starting cell within the maze
            /// </summary>
            Start = 1,
            /// <summary>
            ///  A finishing cell within the maze
            /// </summary>
            Finish = 2,
            /// <summary>
            /// A cell containing a wall, meaning it can't be passed through
            /// </summary>
            Wall = 3,
        }

        /// <summary>
        /// Represents a point within a maze
        /// </summary>
       public struct Point
        {
            /// <summary>
            /// Row associated with the point (zero-based)
            /// </summary>
            /// <returns>Row index (zero-based)</returns>
            public UInt32 Row;
            /// <summary>
            /// Column associated with the point (zero-based)
            /// </summary>
            /// <returns>Column index (zero-based)</returns>
            public UInt32 Column;
        }
        /// <summary>
        /// Identifies the maze generation algorithm to use
        /// </summary>
        public enum GenerationAlgorithm
        {
            /// <summary>
            /// Two-phase recursive backtracking
            /// </summary>
            RecursiveBacktracking = 0,
        }
        /// <summary>
        /// Options that control maze generation
        /// </summary>
        public class GenerationOptions
        {
            /// <summary>Number of rows in the generated maze. Must be >= 3.</summary>
            public UInt32 RowCount { get; set; }
            /// <summary>Number of columns in the generated maze. Must be >= 3.</summary>
            public UInt32 ColCount { get; set; }
            /// <summary>Generation algorithm to use</summary>
            public GenerationAlgorithm Algorithm { get; set; } = GenerationAlgorithm.RecursiveBacktracking;
            /// <summary>Random number generator seed for deterministic generation</summary>
            public UInt64 Seed { get; set; }
            /// <summary>Start cell row. When null, defaults to row 0.</summary>
            public UInt32? StartRow { get; set; }
            /// <summary>Start cell column. When null, defaults to column 0.</summary>
            public UInt32? StartCol { get; set; }
            /// <summary>Finish cell row. When null, defaults to the last row.</summary>
            public UInt32? FinishRow { get; set; }
            /// <summary>Finish cell column. When null, defaults to the last column.</summary>
            public UInt32? FinishCol { get; set; }
            /// <summary>Minimum spine path length. When null, defaults to (RowCount + ColCount) / 2.</summary>
            public UInt32? MinSpineLength { get; set; }
            /// <summary>Maximum generation attempts before throwing. When null, defaults to 100.</summary>
            public UInt32? MaxRetries { get; set; }
            /// <summary>Whether branches may grow out of the finish cell. Defaults to false.</summary>
            public bool? BranchFromFinish { get; set; }
        }
        /// <summary>
        /// Converts a [MazePoint](xref:Maze.Interop.MazeInterop.MazePoint) to a [Maze.Point](xref:Maze.Api.Maze.Point)
        /// </summary>
        /// <param name="pt">Point to be converted</param>
        /// <returns>The resultant [Maze.Point](xref:Maze.Api.Maze.Point)</returns>
        static public Maze.Point ToMazePoint(MazePoint pt)
        {
            return new Maze.Point
            {
                Row = pt.row,
                Column = pt.col
            };
        }
        /// <summary>
        /// Converts a list of [MazePoint](xref:Maze.Interop.MazeInterop.MazePoint) points to a list of [Maze.Point](xref:Maze.Api.Maze.Point) points
        /// </summary>
        /// <returns>List of [Maze.Point](xref:Maze.Api.Maze.Point) points</returns>
        /// <param name="pts">List of [MazePoint](xref:Maze.Interop.MazeInterop.MazePoint) points to be converted</param>
        static public List<Maze.Point> ToMazePoints(List<MazePoint> pts)
        {
            int numPoints = pts.Count;
            List<Maze.Point> points = new List<Maze.Point>();
            for (int i = 0; i < numPoints; i++)
            {
                MazePoint pt = pts[i];
                points.Add(new Maze.Point
                {
                    Row = pt.row,
                    Column = pt.col,
                });
            }
            return points;
        }
        /// <summary>
        /// Creates a new maze, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="rowCount">Number of rows</param>
        /// <param name="colCount">Number of columns</param>
        /// <returns>New maze instance</returns>
        public Maze(UInt32 rowCount, UInt32 colCount)
        {
            _mazePtr = Interop.NewMaze();
            if (_mazePtr == UIntPtr.Zero)
            {
                throw new Exception("interop.NewMaze() failed to create maze (zero returned)");
            }
            Interop.MazeResize(_mazePtr, rowCount, colCount);
        }
        /// <summary>
        /// Generates a new maze from the given options, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="options">Generation options</param>
        /// <returns>New maze instance containing the generated maze</returns>
        public static Maze Generate(GenerationOptions options)
        {
            Maze maze = new Maze(0, 0);
            MazeGenerationAlgorithm mazeAlgorithm = (MazeGenerationAlgorithm)(byte)options.Algorithm;
            UIntPtr optionsPtr = maze.Interop.NewGeneratorOptions(
                options.RowCount,
                options.ColCount,
                mazeAlgorithm,
                options.Seed);
            try
            {
                if (options.StartRow.HasValue && options.StartCol.HasValue)
                    maze.Interop.GeneratorOptionsSetStart(optionsPtr, options.StartRow.Value, options.StartCol.Value);
                if (options.FinishRow.HasValue && options.FinishCol.HasValue)
                    maze.Interop.GeneratorOptionsSetFinish(optionsPtr, options.FinishRow.Value, options.FinishCol.Value);
                if (options.MinSpineLength.HasValue)
                    maze.Interop.GeneratorOptionsSetMinSpineLength(optionsPtr, options.MinSpineLength.Value);
                if (options.MaxRetries.HasValue)
                    maze.Interop.GeneratorOptionsSetMaxRetries(optionsPtr, options.MaxRetries.Value);
                if (options.BranchFromFinish.HasValue)
                    maze.Interop.GeneratorOptionsSetBranchFromFinish(optionsPtr, options.BranchFromFinish.Value ? (byte)1 : (byte)0);
                maze.Interop.MazeGenerate(maze._mazePtr, optionsPtr);
            }
            finally
            {
                maze.Interop.FreeGeneratorOptions(optionsPtr);
            }
            return maze;
        }
        /// <summary>
        /// Handles object disposal, releasing managed and unmanaged [Maze.Interop](xref:Maze.Interop) resources and marking
        /// the object as having been finalized
        /// </summary>
        /// <returns>Nothing</returns>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }
        /// <summary>
        /// Handles object disposal
        /// </summary>
        /// <param name="disposing">Flag indicating whether the object should be fully disposed (ie. including managed
        /// as well as unmanaged  resources)</param>
        /// <returns>Nothing</returns>
        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                // Dispose unmanaged resources
                if (_mazePtr != UIntPtr.Zero)
                {
                    Interop.FreeMaze(_mazePtr);
                    _mazePtr = UIntPtr.Zero;
                }

                _disposed = true;
            }
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~Maze()
        {
            Dispose(false);
        }
        /// <summary>
        /// Whether the maze is empty
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsEmpty
        {
            get
            {
                return Interop.MazeIsEmpty(_mazePtr);
            }
        }
        /// <summary>
        /// The number of rows currently in the maze
        /// </summary>
        /// <returns>Number of rows</returns>
        public UInt32 RowCount
        {
            get
            {
                return Interop.MazeGetRowCount(_mazePtr);
            }
        }
        /// <summary>
        /// The number of columns currently in the maze
        /// </summary>
        /// <returns>Number of columns</returns>
        public UInt32 ColCount
        {
            get
            {
                return Interop.MazeGetColCount(_mazePtr);
            }
        }
        /// <summary>
        /// Resizes the maze
        /// </summary>
        /// <param name="newRowCount">New number of rows</param>
        /// <param name="newColCount">New number of columns</param>
        /// <returns>Nothing</returns>
        public void Resize(UInt32 newRowCount, UInt32 newColCount)
        {
            Interop.MazeResize(_mazePtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets the maze to empty
        /// </summary>
        /// <returns>Nothing</returns>
        public void Reset()
        {
            Interop.MazeReset(_mazePtr);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within the maze, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="row">Target row</param>
        /// <param name="column">Target column</param>
        /// <returns>Cell type</returns>
        public CellType GetCellType(UInt32 row, UInt32 column)
        {
            return (CellType)(int)Interop.MazeGetCellType(_mazePtr, row, column);
        }
        /// <summary>
        /// Sets the start cell associated with the maze, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void SetStartCell(UInt32 startRow, UInt32 startCol)
        {
            Interop.MazeSetStartCell(_mazePtr, startRow, startCol);
        }
        /// <summary>
        /// Indicates whether the maze has a start cell defined
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool HasStartCell
        {
            get { try { GetStartCell(); return true; } catch { return false; } }
        }
        /// <summary>
        /// Gets the start cell associated with the maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <returns>Start cell point</returns>
        public Maze.Point GetStartCell()
        {
            return ToMazePoint(Interop.MazeGetStartCell(_mazePtr));
        }
        /// <summary>
        /// Sets the finish cell associated with the maze, or will throw an exception
        /// if the finish cell cannot be set
        /// </summary>
        /// <param name="finishRow">New finish cell row</param>
        /// <param name="finishCol">New finsh cell column</param>
        /// <returns>Nothing</returns>
        public void SetFinishCell(UInt32 finishRow, UInt32 finishCol)
        {
            Interop.MazeSetFinishCell(_mazePtr, finishRow, finishCol);
        }
        /// <summary>
        /// Indicates whether the maze has a finish cell defined
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool HasFinishCell
        {
            get { try { GetFinishCell(); return true; } catch { return false; } }
        }
        /// <summary>
        /// Gets the finish cell associated with the maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <returns>Finish cell point</returns>
        public Maze.Point GetFinishCell()
        {
            return ToMazePoint(Interop.MazeGetFinishCell(_mazePtr));
        }
        /// <summary>
        /// Sets a range of cells to walls within a maze, or will throw an exception
        /// if the walls cannot be set
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="startCol">Target start column</param>
        /// <param name="endRow">Target end row</param>
        /// <param name="endCol">Target end column</param>
        /// <returns>Nothing</returns>
        public void SetWallCells(UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            Interop.MazeSetWallCells(_mazePtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Inserts rows into the maze, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void InsertRows(UInt32 startRow, UInt32 count)
        {
            Interop.MazeInsertRows(_mazePtr, startRow, count);
        }
        /// <summary>
        /// Deletes rows from the maze, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteRows(UInt32 startRow, UInt32 count)
        {
            Interop.MazeDeleteRows(_mazePtr, startRow, count);
        }
        /// <summary>
        /// Inserts columns into the maze, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void InsertCols(UInt32 startCol, UInt32 count)
        {
            Interop.MazeInsertCols(_mazePtr, startCol, count);
        }
        /// <summary>
        /// Deletes columns from the maze, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteCols(UInt32 startCol, UInt32 count)
        {
            Interop.MazeDeleteCols(_mazePtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="json">JSON string</param>
        /// <returns>Nothing</returns>
        public void FromJson(string json)
        {
            Interop.MazeFromJson(_mazePtr, json);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>JSON string</returns>
        public string ToJson()
        {
            return Interop.MazeToJson(_mazePtr);
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails. 
        /// </summary>
        /// <returns>Maze solution</returns>
        public global::Maze.Api.Solution Solve()
        {
            UIntPtr solutionPtr = Interop.MazeSolve(_mazePtr);
            return new global::Maze.Api.Solution(solutionPtr);
        }
    }
}
