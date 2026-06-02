namespace Maze.Maui.App.Utils;

using ApiMaze = global::Maze.Api.Maze;

/// <summary>
/// Cell-tally helpers for an <see cref="ApiMaze"/>. Walks the grid via
/// <see cref="ApiMaze.GetCellType"/> — the cost is one Interop call per
/// cell, fine for the editor's "count once per Save / count once per
/// Generate-popup-open" usage. Mirrors React's
/// <c>countKeysAndDoors</c> / <c>defaultsFromGrid</c> helpers.
/// </summary>
internal static class MazeCellCounter
{
    /// <summary>
    /// Returns how many cells in <paramref name="maze"/> have
    /// <paramref name="cellType"/>.
    /// </summary>
    internal static uint CountCellsOfType(ApiMaze maze, ApiMaze.CellType cellType)
    {
        uint count = 0;
        uint rows = maze.RowCount;
        uint cols = maze.ColCount;
        for (uint r = 0; r < rows; r++)
        {
            for (uint c = 0; c < cols; c++)
            {
                if (maze.GetCellType(r, c) == cellType)
                    count++;
            }
        }
        return count;
    }

    /// <summary>
    /// Returns the combined <c>'K'</c> + <c>'D'</c> counts in
    /// <paramref name="maze"/>. Used by the editor save guard to refuse
    /// over-cap saves up-front (mirrors React's <c>countKeysAndDoors</c>
    /// in <c>utils/validation.ts</c>).
    /// </summary>
    internal static (uint keys, uint doors) CountKeysAndDoors(ApiMaze maze)
    {
        uint keys = 0, doors = 0;
        uint rows = maze.RowCount;
        uint cols = maze.ColCount;
        for (uint r = 0; r < rows; r++)
        {
            for (uint c = 0; c < cols; c++)
            {
                switch (maze.GetCellType(r, c))
                {
                    case ApiMaze.CellType.Key: keys++; break;
                    case ApiMaze.CellType.Door: doors++; break;
                }
            }
        }
        return (keys, doors);
    }
}
