namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class CellRange
    {
        public int Left { get; set; } = 0;
        public int Right { get; set; } = 0;

        public int Top { get; set; } = 0;

        public int Bottom { get; set; } = 0;

        public int Width
        {
            get => Right - Left + 1;
        }

        public int Height
        {
            get=> Bottom - Top + 1;
        }

        public int CellCount
        {
            get => Width * Height;
        }

        public CellRange(int top, int left, int bottom, int right)
        {
            Top = Math.Min(top, bottom);
            Left = Math.Min(left, right);
            Bottom = Math.Max(top, bottom);
            Right = Math.Max(left, right);
        }

        public CellRange(int row, int column)
        {
            Top = row;
            Left = column;
            Bottom = Top;
            Right = Left;
        }

        public CellRange(CellPoint point)
        {
            Top = point.Row;
            Left = point.Column;
            Bottom = Top;
            Right = Left;
        }

        public CellRange Clone()
        {
            return (CellRange)this.MemberwiseClone();
        }

        public bool Equals(CellRange? compare)
        {
            if (compare == null) return false;

            return this.Top == compare.Top &&
                this.Left == compare.Left &&
                this.Bottom == compare.Bottom &&
                this.Right == compare.Right;
        }

        public bool ContainsRow(int row)
        {
            return row >= Top && row <= Bottom;
        }

        public bool ContainsColumn(int column)
        {
            return column >= Left && column <= Right;
        }

        public bool ContainsPoint(CellPoint point)
        {
            return ContainsRow(point.Row) && ContainsColumn(point.Column);
        }

    }
}
