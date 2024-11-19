using Microsoft.Maui.Controls;

namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `CellFrame` class represents a grid cell frame
    /// </summary>
    public class CellFrame : Border
    {
        /// <summary>
        /// The row index of the cell within the grid definition
        /// </summary>
        /// <returns>Row index</returns>
        public int Row { get; set; }
        /// <summary>
        /// The column index of the cell within the grid definition
        /// </summary>
        /// <returns>Column index</returns>
        public int Column { get; set; }
        /// <summary>
        /// The display row of the cell within the grid display, accounting for any header rows and columns
        /// </summary>
        /// <returns>Display row</returns>
        public int DisplayRow { get => Row + 1; }
        /// <summary>
        /// The display column of the cell within the grid display, accounting for any header rows and columns
        /// </summary>
        /// <returns>Display column</returns>
        public int DisplayColumn { get => Column + 1; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="row">Row index within the grid definition</param>
        /// <param name="column">Column index within the grid definition</param>
        public CellFrame(int row, int column)
        {
            this.Row = row;
            this.Column = column;
        }
        /// <summary>
        /// Checks whether the cell is associated with a given display position
        /// </summary>
        /// <param name="displayRow">Display row</param>
        /// <param name="displayColumn">Display column</param>
        public bool IsDisplayPosition(int displayRow, int displayColumn)
        {
            return displayRow == DisplayRow && displayColumn == DisplayColumn;
        }
    }
}
