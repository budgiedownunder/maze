namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class CellPoint
    {
        public int Row { get; set; } = -1;
        public int Column { get; set; } = -1;

        public CellPoint(int row = -1, int column = -1)
        {
            Set(row, column);
        }

        public CellPoint Clone()
        {
            return (CellPoint)this.MemberwiseClone();
        }
        public bool IsPosition(int row, int column)
        {
            return Row == row && Column == column;
        }

        public void Clear()
        {
            Set(-1, -1);
        }

        public void Set(int row, int column)
        {
            Row = row;
            Column = column;
        }

        public void ClampRow(int maxRow)
        {
            if (Row > maxRow)
                Row = maxRow;
        }

        public void ClampColumn(int maxColumn)
        {
            if (Column > maxColumn)
                Column = maxColumn;
        }

    }
}
