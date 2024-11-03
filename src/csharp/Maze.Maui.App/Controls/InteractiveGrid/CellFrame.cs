using Microsoft.Maui.Controls;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class CellFrame : Frame
    {
        int row;
        int column;

        public int DisplayRow { get => row + 1; }

        public int DisplayColumn { get => column + 1; }

        public CellFrame(int row, int column)
        {
            this.row = row;
            this.column = column;
        }

        public bool IsPosition(int row, int column)
        {
            return row == DisplayRow && column == DisplayColumn;
        }

    }
}
