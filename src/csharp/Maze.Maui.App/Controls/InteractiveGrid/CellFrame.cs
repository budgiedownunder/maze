using Microsoft.Maui.Controls;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class CellFrame : Border
    {
        public int Row { get; set; }

        public int Column { get; set; }

        public int DisplayRow { get => Row + 1; }

        public int DisplayColumn { get => Column + 1; }

        public CellFrame(int row, int column)
        {
            this.Row = row;
            this.Column = column;
        }

        public bool IsPosition(int row, int column)
        {
            return row == DisplayRow && column == DisplayColumn;
        }
    }
}
