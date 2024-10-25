namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class SelectionFrame
    {
        private readonly Grid parent;

        private const int BORDER_BOX_TOP = 0;
        private const int BORDER_BOX_LEFT = 1;
        private const int BORDER_BOX_BOTTOM = 2;
        private const int BORDER_BOX_RIGHT = 3;

        private const int BORDER_BOX_FIRST = BORDER_BOX_TOP;
        private const int BORDER_BOX_LAST = BORDER_BOX_RIGHT;

        private readonly BoxView[] borderBoxes = new BoxView[4];

        //private List<BoxView>? selectionFrameGrips;

        static private Color DEFAULT_BORDER_COLOR = Colors.Black;
        const double DEFAULT_BORDER_WIDTH = 2.0;

        private Color borderColor = DEFAULT_BORDER_COLOR;

        public Color BorderColor
        {
            get => borderColor;
            set
            {
                borderColor = value;
                UpdateBorderColor();
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

        public int TopRow { get { return CellRange != null ? CellRange.Top : -1; } }

        public int BottomRow { get { return CellRange != null ? CellRange.Bottom : -1; } }

        public int LeftColumn { get { return CellRange != null ? CellRange.Left : -1; } }

        public int RightColumn { get { return CellRange != null ? CellRange.Right : -1; } }

        public double BorderWidth { get; set; } = DEFAULT_BORDER_WIDTH;

        public SelectionFrame(Grid parent)
        {
            this.parent = parent;
            CreateBorderBoxes();
        }

        private void CreateBorderBoxes()
        {
            for (int i = BORDER_BOX_FIRST; i <= BORDER_BOX_LAST; i++)
                borderBoxes[i] = NewBorderBox();
        }

        private BoxView NewBorderBox()
        {
            return new BoxView
            {
                Color = BorderColor,
                IsVisible = false,
                HeightRequest = 0,
                WidthRequest = 0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = true
            };
        }

        private void UpdateBorderColor()
        {
            foreach (var box in borderBoxes)
                box.Color = BorderColor;
        }


        public void Show(bool show)
        {
            foreach (var box in borderBoxes)
                box.IsVisible = show;
        }

        public void AddToGrid()
        {
            foreach (var box in borderBoxes)
            {
                parent.SetRow(box, 0);
                parent.SetColumn(box, 0);
                parent.Children.Add(box);
            }
        }

        public void SetRange(CellRange newRange, bool show)
        {
            CellRange = newRange.Clone();

            double width = parent.GetCellsWidth(CellRange);
            double height = parent.GetCellsHeight(CellRange);

            SetBorder(BORDER_BOX_TOP, TopRow, LeftColumn, width, BorderWidth, new Thickness(0));
            SetBorder(BORDER_BOX_LEFT, TopRow, LeftColumn, BorderWidth, height, new Thickness(0));
            SetBorder(BORDER_BOX_BOTTOM, BottomRow, LeftColumn, width, BorderWidth,
                new Thickness(0, parent.GetRowHeight(BottomRow) - BorderWidth, 0, 0));
            SetBorder(BORDER_BOX_RIGHT, TopRow, RightColumn, BorderWidth, height,
                new Thickness(parent.GetColumnWidth(RightColumn) - BorderWidth, 0, 0, 0));
            Show(show);
        }

        private void SetBorder(int borderID, int row, int column, double width, double height, Thickness margin)
        {
            BoxView box = borderBoxes[borderID];
            parent.SetRow(box, row);
            parent.SetColumn(box, column);
            box.WidthRequest = width;
            box.HeightRequest = height;
            box.Margin = margin;
        }
    }
}


// Initialize grip points (one for each side: top, bottom, left, right)
/*      selectionFrameGrips = new List<BoxView>();
      for (int i = 0; i < 4; i++)
      {
          var grip = new BoxView
          {
              Color = Colors.Red,
              WidthRequest = 20,
              HeightRequest = 20,
              CornerRadius = 10,
              IsVisible = false
          };
          selectionFrameGrips.Add(grip);
          this.Children.Add(grip); 
      }
*/
