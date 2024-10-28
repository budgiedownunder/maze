namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class SelectionFrame
    {
        private readonly Grid parentGrid;

        private const int BORDER_ID_TOP = 0;
        private const int BORDER_ID_LEFT = 1;
        private const int BORDER_ID_BOTTOM = 2;
        private const int BORDER_ID_RIGHT = 3;

        private const int BORDER_ID_FIRST = BORDER_ID_TOP;
        private const int BORDER_ID_LAST = BORDER_ID_RIGHT;

        static private Color DEFAULT_BORDER_COLOR = Colors.Black;
        const double DEFAULT_BORDER_WIDTH = 2.0;
        const double DEFAULT_BORDER_GRIP_SIZE = 4.0;


        private Color borderColor = DEFAULT_BORDER_COLOR;
        private double borderWidth = DEFAULT_BORDER_WIDTH;
        private double borderGripSize = DEFAULT_BORDER_GRIP_SIZE;

        private readonly FrameBorder[] frameBorders = new FrameBorder[4];

        public Color BorderColor
        {
            get => borderColor;
            set
            {
                borderColor = value;
                UpdateBorders();
            }
        }

        public double BorderWidth
        {
            get => borderWidth;
            set
            {
                borderWidth = value;
                UpdateBorders();
            }
        }

        public double BorderGripSize
        {
            get => borderGripSize;
            set
            {
                borderGripSize = value;
                UpdateBorders();
            }
        }

        private CellRange? cellRange;
        public CellRange? CellRange
        {
            get { return cellRange; }
            set
            {
                cellRange = value;
            }
        }

        public Grid ParentGrid { get => parentGrid; }

        public int TopRow { get { return CellRange != null ? CellRange.Top : -1; } }

        public int BottomRow { get { return CellRange != null ? CellRange.Bottom : -1; } }

        public int RowCount { get { return CellRange != null ? CellRange.Height : 0; } }

        public int LeftColumn { get { return CellRange != null ? CellRange.Left : -1; } }

        public int RightColumn { get { return CellRange != null ? CellRange.Right : -1; } }

        public int ColumnCount { get { return CellRange != null ? CellRange.Width : 0; } }

        public SelectionFrame(Grid parentGrid)
        {
            this.parentGrid = parentGrid;
            CreateBorders();
        }

        private void CreateBorders()
        {
            for (int id = BORDER_ID_FIRST; id <= BORDER_ID_LAST; id++)
                frameBorders[id] = NewFrameBorder(ToFrameBorderLocation(id));
        }

        private FrameBorder.FrameLocation ToFrameBorderLocation(int borderID)
        {
            switch (borderID)
            {
                case BORDER_ID_TOP:
                    return FrameBorder.FrameLocation.Top;
                case BORDER_ID_LEFT:
                    return FrameBorder.FrameLocation.Left;
                case BORDER_ID_BOTTOM:
                    return FrameBorder.FrameLocation.Bottom;
                case BORDER_ID_RIGHT:
                    return FrameBorder.FrameLocation.Right;
            }
            return FrameBorder.FrameLocation.Top;
        }

        private FrameBorder NewFrameBorder(FrameBorder.FrameLocation location)
        {
            return new FrameBorder(this)
            {
                Location = location,
                Color = BorderColor,
                GripSize = BorderGripSize
            };
        }

        private void UpdateBorders()
        {
            foreach (var border in frameBorders)
            {
                border.Color = BorderColor;
                border.GripSize = BorderGripSize;
            }
        }

        public void Show(bool show)
        {
            foreach (var border in frameBorders)
                border.Show(show);
        }

        public void AddToGrid()
        {
            foreach (var border in frameBorders)
                border.AddToGrid();
        }

        public void SetRange(CellRange newRange, bool show)
        {
            CellRange = newRange.Clone();

            double width = ParentGrid.GetCellsWidth(CellRange);
            double height = ParentGrid.GetCellsHeight(CellRange);

            SetBorder(BORDER_ID_TOP, TopRow, LeftColumn, width, BorderWidth, new Thickness(0));
            SetBorder(BORDER_ID_LEFT, TopRow, LeftColumn, BorderWidth, height, new Thickness(0));
            SetBorder(BORDER_ID_BOTTOM, BottomRow, LeftColumn, width, BorderWidth,
                new Thickness(0, ParentGrid.GetRowHeight(BottomRow) - BorderWidth, 0, 0));
            SetBorder(BORDER_ID_RIGHT, TopRow, RightColumn, BorderWidth, height,
                new Thickness(ParentGrid.GetColumnWidth(RightColumn) - BorderWidth, 0, 0, 0));
            Show(show);
        }

        private void SetBorder(int borderID, int row, int column, double width, double height, Thickness margin)
        {
            FrameBorder border = frameBorders[borderID];
            border.SetPosition(row, column, width, height, margin);
        }

    }
}
