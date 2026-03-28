using Maze.Interop;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using static Maze.Interop.MazeInterop;

namespace Maze.Api
{
    /// <summary>
    /// The `Solution` class represents a maze solution
    /// </summary>
    public class Solution : IDisposable
    {
        // Private data
        static MazeInterop _interop = MazeInterop.GetInstance(); // Used when UseStaticInterop = true
        private bool _disposed = false;
        private UIntPtr _solutionWasmPtr = default;
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
        /// Creates a new solution that wraps a [Maze.Interop](xref:Maze.Interop) solution pointer, or will throw an exception if the operation fails
        /// </summary>
        /// <param name="solutionWasmPtr">[Maze.Interop](xref:Maze.Interop) solution pointer</param>
        /// <returns>New solution instance</returns>
        public Solution(UIntPtr solutionWasmPtr)
        {
            _solutionWasmPtr = solutionWasmPtr;
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
                if (_solutionWasmPtr != UIntPtr.Zero)
                {
                    Interop.FreeMazeWasmSolution(_solutionWasmPtr);
                    _solutionWasmPtr = UIntPtr.Zero;
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
            return Maze.ToMazePoints(Interop.MazeWasmSolutionGetPathPoints(_solutionWasmPtr));
        }
    }
}
