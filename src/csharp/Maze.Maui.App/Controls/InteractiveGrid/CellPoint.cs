namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class CellPoint
    {
        public int Row { get; set; } = -1;
        public int Column { get; set; } = -1;

        public CellPoint Clone()
        {
            return (CellPoint)this.MemberwiseClone();
        }
        public bool IsPosition(int row, int column)
        {
            return Row == row && Column == column;
        }
    }
}
