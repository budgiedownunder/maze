namespace Maze.Maui.App.Controls.InteractiveGrid
{
    /// <summary>
    /// The `CellRange` class represents a rectangular grid cell range
    /// </summary>
    public class CellRange
    {
        /// <summary>
        /// The left column number associated with the range
        /// </summary>
        /// <returns>Left column number</returns>
        public int Left { get; set; } = 0;
        /// <summary>
        /// The right column number associated with the range
        /// </summary>
        /// <returns>Right column number</returns>
        public int Right { get; set; } = 0;
        /// <summary>
        /// The top row number associated with the range
        /// </summary>
        /// <returns>Top row number</returns>
        public int Top { get; set; } = 0;
        /// <summary>
        /// The bottom row number associated with the range
        /// </summary>
        /// <returns>Bottom row number</returns>
        public int Bottom { get; set; } = 0;
        /// <summary>
        /// The width of the range (`Right - Left + 1`)
        /// </summary>
        /// <returns>Width</returns>
        public int Width
        {
            get => Right - Left + 1;
        }
        /// <summary>
        /// The height of the range (`Bottom - Top + 1`)
        /// </summary>
        /// <returns>Height</returns>
        public int Height
        {
            get => Bottom - Top + 1;
        }
        /// <summary>
        /// The number of cells in the range (`Width * Height`)
        /// </summary>
        /// <returns>Number of cells</returns>
        public int CellCount
        {
            get => Width * Height;
        }
        /// <summary>
        /// Indicates whether the the range corresponds to a single cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsSingleCell
        {
            get => CellCount == 1;
        }
        /// <summary>
        /// Constructor (range)
        /// </summary>
        /// <param name="top">Top row number</param>
        /// <param name="left">Left column number</param>
        /// <param name="bottom">Bottom row number</param>
        /// <param name="right">Right column number</param>
        /// <returns>Boolean</returns>
        public CellRange(int top, int left, int bottom, int right)
        {
            Top = Math.Min(top, bottom);
            Left = Math.Min(left, right);
            Bottom = Math.Max(top, bottom);
            Right = Math.Max(left, right);
        }
        /// <summary>
        /// Constructor (single cell)
        /// </summary>
        /// <param name="row">Row number</param>
        /// <param name="column">Column number</param>
        public CellRange(int row, int column)
        {
            Top = row;
            Left = column;
            Bottom = Top;
            Right = Left;
        }
        /// <summary>
        /// Constructor (point)
        /// </summary>
        /// <param name="point">Point</param>
        public CellRange(CellPoint point)
        {
            Top = point.Row;
            Left = point.Column;
            Bottom = Top;
            Right = Left;
        }
        /// <summary>
        /// Creates a copy of the object
        /// </summary>
        /// <returns>Copy of the object</returns>
        public CellRange Clone()
        {
            return (CellRange)this.MemberwiseClone();
        }
        /// <summary>
        /// Checks whether the range matches a comparison range
        /// </summary>
        /// <param name="compare">Comparison range</param>
        /// <returns>Boolean</returns>
        public bool Equals(CellRange? compare)
        {
            if (compare == null) return false;

            return this.Top == compare.Top &&
                this.Left == compare.Left &&
                this.Bottom == compare.Bottom &&
                this.Right == compare.Right;
        }
        /// <summary>
        /// Checks whether the range includes a given row
        /// </summary>
        /// <param name="row">Row to check</param>
        /// <returns>Boolean</returns>
        public bool ContainsRow(int row)
        {
            return row >= Top && row <= Bottom;
        }
        /// <summary>
        /// Checks whether the range includes a given column
        /// </summary>
        /// <param name="column">Column to check</param>
        /// <returns>Boolean</returns>
        public bool ContainsColumn(int column)
        {
            return column >= Left && column <= Right;
        }
        /// <summary>
        /// Checks whether the range includes a given row and column
        /// </summary>
        /// <param name="row">Row to check</param>
        /// <param name="column">Column to check</param>
        /// <returns>Boolean</returns>
        public bool ContainsPosition(int row, int column)
        {
            return ContainsRow(row) && ContainsColumn(column);
        }
        /// <summary>
        /// Checks whether the range includes a given point
        /// </summary>
        /// <param name="point">Point to check</param>
        /// <returns>Boolean</returns>
        public bool ContainsPoint(CellPoint point)
        {
            return ContainsPosition(point.Row, point.Column);
        }
        /// <summary>
        /// Restricts the rows associated with the object so that they do not exceed a given maximum row number
        /// </summary>
        /// <param name="maxRow">Maximum row number</param>
        public void ClampRows(int maxRow)
        {
            if (Top > maxRow)
                Top = maxRow;

            if (Bottom > maxRow)
                Bottom = maxRow;
        }
        /// <summary>
        /// Restricts the columns associated with the object so that they do not exceed a given maximum column number
        /// </summary>
        /// <param name="maxColumn">Maximum column number</param>
        public void ClampColumns(int maxColumn)
        {
            if (Left > maxColumn)
                Left = maxColumn;

            if (Right > maxColumn)
                Right = maxColumn;
        }
    }
}
