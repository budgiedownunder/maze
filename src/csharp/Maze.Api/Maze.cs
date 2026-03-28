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
        private UIntPtr _mazeWasmPtr = default;
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
        /// Converts a [MazeWasmPoint](xref:Maze.Interop.MazeInterop.MazeWasmPoint) to a [Maze.Point](xref:Maze.Api.Maze.Point)
        /// </summary>
        /// <param name="wasmPoint">Point to be converted</param>
        /// <returns>The resultant [Maze.Point](xref:Maze.Api.Maze.Point)</returns>
        static public Maze.Point ToMazePoint(MazeWasmPoint wasmPoint)
        {
            return new Maze.Point
            {
                Row = wasmPoint.row,
                Column = wasmPoint.col
            };
        }
        /// <summary>
        /// Converts a list of [MazeWasmPoint](xref:Maze.Interop.MazeInterop.MazeWasmPoint) points to a list of [Maze.Point](xref:Maze.Api.Maze.Point) points
        /// </summary>
        /// <returns>List of [Maze.Point](xref:Maze.Api.Maze.Point) points</returns>
        /// <param name="wasmPoints">List of [MazeWasmPoint](xref:Maze.Interop.MazeInterop.MazeWasmPoint) points to be converted</param>
        static public List<Maze.Point> ToMazePoints(List<MazeWasmPoint> wasmPoints)
        {
            int numPoints = wasmPoints.Count;
            List<Maze.Point> points = new List<Maze.Point>();
            for (int i = 0; i < numPoints; i++)
            {
                MazeWasmPoint wasmPoint = wasmPoints[i];
                points.Add(new Maze.Point
                {
                    Row = wasmPoint.row,
                    Column = wasmPoint.col,
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
            _mazeWasmPtr = Interop.NewMazeWasm();
            if (_mazeWasmPtr == UIntPtr.Zero)
            {
                throw new Exception("interop.NewMazeWasm() failed to create maze (zero returned)");
            }
            Interop.MazeWasmResize(_mazeWasmPtr, rowCount, colCount);
        }
        /// <summary>
        /// Generates a new maze from the given options, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="options">Generation options</param>
        /// <returns>New maze instance containing the generated maze</returns>
        public static Maze Generate(GenerationOptions options)
        {
            Maze maze = new Maze(0, 0);
            MazeWasmGenerationAlgorithm wasmAlgorithm = (MazeWasmGenerationAlgorithm)(byte)options.Algorithm;
            UIntPtr optionsPtr = maze.Interop.NewGeneratorOptionsWasm(
                options.RowCount,
                options.ColCount,
                wasmAlgorithm,
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
                maze.Interop.MazeWasmGenerate(maze._mazeWasmPtr, optionsPtr);
            }
            finally
            {
                maze.Interop.FreeGeneratorOptionsWasm(optionsPtr);
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
                if (_mazeWasmPtr != UIntPtr.Zero)
                {
                    Interop.FreeMazeWasm(_mazeWasmPtr);
                    _mazeWasmPtr = UIntPtr.Zero;
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
                return Interop.MazeWasmIsEmpty(_mazeWasmPtr);
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
                return Interop.MazeWasmGetRowCount(_mazeWasmPtr);
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
                return Interop.MazeWasmGetColCount(_mazeWasmPtr);
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
            Interop.MazeWasmResize(_mazeWasmPtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets the maze to empty
        /// </summary>
        /// <returns>Nothing</returns>
        public void Reset()
        {
            Interop.MazeWasmReset(_mazeWasmPtr);
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
            return (CellType)(int)Interop.MazeWasmGetCellType(_mazeWasmPtr, row, column);
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
            Interop.MazeWasmSetStartCell(_mazeWasmPtr, startRow, startCol);
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
            return ToMazePoint(Interop.MazeWasmGetStartCell(_mazeWasmPtr));
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
            Interop.MazeWasmSetFinishCell(_mazeWasmPtr, finishRow, finishCol);
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
            return ToMazePoint(Interop.MazeWasmGetFinishCell(_mazeWasmPtr));
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
            Interop.MazeWasmSetWallCells(_mazeWasmPtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Inserts rows into the maze, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void InsertRows(UInt32 startRow, UInt32 count)
        {
            Interop.MazeWasmInsertRows(_mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Deletes rows from the maze, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteRows(UInt32 startRow, UInt32 count)
        {
            Interop.MazeWasmDeleteRows(_mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Inserts columns into the maze, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void InsertCols(UInt32 startCol, UInt32 count)
        {
            Interop.MazeWasmInsertCols(_mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Deletes columns from the maze, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteCols(UInt32 startCol, UInt32 count)
        {
            Interop.MazeWasmDeleteCols(_mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="json">JSON string</param>
        /// <returns>Nothing</returns>
        public void FromJson(string json)
        {
            Interop.MazeWasmFromJson(_mazeWasmPtr, json);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>JSON string</returns>
        public string ToJson()
        {
            return Interop.MazeWasmToJson(_mazeWasmPtr);
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails. 
        /// </summary>
        /// <returns>Maze solution</returns>
        public global::Maze.Api.Solution Solve()
        {
            UIntPtr solutionPtr = Interop.MazeWasmSolve(_mazeWasmPtr);
            return new global::Maze.Api.Solution(solutionPtr);
        }
    }
}
