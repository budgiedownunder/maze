using Maze.Wasm.Interop;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using static Maze.Wasm.Interop.MazeWasmInterop;

namespace Maze.Api
{
    /// <summary>
    /// The `Maze` class represents a maze
    /// </summary>
    public class Maze : IDisposable
    {
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
            public UInt32 Row;
            /// <summary>
            /// Column associated with the point (zero-based)
            /// </summary>
            public UInt32 Column;
        }
        /// <summary>
        /// Converts a MazeWasmPoint to a Maze.Point
        /// </summary>
        /// <returns>Maze.Point</returns>
        static public Maze.Point ToMazePoint(MazeWasmPoint wasmPoint)
        {
            return new Maze.Point
            {
                Row = wasmPoint.row,
                Column = wasmPoint.col
            };
        }
        /// <summary>
        /// Converts a list of MazeWasmPoint to a list of Maze.Point
        /// </summary>
        /// <returns>List of Maze.Point</returns>
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
        static MazeWasmInterop interop = MazeWasmInterop.GetInstance();
        private bool _disposed = false;
        private UInt32 _mazeWasmPtr = default;
        /// <summary>
        /// Creates a new maze, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="rowCount">Number of rows</param>
        /// <param name="colCount">Number of columns</param>
        /// <returns>New maze instance</returns>
        public Maze(UInt32 rowCount, UInt32 colCount)
        {
            _mazeWasmPtr = interop.NewMazeWasm();
            if (_mazeWasmPtr == 0)
            {
                throw new Exception("interop.NewMazeWasm() failed to create maze (zero returned)");
            }
            interop.MazeWasmResize(_mazeWasmPtr, rowCount, colCount);
        }
        /// <summary>
        /// Handles object disposal, releasing managed and unmanaged `MazeWasm.Interop` resources  and marking
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
                if (_mazeWasmPtr != 0)
                {
                    interop.FreeMazeWasm(_mazeWasmPtr);
                    _mazeWasmPtr = 0;
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
        /// <param name="mazeWasmPtr">Pointer to maze</param>
        /// <returns>Boolean</returns>
        public bool IsEmpty
        {
            get
            {
                return interop.MazeWasmIsEmpty(_mazeWasmPtr);
            }
        }
        /// <summary>
        /// The number of rows currently in the maze
        /// </summary>
        public UInt32 RowCount
        {
            get
            {
                return interop.MazeWasmGetRowCount(_mazeWasmPtr);
            }
        }
        /// <summary>
        /// The number of columns currently in the maze
        /// </summary>
        public UInt32 ColCount
        {
            get
            {
                return interop.MazeWasmGetColCount(_mazeWasmPtr);
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
            interop.MazeWasmResize(_mazeWasmPtr, newRowCount, newColCount);
        }
        /// <summary>
        /// Resets the maze to empty
        /// </summary>
        /// <returns>Nothing</returns>
        public void Reset()
        {
            interop.MazeWasmReset(_mazeWasmPtr);
        }
        /// <summary>
        /// Gets the cell type associated with a cell within the maze, or will throw an exception
        /// if the cell type cannot be determined
        /// </summary>
        /// <param name="row">Target row</param>
        /// <param name="col">Target column</param>
        /// <returns>Cell type</returns>
        public CellType GetCellType(UInt32 row, UInt32 col)
        {
            return (CellType)(int)interop.MazeWasmGetCellType(_mazeWasmPtr, row, col);
        }
        /// <summary>
        /// Sets the start cell associated with the maze`, or will throw an exception
        /// if the start cell cannot be set
        /// </summary>
        /// <param name="startRow">New start cell row</param>
        /// <param name="startCol">New start cell column</param>
        /// <returns>Nothing</returns>
        public void SetStartCell(UInt32 startRow, UInt32 startCol)
        {
            interop.MazeWasmSetStartCell(_mazeWasmPtr, startRow, startCol);
        }
        /// <summary>
        /// Gets the start cell associated with the maze, or will throw an exception
        /// if the start cell cannot be retrieved
        /// </summary>
        /// <returns>Start cell point</returns>
        public Maze.Point GetStartCell()
        {
            return ToMazePoint(interop.MazeWasmGetStartCell(_mazeWasmPtr));
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
            interop.MazeWasmSetFinishCell(_mazeWasmPtr, finishRow, finishCol);
        }
        /// <summary>
        /// Gets the finish cell associated with the maze, or will throw an exception
        /// if the finish cell cannot be retrieved
        /// </summary>
        /// <returns>FInish cell point</returns>
        public Maze.Point GetFinishCell()
        {
            return ToMazePoint(interop.MazeWasmGetFinishCell(_mazeWasmPtr));
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
            interop.MazeWasmSetWallCells(_mazeWasmPtr, startRow, startCol, endRow, endCol);
        }
        /// <summary>
        /// Inserts rows into the maze, or will throw an exception if the rows cannot be inserted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to insert</param>
        /// <returns>Nothing</returns>
        public void InsertRows(UInt32 startRow, UInt32 count)
        {
            interop.MazeWasmInsertRows(_mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Deletes rows from the maze, or will throw an exception if the rows cannot be deleted
        /// </summary>
        /// <param name="startRow">Target start row</param>
        /// <param name="count">Number rows to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteRows(UInt32 startRow, UInt32 count)
        {
            interop.MazeWasmDeleteRows(_mazeWasmPtr, startRow, count);
        }
        /// <summary>
        /// Inserts columns into the maze, or will throw an exception if the columns cannot be inserted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to insert</param>
        /// <returns>Nothing</returns>
        public void InsertCols(UInt32 startCol, UInt32 count)
        {
            interop.MazeWasmInsertCols(_mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Deletes columns from the maze, or will throw an exception if the columns cannot be deleted
        /// </summary>
        /// <param name="startCol">Target start column</param>
        /// <param name="count">Number columns to delete</param>
        /// <returns>Nothing</returns>
        public void DeleteCols(UInt32 startCol, UInt32 count)
        {
            interop.MazeWasmDeleteCols(_mazeWasmPtr, startCol, count);
        }
        /// <summary>
        /// Reinitialises a maze from a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="json">JSON strimg</param>
        /// <returns>Nothing</returns>
        public void FromJson(string json)
        {
            interop.MazeWasmFromJson(_mazeWasmPtr, json);
        }
        /// <summary>
        /// Converts a maze to a JSON string, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>JSON string</returns>
        public string ToJson()
        {
            return interop.MazeWasmToJson(_mazeWasmPtr);
        }
        /// <summary>
        /// Solves a maze, else will throw an exception if the operation fails. 
        /// </summary>
        /// <returns>Maze solution</returns>
        public global::Maze.Api.Solution Solve()
        {
            UInt32 solutionPtr = interop.MazeWasmSolve(_mazeWasmPtr);
            return new global::Maze.Api.Solution(solutionPtr);
        }
    }
}
