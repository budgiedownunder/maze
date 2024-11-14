using Maze.Wasm.Interop;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using static Maze.Wasm.Interop.MazeWasmInterop;

namespace Maze.Api
{
    /// <summary>
    /// The `Solution` class represents a maze solution
    /// </summary>
    public class Solution : IDisposable
    {
        static MazeWasmInterop interop = MazeWasmInterop.GetInstance();
        private bool _disposed = false;
        private UInt32 _solutionWasmPtr = default;
        /// <summary>
        /// Creates a new solution that wraps a [Maze.Wasm.Interop](xref:Maze.Wasm.Interop) solution pointer, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionWasmPtr">[Maze.Wasm.Interop](xref:Maze.Wasm.Interop) solution pointer</param>
        /// <returns>New solution instance</returns>
        public Solution(UInt32 solutionWasmPtr)
        {
            _solutionWasmPtr = solutionWasmPtr;
        }
        /// <summary>
        /// Handles object disposal, releasing managed and unmanaged [Maze.Wasm.Interop](xref:Maze.Wasm.Interop) resources and marking
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
                if (_solutionWasmPtr != 0)
                {
                    interop.FreeMazeWasmSolution(_solutionWasmPtr);
                    _solutionWasmPtr = 0;
                }

                _disposed = true;
            }
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~Solution()
        {
            Dispose(false);
        }
        /// <summary>
        /// Returns the list of points associated with the solution's path, or will throw an exception if the operation fails
        /// </summary>
        /// <returns>List of points</returns>
        public List<Maze.Point> GetPathPoints()
        {
            return Maze.ToMazePoints(interop.MazeWasmSolutionGetPathPoints(_solutionWasmPtr));
        }
    }
}
