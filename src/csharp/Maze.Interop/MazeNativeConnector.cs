#if IOS
using System.Runtime.InteropServices;
using static Maze.Interop.MazeInterop;

namespace Maze.Interop
{
    /// <summary>
    /// iOS-only connector that P/Invokes into the statically-linked <c>maze_c</c>
    /// native library via <c>DllImport("__Internal")</c>.
    ///
    /// Implements <see cref="IMazeConnector"/> directly (no
    /// <see cref="MazeWebAssemblyConnectorBase"/> involvement) — all maze logic is
    /// executed natively without a WebAssembly runtime.
    /// </summary>
    internal sealed class MazeNativeConnector : IMazeConnector
    {
        // ── P/Invoke declarations ─────────────────────────────────────────────

        [DllImport("__Internal")] private static extern IntPtr maze_c_new_maze();
        [DllImport("__Internal")] private static extern void maze_c_free_maze(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte maze_c_maze_is_empty(IntPtr ptr);
        [DllImport("__Internal")] private static extern void maze_c_maze_resize(IntPtr ptr, UInt32 newRowCount, UInt32 newColCount);
        [DllImport("__Internal")] private static extern void maze_c_maze_reset(IntPtr ptr);
        [DllImport("__Internal")] private static extern UInt32 maze_c_maze_get_row_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern UInt32 maze_c_maze_get_col_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte maze_c_maze_get_cell_type(IntPtr ptr, UInt32 row, UInt32 col, out UInt32 outCellType);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_start_cell(IntPtr ptr, UInt32 row, UInt32 col);
        [DllImport("__Internal")] private static extern byte maze_c_maze_get_start_cell(IntPtr ptr, out UInt32 outRow, out UInt32 outCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_finish_cell(IntPtr ptr, UInt32 row, UInt32 col);
        [DllImport("__Internal")] private static extern byte maze_c_maze_get_finish_cell(IntPtr ptr, out UInt32 outRow, out UInt32 outCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_wall_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_clear_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_insert_rows(IntPtr ptr, UInt32 startRow, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_delete_rows(IntPtr ptr, UInt32 startRow, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_insert_cols(IntPtr ptr, UInt32 startCol, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_delete_cols(IntPtr ptr, UInt32 startCol, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_from_json(IntPtr ptr, [MarshalAs(UnmanagedType.LPStr)] string json);
        [DllImport("__Internal")] private static extern IntPtr maze_c_maze_to_json(IntPtr ptr);
        [DllImport("__Internal")] private static extern IntPtr maze_c_maze_solve(IntPtr ptr);
        [DllImport("__Internal")] private static extern void maze_c_free_maze_solution(IntPtr ptr);
        [DllImport("__Internal")] private static extern IntPtr maze_c_maze_solution_get_path_points(IntPtr solutionPtr, out UInt32 outCount);
        [DllImport("__Internal")] private static extern void maze_c_free_path_points(IntPtr ptr, UInt32 count);
        [DllImport("__Internal")] private static extern IntPtr maze_c_get_last_error();
        [DllImport("__Internal")] private static extern void maze_c_free_string(IntPtr ptr);
        [DllImport("__Internal")] private static extern Int64 maze_c_get_sized_memory_used();
        [DllImport("__Internal")] private static extern Int64 maze_c_get_num_objects_allocated();
        [DllImport("__Internal")] private static extern IntPtr maze_c_new_generator_options(UInt32 rowCount, UInt32 colCount, UInt32 algorithm, UInt64 seed);
        [DllImport("__Internal")] private static extern void maze_c_free_generator_options(IntPtr ptr);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_start(IntPtr ptr, UInt32 row, UInt32 col);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_finish(IntPtr ptr, UInt32 row, UInt32 col);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_min_spine_length(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_max_retries(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_branch_from_finish(IntPtr ptr, byte value);
        [DllImport("__Internal")] private static extern byte maze_c_maze_generate(IntPtr mazePtr, IntPtr optsPtr);

        // ── helpers ───────────────────────────────────────────────────────────

        private static string GetLastErrorMessage()
        {
            IntPtr errPtr = maze_c_get_last_error();
            return errPtr != IntPtr.Zero
                ? Marshal.PtrToStringAnsi(errPtr) ?? "unknown error"
                : "unknown error";
        }

        private static void ThrowIfError(byte result)
        {
            if (result == 0)
                throw new Exception(GetLastErrorMessage());
        }

        // ── IMazeConnector ────────────────────────────────────────────────

        public void Dispose() { /* no native resources held directly */ }

        public UIntPtr NewMaze()
        {
            IntPtr ptr = maze_c_new_maze();
            if (ptr == IntPtr.Zero)
                throw new Exception("maze_c_new_maze() returned null, possibly due to low memory");
            return (UIntPtr)(ulong)ptr;
        }

        public void FreeMaze(UIntPtr mazePtr)
        {
            maze_c_free_maze((IntPtr)(ulong)mazePtr);
        }

        public bool MazeIsEmpty(UIntPtr mazePtr)
        {
            return maze_c_maze_is_empty((IntPtr)(ulong)mazePtr) != 0;
        }

        public void MazeResize(UIntPtr mazePtr, UInt32 newRowCount, UInt32 newColCount)
        {
            maze_c_maze_resize((IntPtr)(ulong)mazePtr, newRowCount, newColCount);
        }

        public void MazeReset(UIntPtr mazePtr)
        {
            maze_c_maze_reset((IntPtr)(ulong)mazePtr);
        }

        public UInt32 MazeGetRowCount(UIntPtr mazePtr)
        {
            return maze_c_maze_get_row_count((IntPtr)(ulong)mazePtr);
        }

        public UInt32 MazeGetColCount(UIntPtr mazePtr)
        {
            return maze_c_maze_get_col_count((IntPtr)(ulong)mazePtr);
        }

        public MazeCellType MazeGetCellType(UIntPtr mazePtr, UInt32 row, UInt32 col)
        {
            byte ok = maze_c_maze_get_cell_type((IntPtr)(ulong)mazePtr, row, col, out UInt32 cellType);
            ThrowIfError(ok);
            return (MazeCellType)cellType;
        }

        public void MazeSetStartCell(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol)
        {
            ThrowIfError(maze_c_maze_set_start_cell((IntPtr)(ulong)mazePtr, startRow, startCol));
        }

        public MazePoint MazeGetStartCell(UIntPtr mazePtr)
        {
            byte ok = maze_c_maze_get_start_cell((IntPtr)(ulong)mazePtr, out UInt32 row, out UInt32 col);
            ThrowIfError(ok);
            return new MazePoint { row = row, col = col };
        }

        public void MazeSetFinishCell(UIntPtr mazePtr, UInt32 finishRow, UInt32 finishCol)
        {
            ThrowIfError(maze_c_maze_set_finish_cell((IntPtr)(ulong)mazePtr, finishRow, finishCol));
        }

        public MazePoint MazeGetFinishCell(UIntPtr mazePtr)
        {
            byte ok = maze_c_maze_get_finish_cell((IntPtr)(ulong)mazePtr, out UInt32 row, out UInt32 col);
            ThrowIfError(ok);
            return new MazePoint { row = row, col = col };
        }

        public void MazeSetWallCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_wall_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeClearCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_clear_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeInsertRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            ThrowIfError(maze_c_maze_insert_rows((IntPtr)(ulong)mazePtr, startRow, count));
        }

        public void MazeDeleteRows(UIntPtr mazePtr, UInt32 startRow, UInt32 count)
        {
            ThrowIfError(maze_c_maze_delete_rows((IntPtr)(ulong)mazePtr, startRow, count));
        }

        public void MazeInsertCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            ThrowIfError(maze_c_maze_insert_cols((IntPtr)(ulong)mazePtr, startCol, count));
        }

        public void MazeDeleteCols(UIntPtr mazePtr, UInt32 startCol, UInt32 count)
        {
            ThrowIfError(maze_c_maze_delete_cols((IntPtr)(ulong)mazePtr, startCol, count));
        }

        public void MazeFromJson(UIntPtr mazePtr, string json)
        {
            ThrowIfError(maze_c_maze_from_json((IntPtr)(ulong)mazePtr, json));
        }

        public string MazeToJson(UIntPtr mazePtr)
        {
            IntPtr jsonPtr = maze_c_maze_to_json((IntPtr)(ulong)mazePtr);
            if (jsonPtr == IntPtr.Zero)
                throw new Exception(GetLastErrorMessage());
            string json = Marshal.PtrToStringAnsi(jsonPtr) ?? string.Empty;
            maze_c_free_string(jsonPtr);
            return json;
        }

        public UIntPtr MazeSolve(UIntPtr mazePtr)
        {
            IntPtr solutionPtr = maze_c_maze_solve((IntPtr)(ulong)mazePtr);
            if (solutionPtr == IntPtr.Zero)
                throw new Exception(GetLastErrorMessage());
            return (UIntPtr)(ulong)solutionPtr;
        }

        public List<MazePoint> MazeSolutionGetPathPoints(UIntPtr solutionPtr)
        {
            if (solutionPtr == UIntPtr.Zero) throw new Exception("solutionPtr is zero");
            IntPtr rawPtr = maze_c_maze_solution_get_path_points((IntPtr)(ulong)solutionPtr, out UInt32 count);
            var points = new List<MazePoint>((int)count);
            if (rawPtr != IntPtr.Zero && count > 0)
            {
                int[] data = new int[2 * count];
                Marshal.Copy(rawPtr, data, 0, 2 * (int)count);
                for (int i = 0; i < (int)count; i++)
                    points.Add(new MazePoint { row = (UInt32)data[2 * i], col = (UInt32)data[2 * i + 1] });
                maze_c_free_path_points(rawPtr, count);
            }
            return points;
        }

        public void FreeMazeSolution(UIntPtr solutionPtr)
        {
            maze_c_free_maze_solution((IntPtr)(ulong)solutionPtr);
        }

        public UInt32 AllocateSizedMemory(UInt32 size)
        {
            throw new NotSupportedException("AllocateSizedMemory is not supported in Native mode");
        }

        public void FreeSizedMemory(UInt32 ptr)
        {
            throw new NotSupportedException("FreeSizedMemory is not supported in Native mode");
        }

        public Int64 GetSizedMemoryUsed()
        {
            return maze_c_get_sized_memory_used();
        }

        public Int64 GetNumObjectsAllocated()
        {
            return maze_c_get_num_objects_allocated();
        }

        public UIntPtr NewGeneratorOptions(UInt32 rowCount, UInt32 colCount, MazeGenerationAlgorithm algorithm, UInt64 seed)
        {
            IntPtr ptr = maze_c_new_generator_options(rowCount, colCount, (UInt32)algorithm, seed);
            if (ptr == IntPtr.Zero)
                throw new Exception("maze_c_new_generator_options() returned null, possibly due to low memory");
            return (UIntPtr)(ulong)ptr;
        }

        public void GeneratorOptionsSetStart(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            maze_c_generator_options_set_start((IntPtr)(ulong)optionsPtr, row, col);
        }

        public void GeneratorOptionsSetFinish(UIntPtr optionsPtr, UInt32 row, UInt32 col)
        {
            maze_c_generator_options_set_finish((IntPtr)(ulong)optionsPtr, row, col);
        }

        public void GeneratorOptionsSetMinSpineLength(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_min_spine_length((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetMaxRetries(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_max_retries((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetBranchFromFinish(UIntPtr optionsPtr, byte value)
        {
            maze_c_generator_options_set_branch_from_finish((IntPtr)(ulong)optionsPtr, value);
        }

        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            ThrowIfError(maze_c_maze_generate((IntPtr)(ulong)mazePtr, (IntPtr)(ulong)optionsPtr));
        }

        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            maze_c_free_generator_options((IntPtr)(ulong)optionsPtr);
        }
    }
}
#endif
