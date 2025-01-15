namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `SelectionFrane` class holds a selection state for a grid
    /// </summary>
    public class SelectionState
    {
        // Private properties
        CellRange? selectedCells;
        CellPoint? activeCellPoint;
        CellPoint? anchorCellPoint;

        /// <summary>
        /// Selected cell range
        /// </summary>
        /// <returns>Selected cell range</returns>
        public CellRange? SelectedCells { get => selectedCells; }
        /// <summary>
        /// Active cell point
        /// </summary>
        /// <returns>Active cell point</returns>
        public CellPoint? ActiveCellPoint { get => activeCellPoint; }
        /// <summary>
        /// Anchor cell point
        /// </summary>
        /// <returns>Anchor cell point</returns>
        public CellPoint? AnchorCellPoint { get => anchorCellPoint; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="selectedCells">Selected cell range</param>
        /// <param name="activeCellPoint">Active cell point</param>
        /// <param name="anchorCellPoint">Anchor cell point</param>
        public SelectionState(CellRange? selectedCells, CellPoint? activeCellPoint, CellPoint? anchorCellPoint)
        {
            this.selectedCells = selectedCells;
            this.activeCellPoint = activeCellPoint;
            this.anchorCellPoint = anchorCellPoint;
        }
        /// <summary>
        /// Restricts the rows associated with the object so that they do not exceed the given maximum row number
        /// </summary>
        /// <param name="maxRow">Maximum row number</param>
        public void ClampRows(int maxRow)
        {
            if (activeCellPoint is not null)
                activeCellPoint.ClampRow(maxRow);

            if (anchorCellPoint is not null)
                anchorCellPoint.ClampRow(maxRow);

            if (selectedCells is not null)
                selectedCells.ClampRows(maxRow);
        }
        /// <summary>
        /// Restricts the columns associated with the object so that they do not exceed the given maximum column number
        /// </summary>
        /// <param name="maxColumn">Maximum column number</param>
        public void ClampColumns(int maxColumn)
        {
            if (activeCellPoint is not null)
                activeCellPoint.ClampColumn(maxColumn);

            if (anchorCellPoint is not null)
                anchorCellPoint.ClampColumn(maxColumn);

            if (selectedCells is not null)
                selectedCells.ClampColumns(maxColumn);
        }
    }
}
