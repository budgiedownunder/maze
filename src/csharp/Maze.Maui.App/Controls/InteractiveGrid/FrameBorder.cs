using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class FrameBorder
    {
        SelectionFrame parentFrame;

        private readonly BoxView box;
        private readonly BoxView grip;

        static private Color DEFAULT_COLOR = Colors.Black;
        const double DEFAULT_WIDTH = 2.0;
        const double DEFAULT_GRIP_SIZE = 4.0;

        Color color = DEFAULT_COLOR;
        double gripSize = DEFAULT_GRIP_SIZE;

        public enum FrameLocation
        {
            Top = 0,
            Left = 1,
            Bottom = 2,
            Right = 3
        }

        public SelectionFrame ParentFrame { get => parentFrame; }

        public Grid ParentGrid { get => ParentFrame.ParentGrid; }

        public FrameLocation Location { get; set; } = FrameLocation.Top;

        public Color Color
        {
            get => color;
            set
            {
                color = value;
                UpdateColor();
            }
        }

        public double GripSize
        {
            get => gripSize;
            set
            {
                gripSize = value;
                UpdateGripSize();
            }
        }

        public FrameBorder(SelectionFrame parentFrame)
        {
            this.parentFrame = parentFrame;
            box = NewBox();
            grip = NewGrip();
        }

        private BoxView NewBox()
        {
            BoxView box = new BoxView
            {
                Color = Color,
                IsVisible = false,
                HeightRequest = 0,
                WidthRequest = 0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = false
            };
            RegisterPointerEventHandlers(box, false);
            return box;
        }

        private BoxView NewGrip()
        {
            BoxView grip = new BoxView
            {
                Color = Color,
                IsVisible = false,
                HeightRequest = GripSize,
                WidthRequest = GripSize,
                CornerRadius = GripSize / 2.0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = false
            };
            RegisterPointerEventHandlers(grip, true);
            return grip;
        }

        private void RegisterPointerEventHandlers(BoxView view, bool isGrip)
        {
            var pointerGestureRecognizer = new PointerGestureRecognizer();
            pointerGestureRecognizer.PointerEntered += (s, e) =>
            {
                OnPointerEntered(view, getPointerIcon(isGrip));
            };
            pointerGestureRecognizer.PointerExited += (s, e) =>
            {
                OnPointerExited(view);
            };
            view.GestureRecognizers.Add(pointerGestureRecognizer);
        }

        private void OnPointerEntered(BoxView view, Icon icon)
        {
            Pointer.Pointer.SetCursor(view, icon);
        }

        private void OnPointerExited(BoxView view)
        {
            Pointer.Pointer.SetCursor(view, Icon.Arrow);
        }

        private Icon getPointerIcon(bool isGrip)
        {
            switch (Location)
            {
                case FrameLocation.Top:
                    return isGrip ? Icon.SizeNorthWestSouthEast : Icon.SizeNorthSouth;
                case FrameLocation.Left:
                    return isGrip ? Icon.SizeNorthEastSouthWest : Icon.SizeWestEast;
                case FrameLocation.Bottom:
                    return isGrip ? Icon.SizeNorthWestSouthEast : Icon.SizeNorthSouth;
                case FrameLocation.Right:
                    return isGrip ? Icon.SizeNorthEastSouthWest : Icon.SizeWestEast;
            }
            return isGrip ? Icon.SizeNorthWestSouthEast : Icon.SizeNorthSouth;
        }

        private void UpdateColor()
        {
            box.Color = Color;
            grip.Color = Color;
        }

        private void UpdateGripSize()
        {
            grip.HeightRequest = GripSize;
            grip.WidthRequest = GripSize;
            grip.CornerRadius = GripSize / 2.0;
        }

        public void Show(bool show)
        {
            box.IsVisible = show;
            grip.IsVisible = show;
        }

        public void AddToGrid()
        {
            AddViewToGrid(box);
            AddViewToGrid(grip);
        }

        private void AddViewToGrid(BoxView view)
        {
            ParentGrid.SetRow(view, 0);
            ParentGrid.SetColumn(view, 0);
            ParentGrid.Children.Add(view);
        }

        public void SetPosition(int row, int column, double width, double height, Thickness margin)
        {
            ParentGrid.SetRow(box, row);
            ParentGrid.SetColumn(box, column);
            box.WidthRequest = width;
            box.HeightRequest = height;
            box.Margin = margin;

            ParentGrid.SetRow(grip, row);
            ParentGrid.SetColumn(grip, column);
            grip.TranslationX = GetGripTranslationX(width);
            grip.TranslationY = GetGripTranslationY(height);
            grip.Margin = margin;
        }

        private double GetGripTranslationX(double width)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Location)
            {
                case FrameLocation.Top:
                    return -GripSize / 2.0;
                case FrameLocation.Left:
                    return -GripSize / 2.0;
                case FrameLocation.Bottom:
                    return width - GripSize / 2.0; ;
                case FrameLocation.Right:
                    return 1.0 - GripSize / 2.0;
            }
            return 0.0;
        }

        private double GetGripTranslationY(double height)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Location)
            {
                case FrameLocation.Top:
                    return -GripSize / 2.0;
                case FrameLocation.Left:
                    return height - GripSize / 2.0;
                case FrameLocation.Bottom:
                    return height - GripSize / 2.0;
                case FrameLocation.Right:
                    return -GripSize / 2.0;
            }
            return 0.0;
        }
    }
}