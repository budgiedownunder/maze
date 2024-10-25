namespace Maze.Maui.App.Controls.InteractiveGrid
{
    internal class CellRange
    {
        public int Left { get; set; } = 0;
        public int Right { get; set; } = 0;

        public int Top { get; set; } = 0;

        public int Bottom { get; set; } = 0;

        public int Width
        {
            get
            {
                return this.Right - this.Left + 1;
            }
        }

        public int Height
        {
            get
            {
                return this.Bottom - this.Top + 1;
            }
        }

        public CellRange(int top, int left, int bottom, int right)
        {
            Top = Math.Min(top, bottom);
            Left = Math.Min(left, right);
            Bottom = Math.Max(top, bottom);
            Right = Math.Max(left, right);
        }

        public CellRange(int row, int col)
        {
            Top = row;
            Left = col;
            Bottom = Top;
            Right = Left;
        }

        public CellRange(CellPoint point)
        {
            Top = point.Row;
            Left = point.Col;
            Bottom = Top;
            Right = Left;
        }

        public CellRange Clone()
        {
            return (CellRange)this.MemberwiseClone();
        }
    }
}
