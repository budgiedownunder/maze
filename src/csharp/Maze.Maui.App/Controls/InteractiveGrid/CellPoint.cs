namespace Maze.Maui.App.Controls.InteractiveGrid
{
    /// <summary>
    /// The `CellPoint` class represents a grid cell point location
    /// </summary>
    public class CellPoint
    {
        /// <summary>
        /// The row number associated with the point. Default value is -1, indicating no assigned row number.
        /// </summary>
        /// <returns>Row number</returns>
        public int Row { get; set; } = -1;
        /// <summary>
        /// The column number associated with the point. Default value is -1, indicating no assigned column number.
        /// </summary>
        /// <returns>Row number</returns>
        public int Column { get; set; } = -1;
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="row">Row number (default = -1, indicating no row number assigned)</param>
        /// <param name="column">Column number (default = -1, indicating no column number assigned)</param>
        public CellPoint(int row = -1, int column = -1)
        {
            Set(row, column);
        }
        /// <summary>
        /// Returns a copy of the object
        /// </summary>
        /// <returns>Copy of the object</returns>
        public CellPoint Clone()
        {
            return (CellPoint)this.MemberwiseClone();
        }
        /// <summary>
        /// Checks whether the object corresponds to a given position
        /// </summary>
        /// <param name="row">Row number to check</param>
        /// <param name="column">Column number to check</param>
        /// <returns>Boolean</returns>
        public bool IsPosition(int row, int column)
        {
            return Row == row && Column == column;
        }
        /// <summary>
        /// Clears the object, resetting it to an empty state
        /// </summary>
        public void Clear()
        {
            Set(-1, -1);
        }
        /// <summary>
        /// Sets the row and column numbers associated with the object
        /// </summary>
        /// <param name="row">New row number</param>
        /// <param name="column">New column number</param>
        public void Set(int row, int column)
        {
            Row = row;
            Column = column;
        }
        /// <summary>
        /// Restricts the row associated with the object so that it does not exceed a given maximum row number
        /// </summary>
        public void ClampRow(int maxRow)
        {
            if (Row > maxRow)
                Row = maxRow;
        }
        /// <summary>
        /// Restricts the column associated with the object so that it does not exceed a given maximum column number
        /// </summary>
        public void ClampColumn(int maxColumn)
        {
            if (Column > maxColumn)
                Column = maxColumn;
        }
    }
}
