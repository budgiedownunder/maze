using Maze.Api;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// The per-cell-override operations the cell-override panel drives on the editor
    /// grid. Abstracted from <see cref="MazeGrid"/> so the panel's view model can be
    /// unit-tested against a mock. All coordinates are one-based.
    /// </summary>
    public interface ICellOverrideEditor
    {
        /// <summary>The maze's game settings, supplying the wall/enemy/health defaults the
        /// panel inherits when a cell carries no per-cell override (drives the "Default"
        /// tier-1 texture visibility and the maze-default previews). Null when unset.</summary>
        MazeGameSettings? GameSettings { get; }

        /// <summary>The override on the cell, or null when it carries none.</summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        /// <returns>The cell's override, or null</returns>
        CellEntityInfo? GetCellOverride(int row, int column);

        /// <summary>Sets (or replaces) the override on the cell.</summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        /// <param name="entity">The override to apply</param>
        void SetCellOverride(int row, int column, CellEntityInfo entity);

        /// <summary>Clears the override on the cell, if any.</summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        void ClearCellOverride(int row, int column);

        /// <summary>Re-renders the cell so an override change shows immediately.</summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        void RefreshCellContent(int row, int column);
    }
}
