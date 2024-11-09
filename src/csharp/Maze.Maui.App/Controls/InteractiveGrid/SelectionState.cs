using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class SelectionState
    {
        CellRange? selectedCells;
        CellPoint? activeCellPoint;
        CellPoint? anchorCellPoint;

        public CellRange? SelectedCells { get => selectedCells; }

        public CellPoint? ActiveCellPoint { get => anchorCellPoint; }

        public CellPoint? AnchorCellPoint { get => anchorCellPoint; }

        public SelectionState(CellRange? selectedCells, CellPoint? activeCellPoint, CellPoint? anchorCellPOint)
        {
            this.selectedCells = selectedCells;
            this.activeCellPoint = activeCellPoint;
            this.anchorCellPoint = anchorCellPOint;
        }

        public void ClampRows(int maxRow)
        {
            if (activeCellPoint != null)
                activeCellPoint.ClampRow(maxRow);

            if (anchorCellPoint != null)
                anchorCellPoint.ClampRow(maxRow);

            if (selectedCells != null)
                selectedCells.ClampRows(maxRow);
        }

        public void ClampColumns(int maxColumn)
        {
            if (activeCellPoint != null)
                activeCellPoint.ClampColumn(maxColumn);

            if (anchorCellPoint != null)
                anchorCellPoint.ClampColumn(maxColumn);

            if (selectedCells != null)
                selectedCells.ClampColumns(maxColumn);
        }

    }
}
