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
    /// <c>MazeWebAssemblyConnectorBase</c> involvement) — all maze logic is
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
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_key_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_door_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_enemy_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_health_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_set_treasure_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_clear_cells(IntPtr ptr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol);
        [DllImport("__Internal")] private static extern byte maze_c_maze_insert_rows(IntPtr ptr, UInt32 startRow, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_delete_rows(IntPtr ptr, UInt32 startRow, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_insert_cols(IntPtr ptr, UInt32 startCol, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_delete_cols(IntPtr ptr, UInt32 startCol, UInt32 count);
        [DllImport("__Internal")] private static extern byte maze_c_maze_from_json(IntPtr ptr, [MarshalAs(UnmanagedType.LPUTF8Str)] string json);
        [DllImport("__Internal")] private static extern IntPtr maze_c_maze_to_json(IntPtr ptr);
        [DllImport("__Internal")] private static extern IntPtr maze_c_maze_get_cell_entity(IntPtr ptr, uint row, uint col);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_set_cell_entity(IntPtr ptr, uint row, uint col, [MarshalAs(UnmanagedType.LPUTF8Str)] string json);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_clear_cell_entity(IntPtr ptr, uint row, uint col);
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
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_door_count(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_spare_doors(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_spare_keys(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_enemy_count(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_health_count(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern void maze_c_generator_options_set_treasure_count(IntPtr ptr, UInt32 value);
        [DllImport("__Internal")] private static extern byte maze_c_maze_generate(IntPtr mazePtr, IntPtr optsPtr);
        [DllImport("__Internal")] private static extern IntPtr maze_c_new_maze_game([MarshalAs(UnmanagedType.LPUTF8Str)] string json);
        [DllImport("__Internal")] private static extern void   maze_c_free_maze_game(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_move_player(IntPtr ptr, int dir);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_player_row(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_player_col(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_player_direction(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_is_complete(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_is_lost(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_lose_reason(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_pickup(IntPtr ptr, out uint kindOut, out uint idOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_bag_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_bag_item(IntPtr ptr, int index, out uint kindOut, out uint idOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_door_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_door(IntPtr ptr, int index, out uint rowOut, out uint colOut, out uint stateOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_tick(IntPtr ptr, float dtMs);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_tick_event_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_tick_event(IntPtr ptr, int index, out uint kindOut, out uint rowOut, out uint colOut);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_tick_event_payload(IntPtr ptr, int index, out uint payloadOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_key_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_key(IntPtr ptr, int index, out uint rowOut, out uint colOut, out uint idOut);
        [DllImport("__Internal")] private static extern uint   maze_c_maze_game_hp(IntPtr ptr);
        [DllImport("__Internal")] private static extern uint   maze_c_maze_game_max_hp(IntPtr ptr);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_enemy_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_enemy(IntPtr ptr, int index, out uint rowOut, out uint colOut, out uint idOut, out uint damageOut, out float movePeriodMsOut, out int enemyTypeOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_health_pickup_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_health_pickup(IntPtr ptr, int index, out uint rowOut, out uint colOut, out uint idOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_treasure_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_treasure(IntPtr ptr, int index, out uint rowOut, out uint colOut, out int styleOut, out uint valueOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_collected_treasure_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_collected_treasure(IntPtr ptr, int index, out int styleOut, out uint countOut);
        [DllImport("__Internal")] private static extern int    maze_c_maze_game_visited_cell_count(IntPtr ptr);
        [DllImport("__Internal")] private static extern byte   maze_c_maze_game_get_visited_cell(IntPtr ptr, int index, out int rowOut, out int colOut);

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

        public void MazeSetKeyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_key_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeSetDoorCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_door_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeSetEnemyCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_enemy_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeSetHealthCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_health_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
        }

        public void MazeSetTreasureCells(UIntPtr mazePtr, UInt32 startRow, UInt32 startCol, UInt32 endRow, UInt32 endCol)
        {
            ThrowIfError(maze_c_maze_set_treasure_cells((IntPtr)(ulong)mazePtr, startRow, startCol, endRow, endCol));
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

        public string? MazeGetCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            IntPtr jsonPtr = maze_c_maze_get_cell_entity((IntPtr)(ulong)mazePtr, row, col);
            if (jsonPtr == IntPtr.Zero)
                return null; // no override on this cell
            string json = Marshal.PtrToStringAnsi(jsonPtr) ?? string.Empty;
            maze_c_free_string(jsonPtr);
            return json;
        }

        public void MazeSetCellEntity(UIntPtr mazePtr, uint row, uint col, string json)
        {
            ThrowIfError(maze_c_maze_set_cell_entity((IntPtr)(ulong)mazePtr, row, col, json));
        }

        public void MazeClearCellEntity(UIntPtr mazePtr, uint row, uint col)
        {
            maze_c_maze_clear_cell_entity((IntPtr)(ulong)mazePtr, row, col);
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

        public void GeneratorOptionsSetDoorCount(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_door_count((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetSpareDoors(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_spare_doors((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetSpareKeys(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_spare_keys((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetEnemyCount(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_enemy_count((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetHealthCount(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_health_count((IntPtr)(ulong)optionsPtr, value);
        }

        public void GeneratorOptionsSetTreasureCount(UIntPtr optionsPtr, UInt32 value)
        {
            maze_c_generator_options_set_treasure_count((IntPtr)(ulong)optionsPtr, value);
        }

        public void MazeGenerate(UIntPtr mazePtr, UIntPtr optionsPtr)
        {
            ThrowIfError(maze_c_maze_generate((IntPtr)(ulong)mazePtr, (IntPtr)(ulong)optionsPtr));
        }

        public void FreeGeneratorOptions(UIntPtr optionsPtr)
        {
            maze_c_free_generator_options((IntPtr)(ulong)optionsPtr);
        }

        public UIntPtr NewMazeGame(string definitionJson)
        {
            IntPtr ptr = maze_c_new_maze_game(definitionJson);
            if (ptr == IntPtr.Zero)
                throw new Exception($"maze_c_new_maze_game() failed: {GetLastErrorMessage()}");
            return (UIntPtr)(ulong)ptr;
        }

        public void FreeMazeGame(UIntPtr gamePtr)
        {
            maze_c_free_maze_game((IntPtr)(ulong)gamePtr);
        }

        public int MazeGameMovePlayer(UIntPtr gamePtr, int dir)
        {
            return maze_c_maze_game_move_player((IntPtr)(ulong)gamePtr, dir);
        }

        public int MazeGamePlayerRow(UIntPtr gamePtr)
        {
            return maze_c_maze_game_player_row((IntPtr)(ulong)gamePtr);
        }

        public int MazeGamePlayerCol(UIntPtr gamePtr)
        {
            return maze_c_maze_game_player_col((IntPtr)(ulong)gamePtr);
        }

        public int MazeGamePlayerDirection(UIntPtr gamePtr)
        {
            return maze_c_maze_game_player_direction((IntPtr)(ulong)gamePtr);
        }

        public int MazeGameIsComplete(UIntPtr gamePtr)
        {
            return maze_c_maze_game_is_complete((IntPtr)(ulong)gamePtr);
        }

        public int MazeGameIsLost(UIntPtr gamePtr)
        {
            return maze_c_maze_game_is_lost((IntPtr)(ulong)gamePtr);
        }

        public int MazeGameLoseReason(UIntPtr gamePtr)
        {
            return maze_c_maze_game_lose_reason((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGamePickup(UIntPtr gamePtr, out MazeInterop.MazeBagItem item)
        {
            byte result = maze_c_maze_game_pickup((IntPtr)(ulong)gamePtr, out uint kind, out uint id);
            item = new MazeInterop.MazeBagItem { Kind = (MazeInterop.MazeBagItemKind)kind, Id = id };
            return result != 0;
        }

        public int MazeGameBagCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_bag_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetBagItem(UIntPtr gamePtr, int index, out MazeInterop.MazeBagItem item)
        {
            byte result = maze_c_maze_game_get_bag_item((IntPtr)(ulong)gamePtr, index, out uint kind, out uint id);
            item = new MazeInterop.MazeBagItem { Kind = (MazeInterop.MazeBagItemKind)kind, Id = id };
            return result != 0;
        }

        public int MazeGameDoorCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_door_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetDoor(UIntPtr gamePtr, int index, out MazeInterop.MazeDoor door)
        {
            byte result = maze_c_maze_game_get_door((IntPtr)(ulong)gamePtr, index, out uint row, out uint col, out uint state);
            door = new MazeInterop.MazeDoor { Row = row, Column = col, State = (MazeInterop.MazeDoorState)state };
            return result != 0;
        }

        public int MazeGameTick(UIntPtr gamePtr, float dtMs)
        {
            return maze_c_maze_game_tick((IntPtr)(ulong)gamePtr, dtMs);
        }

        public int MazeGameTickEventCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_tick_event_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetTickEvent(UIntPtr gamePtr, int index, out MazeInterop.MazeGameEvent evt)
        {
            byte result = maze_c_maze_game_get_tick_event((IntPtr)(ulong)gamePtr, index, out uint kind, out uint row, out uint col);
            uint payload = 0;
            if (result != 0)
                maze_c_maze_game_get_tick_event_payload((IntPtr)(ulong)gamePtr, index, out payload);
            evt = new MazeInterop.MazeGameEvent { Kind = (MazeInterop.MazeGameEventKind)kind, Row = row, Column = col, Payload = payload };
            return result != 0;
        }

        public int MazeGameKeyCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_key_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetKey(UIntPtr gamePtr, int index, out MazeInterop.MazeKey key)
        {
            byte result = maze_c_maze_game_get_key((IntPtr)(ulong)gamePtr, index, out uint row, out uint col, out uint id);
            key = new MazeInterop.MazeKey { Row = row, Column = col, Id = id };
            return result != 0;
        }

        public uint MazeGameHp(UIntPtr gamePtr)
        {
            return maze_c_maze_game_hp((IntPtr)(ulong)gamePtr);
        }

        public uint MazeGameMaxHp(UIntPtr gamePtr)
        {
            return maze_c_maze_game_max_hp((IntPtr)(ulong)gamePtr);
        }

        public int MazeGameEnemyCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_enemy_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetEnemy(UIntPtr gamePtr, int index, out MazeInterop.MazeEnemy enemy)
        {
            byte result = maze_c_maze_game_get_enemy((IntPtr)(ulong)gamePtr, index, out uint row, out uint col, out uint id, out uint damage, out float movePeriodMs, out int enemyType);
            enemy = new MazeInterop.MazeEnemy { Row = row, Column = col, Id = id, Damage = damage, MovePeriodMs = movePeriodMs, EnemyType = enemyType };
            return result != 0;
        }

        public int MazeGameHealthPickupCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_health_pickup_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetHealthPickup(UIntPtr gamePtr, int index, out MazeInterop.MazeHealthPickup pickup)
        {
            byte result = maze_c_maze_game_get_health_pickup((IntPtr)(ulong)gamePtr, index, out uint row, out uint col, out uint id);
            pickup = new MazeInterop.MazeHealthPickup { Row = row, Column = col, Id = id };
            return result != 0;
        }

        public int MazeGameTreasureCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_treasure_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetTreasure(UIntPtr gamePtr, int index, out MazeInterop.MazeTreasure treasure)
        {
            byte result = maze_c_maze_game_get_treasure((IntPtr)(ulong)gamePtr, index, out uint row, out uint col, out int style, out uint value);
            treasure = new MazeInterop.MazeTreasure { Row = row, Column = col, Style = style, Value = value };
            return result != 0;
        }

        public int MazeGameCollectedTreasureCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_collected_treasure_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetCollectedTreasure(UIntPtr gamePtr, int index, out MazeInterop.MazeCollectedTreasure collected)
        {
            byte result = maze_c_maze_game_get_collected_treasure((IntPtr)(ulong)gamePtr, index, out int style, out uint count);
            collected = new MazeInterop.MazeCollectedTreasure { Style = style, Count = count };
            return result != 0;
        }

        public int MazeGameVisitedCellCount(UIntPtr gamePtr)
        {
            return maze_c_maze_game_visited_cell_count((IntPtr)(ulong)gamePtr);
        }

        public bool MazeGameGetVisitedCell(UIntPtr gamePtr, int index, out int row, out int col)
        {
            byte result = maze_c_maze_game_get_visited_cell((IntPtr)(ulong)gamePtr, index, out row, out col);
            return result != 0;   // maze_c: 1 = success
        }
    }
}
#endif
