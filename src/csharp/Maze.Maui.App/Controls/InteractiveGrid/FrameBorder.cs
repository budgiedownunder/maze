using Maze.Maui.App.Controls.Pointer;
using Microsoft.Maui.Controls;
using System.Diagnostics;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class FrameBorder
    {
        SelectionFrame parentFrame;

        private readonly BorderBox box;
        private readonly BorderGrip? grip;

        static private Color DEFAULT_COLOR = Colors.Black;
        private const double DEFAULT_EDGE_THICKNESS = 2.0;
        private const double DEFAULT_GRIP_SIZE = 10.0;

        Color color = DEFAULT_COLOR;
        FrameEdge frameEdge = FrameEdge.None;
        FrameCorner gripCorner = FrameCorner.None;
        double edgeThickness = DEFAULT_EDGE_THICKNESS;
        double gripSize = DEFAULT_GRIP_SIZE;

        int startRow = 0;
        int startColumn = 0;

        double panStartX = 0.0;
        double panStartY = 0.0;
        bool havePanSelection = false;
        CellRange panStartSelection = new CellRange(-1, -1);

        public enum FrameEdge
        {
            None = 0,
            Top = 1,
            Left = 2,
            Bottom = 3,
            Right = 4
        }

        public enum FrameCorner
        {
            None = 0,
            TopLeft = 1,
            TopRight = 2,
            BottomLeft = 3,
            BottomRight = 4
        }

        public SelectionFrame ParentFrame { get => parentFrame; }

        public Grid ParentGrid { get => ParentFrame.ParentGrid; }

        public FrameEdge Edge
        {
            get => frameEdge;
            set
            {
                frameEdge = value;
                gripCorner = GetGripCorner(frameEdge);
            }
        }

        public FrameCorner Corner { get => gripCorner; }

        public bool IsVerticalEdge { get => Edge == FrameEdge.Left || Edge == FrameEdge.Right; }

        public bool IsHorizontalEdge { get => Edge == FrameEdge.Top || Edge == FrameEdge.Bottom; }

        public bool IsTopCorner { get => Corner == FrameCorner.TopLeft || Corner == FrameCorner.TopRight; }

        public bool IsBottomCorner { get => Corner == FrameCorner.BottomLeft || Corner == FrameCorner.BottomRight; }

        public bool IsLeftCorner { get => Corner == FrameCorner.TopLeft || Corner == FrameCorner.BottomLeft; }

        public bool IsRightCorner { get => Corner == FrameCorner.TopRight || Corner == FrameCorner.BottomRight; }

        public int StartRow { get => startRow; }

        public int StartColumn { get => startColumn; }

        public Color Color
        {
            get => color;
            set
            {
                UpdateColor(value);
            }
        }

        public double EdgeThickness
        {
            get => edgeThickness;
            set
            {
                UpdateEdgeThickness(value);
            }
        }

        public double GripSize
        {
            get => gripSize;
            set
            {
                UpdateGripSize(value);
            }
        }

        public FrameBorder(SelectionFrame parentFrame, FrameEdge edge, Color color, double gripSize)
        {
            this.parentFrame = parentFrame;
            Edge = edge;
            Color = color;
            GripSize = gripSize;

            box = NewBox();
            if (ParentFrame.IsPanSupportEnabled)
                grip = NewGrip();

            RegisterEventHandlers();
        }

        private BorderBox NewBox()
        {
            BorderBox borderBox = new BorderBox(GetPointerIcon(false), (float)EdgeThickness)
            {
                BackgroundColor = Color,
                IsVisible = false,
                HeightRequest = 0,
                WidthRequest = 0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = false
            };
            return borderBox;
        }

        private BorderGrip? NewGrip()
        {
            if (!ParentFrame.IsPanSupportEnabled) return null;

            BorderGrip grip = new BorderGrip(GetPointerIcon(true), GripSize)
            {
                Color = Color,
                IsVisible = false,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = false
            };
            return grip;
        }

        void RegisterEventHandlers()
        {
            RegisterBoxEventHandlers();
            RegisterGripEventHandlers();
        }

        private void RegisterBoxEventHandlers()
        {
            if (box == null) return;
            if (ParentFrame.IsPanSupportEnabled)
                box.PanUpdated += OnBoxPanUpdated;
        }

        private void OnBoxPanUpdated(BorderBox sender, PanUpdatedEventArgs e)
        {
            OnPanUpdated(sender, e, false);
        }

        private void RegisterGripEventHandlers()
        {
            if (grip == null) return;
            if (ParentFrame.IsPanSupportEnabled)
                grip.PanUpdated += OnGripPanUpdated;

        }

        private void OnGripPanUpdated(BorderGrip sender, PanUpdatedEventArgs e)
        {
            OnPanUpdated(sender, e, true);
        }

        private Icon GetPointerIcon(bool isGrip)
        {
            return isGrip ? GetGripPointerIcon() : GetEdgePointerIcon();
        }

        private Icon GetEdgePointerIcon()
        {
            switch (Edge)
            {
                case FrameEdge.Top:
                    return Icon.SizeNorthSouth;
                case FrameEdge.Left:
                    return Icon.SizeWestEast;
                case FrameEdge.Bottom:
                    return Icon.SizeNorthSouth;
                case FrameEdge.Right:
                    return Icon.SizeWestEast;
            }
            return Icon.Arrow;
        }

        private Icon GetGripPointerIcon()
        {
            switch (Corner)
            {
                case FrameCorner.TopLeft:
                case FrameCorner.BottomRight:
                    return Icon.SizeNorthWestSouthEast;
                case FrameCorner.TopRight:
                case FrameCorner.BottomLeft:
                    return Icon.SizeNorthEastSouthWest;
            }
            return Icon.Arrow;
        }

        private void OnPanUpdated(IView view, PanUpdatedEventArgs e, bool isGrip)
        {
            switch (e.StatusType)
            {
                case GestureStatus.Started:
                    if (ParentFrame != null && ParentFrame.CellRange != null)
                    {
                        havePanSelection = true;
                        panStartSelection = ParentFrame.CellRange.Clone();
                        panStartX = e.TotalX;
                        panStartY = e.TotalY;
                    }
                    break;

                case GestureStatus.Running:
                    if (!havePanSelection) return;
                    var deltaX = isGrip || IsVerticalEdge ? e.TotalX - panStartX : 0.0;
                    var deltaY = isGrip || IsHorizontalEdge ? e.TotalY - panStartY : 0.0;
                    CellRange newCellRamge = CalcCellSelectionForPanChange(isGrip, deltaX, deltaY);
                    bool needSelectionChange = !newCellRamge.Equals(panStartSelection);

                    if (needSelectionChange && newCellRamge != null)
                        ParentGrid.ResetSelection(newCellRamge);
                    break;

                case GestureStatus.Completed:
                    break;
            }
        }

        private CellRange CalcCellSelectionForPanChange(bool isGrip, double deltaX, double deltaY)
        {
            return new CellRange(
                CalcTopCellForPanChange(isGrip, deltaY),
                CalcLeftCellForPanChange(isGrip, deltaX),
                CalcBottomCellForPanChange(isGrip, deltaY),
                CalcRightCellForPanChange(isGrip, deltaX));
        }

        private int CalcTopCellForPanChange(bool isGrip, double deltaY)
        {
            if (deltaY == 0 || (isGrip && !IsTopCorner) || (!isGrip && Edge != FrameEdge.Top))
            {
                return panStartSelection.Top;
            }
            return ParentGrid.FindCellRowAtYOffset(
                    panStartSelection.Top,
                    Grid.YOffsetType.TopEdge,
                    deltaY);
        }

        private int CalcBottomCellForPanChange(bool isGrip, double deltaY)
        {
            if (deltaY == 0 || (isGrip && !IsBottomCorner) || (!isGrip && Edge != FrameEdge.Bottom))
            {
                return panStartSelection.Bottom;
            }
            return ParentGrid.FindCellRowAtYOffset(
                    panStartSelection.Bottom,
                    Grid.YOffsetType.BottomEdge,
                    deltaY);
        }

        private int CalcLeftCellForPanChange(bool isGrip, double deltaX)
        {
            if (deltaX == 0 || (isGrip && !IsLeftCorner) || (!isGrip && Edge != FrameEdge.Left))
            {
                return panStartSelection.Left;
            }
            return ParentGrid.FindCellColumnAtXOffset(
                    panStartSelection.Left,
                    Grid.XOffsetType.LeftEdge,
                    deltaX);
        }

        private int CalcRightCellForPanChange(bool isGrip, double deltaX)
        {
            if (deltaX == 0 || (isGrip && !IsRightCorner) || (!isGrip && Edge != FrameEdge.Right))
            {
                return panStartSelection.Right;
            }
            return ParentGrid.FindCellColumnAtXOffset(
                    panStartSelection.Right,
                    Grid.XOffsetType.RightEdge,
                    deltaX);
        }

        private void UpdateColor(Color newColor)
        {
            color = newColor;
            if (box != null)
                box.Color = newColor;
            if (grip != null)
                grip.Color = newColor;
        }

        private void UpdateEdgeThickness(double newEdgeThickness)
        {
            edgeThickness = newEdgeThickness;
            box.DashThickness = (float)newEdgeThickness;
        }

        private void UpdateGripSize(double newGripSize)
        {
            if (grip == null) return;
            grip.Size = newGripSize;
        }

        public void Show(bool show)
        {
            box.IsVisible = show;
            if (grip != null)
                grip.IsVisible = show;
        }

        public void AddToGrid()
        {
            AddViewToGrid(box);
            if (grip != null)
                AddViewToGrid(grip);
        }

        private void AddViewToGrid(IView view)
        {
            ParentGrid.SetRow(view, 0);
            ParentGrid.SetColumn(view, 0);
            ParentGrid.Children.Add(view);
        }

        public void RemoveFromGrid()
        {
            RemoveViewFromGrid(box);
            if (grip != null)
                RemoveViewFromGrid(grip);
        }

        private void RemoveViewFromGrid(IView view)
        {
            ParentGrid.Children.Remove(view);
        }

        public void SetPosition(int row, int column, double width, double height, Thickness margin)
        {
            startRow = row;
            startColumn = column;

            ParentGrid.SetRow(box, StartRow);
            ParentGrid.SetColumn(box, StartColumn);
            box.WidthRequest = width;
            box.HeightRequest = height;
            box.Margin = margin;

            if (grip != null)
            {
                ParentGrid.SetRow(grip, row);
                ParentGrid.SetColumn(grip, column);
                grip.TranslationX = GetGripTranslationX(width);
                grip.TranslationY = GetGripTranslationY(height);
                grip.Margin = margin;
            }
        }

        private FrameCorner GetGripCorner(FrameEdge edge)
        {
            switch (edge)
            {
                case FrameEdge.Top:
                    return FrameCorner.TopLeft;
                case FrameEdge.Left:
                    return FrameCorner.BottomLeft;
                case FrameEdge.Bottom:
                    return FrameCorner.BottomRight;
                case FrameEdge.Right:
                    return FrameCorner.TopRight;
            }
            return FrameCorner.None;
        }

        private double GetGripTranslationX(double width)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Edge)
            {
                case FrameEdge.Top:
                    return -GripSize / 2.0;
                case FrameEdge.Left:
                    return -GripSize / 2.0;
                case FrameEdge.Bottom:
                    return width - GripSize / 2.0; ;
                case FrameEdge.Right:
                    return 1.0 - GripSize / 2.0;
            }
            return 0.0;
        }

        private double GetGripTranslationY(double height)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Edge)
            {
                case FrameEdge.Top:
                    return -GripSize / 2.0;
                case FrameEdge.Left:
                    return height - GripSize / 2.0;
                case FrameEdge.Bottom:
                    return height - GripSize / 2.0;
                case FrameEdge.Right:
                    return -GripSize / 2.0;
            }
            return 0.0;
        }

        public void EnableDashAnimation(bool enable)
        {
            box.EnableDashAnimation(enable);
        }

        public void UpdateDashAnimation()
        {
            box.UpdateDashAnimation();
        }

    }
}