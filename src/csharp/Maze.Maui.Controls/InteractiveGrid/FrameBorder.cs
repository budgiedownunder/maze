using Maze.Maui.Controls.Pointer;
using Microsoft.Maui.Controls;
using System.Diagnostics;

namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `FrameBorder` class represents a frame border that forms part of a selection frame, optionally
    /// including a circular grip at one end that can be used to resize it
    /// </summary>
    public class FrameBorder
    {
        // Private properties
        SelectionFrame parentFrame;

        private readonly BorderBox box;
        private readonly BorderGrip? grip;

        static private Color DEFAULT_COLOR = Colors.Black;
        private const double DEFAULT_EDGE_THICKNESS = 2.0;
        private const double DEFAULT_GRIP_DIAMETER = 10.0;

        Color color = DEFAULT_COLOR;
        FrameEdge frameEdge = FrameEdge.None;
        FrameCorner gripCorner = FrameCorner.None;
        double edgeThickness = DEFAULT_EDGE_THICKNESS;
        double gripDiameter = DEFAULT_GRIP_DIAMETER;

        int startRow = 0;
        int startColumn = 0;

        double panStartX = 0.0;
        double panStartY = 0.0;
        bool havePanSelection = false;
        CellRange panStartSelection = new CellRange(-1, -1);

        /// <summary>
        /// Represents the edge with which a frame border is associated
        /// </summary>
        public enum FrameEdge
        {
            /// <summary>
            /// None
            /// </summary>
            None = 0,
            /// <summary>
            /// Top edge
            /// </summary>
            Top = 1,
            /// <summary>
            /// Left edge
            /// </summary>
            Left = 2,
            /// <summary>
            /// Bottom edge
            /// </summary>
            Bottom = 3,
            /// <summary>
            /// Right edge
            /// </summary>
            Right = 4
        }
        /// <summary>
        /// Represents the corner with which a frame border is associated
        /// </summary>
        public enum FrameCorner
        {
            /// <summary>
            /// None
            /// </summary>
            None = 0,
            /// <summary>
            /// Top-left
            /// </summary>
            TopLeft = 1,
            /// <summary>
            /// Top-right
            /// </summary>
            TopRight = 2,
            /// <summary>
            /// Bottom-left
            /// </summary>
            BottomLeft = 3,
            /// <summary>
            /// Bottom-right
            /// </summary>
            BottomRight = 4
        }
        /// <summary>
        /// Returns the objects's parent selection frame
        /// </summary>
        /// <returns>Selection frame</returns>
        public SelectionFrame ParentFrame { get => parentFrame; }
        /// <summary>
        /// Returns the object's parent grid
        /// </summary>
        /// <returns>Parent grid</returns>
        public Grid ParentGrid { get => ParentFrame.ParentGrid; }
        /// <summary>
        /// The frame edge associated with the object
        /// </summary>
        /// <returns>Frame edge</returns>
        public FrameEdge Edge
        {
            get => frameEdge;
            set
            {
                frameEdge = value;
                gripCorner = GetGripCorner(frameEdge);
            }
        }
        /// <summary>
        /// The frame corner associated with the object
        /// </summary>
        /// <returns>Frame corner</returns>
        public FrameCorner Corner { get => gripCorner; }
        /// <summary>
        /// Returns whether the object's edge is vertical
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsVerticalEdge { get => Edge == FrameEdge.Left || Edge == FrameEdge.Right; }
        /// <summary>
        /// Returns whether the object's edge is horizontal
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsHorizontalEdge { get => Edge == FrameEdge.Top || Edge == FrameEdge.Bottom; }
        /// <summary>
        /// Returns whether the object is associated with a top corner
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsTopCorner { get => Corner == FrameCorner.TopLeft || Corner == FrameCorner.TopRight; }
        /// <summary>
        /// Returns whether the object is associated with a bottom corner
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsBottomCorner { get => Corner == FrameCorner.BottomLeft || Corner == FrameCorner.BottomRight; }
        /// <summary>
        /// Returns whether the object is associated with a left corner
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsLeftCorner { get => Corner == FrameCorner.TopLeft || Corner == FrameCorner.BottomLeft; }
        /// <summary>
        /// Returns whether the object is associated with a right corner
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsRightCorner { get => Corner == FrameCorner.TopRight || Corner == FrameCorner.BottomRight; }
        /// <summary>
        /// Returns the start row associated with the object
        /// </summary>
        /// <returns>Start row</returns>
        public int StartRow { get => startRow; }
        /// <summary>
        /// Returns the start column associated with the object
        /// </summary>
        /// <returns>Start column</returns>
        public int StartColumn { get => startColumn; }
        /// <summary>
        /// The color associated with the object
        /// </summary>
        /// <returns>Color</returns>
        public Color Color
        {
            get => color;
            set
            {
                UpdateColor(value);
            }
        }
        /// <summary>
        /// The thickness of the frame edge (in DIPs)
        /// </summary>
        /// <returns>Edge thickness</returns>
        public double EdgeThickness
        {
            get => edgeThickness;
            set
            {
                UpdateEdgeThickness(value);
            }
        }
        /// <summary>
        /// The diameter of the frame's grip (in DIPs)
        /// </summary>
        /// <returns>Grip diameter</returns>
        public double GripDiameter
        {
            get => gripDiameter;
            set
            {
                UpdateGripDiameter(value);
            }
        }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="parentFrame">Parent selection frame</param>
        /// <param name="edge">The edge within the selection frame</param>
        /// <param name="color">Color</param>
        /// <param name="gripDiameter">Grip diameter (in DIPs)</param>
        public FrameBorder(SelectionFrame parentFrame, FrameEdge edge, Color color, double gripDiameter)
        {
            this.parentFrame = parentFrame;
            Edge = edge;
            Color = color;
            GripDiameter = gripDiameter;

            box = NewBox();
            if (ParentFrame.IsPanSupportEnabled)
                grip = NewGrip();

            RegisterEventHandlers();
        }
        /// <summary>
        /// Creates a new border box
        /// </summary>
        /// <returns>Border box</returns>
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
        /// <summary>
        /// Creates a new border grip
        /// </summary>
        /// <returns>Border grip</returns>
        private BorderGrip? NewGrip()
        {
            if (!ParentFrame.IsPanSupportEnabled) return null;

            BorderGrip grip = new BorderGrip(GetPointerIcon(true), GripDiameter)
            {
                Color = Color,
                IsVisible = false,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = false
            };
            return grip;
        }
        /// <summary>
        /// Registers event handlers
        /// </summary>
        private void RegisterEventHandlers()
        {
            RegisterBoxEventHandlers();
            RegisterGripEventHandlers();
        }
        /// <summary>
        /// Registers the event handlers for the frame's box
        /// </summary>
        private void RegisterBoxEventHandlers()
        {
            if (box is null) return;
            if (ParentFrame.IsPanSupportEnabled)
                box.PanUpdated += OnBoxPanUpdated;
        }
        /// <summary>
        /// Handles the pan updated event for the object's box, triggering any registered PanUpdated handler
        /// </summary>
        /// <param name="sender">Border box</param>
        /// <param name="e">Pan updated event arguments</param>
        /// 
        private void OnBoxPanUpdated(BorderBox sender, PanUpdatedEventArgs e)
        {
            OnPanUpdated(sender, e, false);
        }
        /// <summary>
        /// Registers the event handlers for the frame's grip (if enabled)
        /// </summary>
        private void RegisterGripEventHandlers()
        {
            if (grip is null) return;
            if (ParentFrame.IsPanSupportEnabled)
                grip.PanUpdated += OnGripPanUpdated;

        }
        /// <summary>
        /// Handles the pan updated event for the object's grip, triggering any registered PanUpdated handler
        /// </summary>
        /// <param name="sender">Border grip</param>
        /// <param name="e">Pan updated event arguments</param>
        /// 
        private void OnGripPanUpdated(BorderGrip sender, PanUpdatedEventArgs e)
        {
            OnPanUpdated(sender, e, true);
        }
        /// <summary>
        /// Gets the icon to display for the box or grip when the pointer hovers over the object
        /// </summary>
        /// <param name="isGrip">Return for the grip?</param>
        /// <returns>Pointer icon to display</returns>
        private Icon GetPointerIcon(bool isGrip)
        {
            return isGrip ? GetGripPointerIcon() : GetEdgePointerIcon();
        }
        /// <summary>
        /// Gets the icon to display when the pointer hovers over the box edge
        /// </summary>
        /// <returns>Pointer icon to display</returns>
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
        /// <summary>
        /// Gets the icon to display when the pointer hovers over the grip
        /// </summary>
        /// <returns>Pointer icon to display</returns>
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
        /// <summary>
        /// Handles the pan updated event for either the border box or grip, triggering any registered PanUpdated handler
        /// </summary>
        /// <param name="view">The object receiving the pan event</param>
        /// <param name="e">Pan updated event arguments</param>
        /// <param name="isGrip">Flag indicating whether `view` corresponds to a grip. If not, it is a box.</param>
        private void OnPanUpdated(IView view, PanUpdatedEventArgs e, bool isGrip)
        {
            switch (e.StatusType)
            {
                case GestureStatus.Started:
                    if (ParentFrame is not null && ParentFrame.CellRange is not null)
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

                    if (needSelectionChange && newCellRamge is not null)
                        ParentGrid.ResetSelection(newCellRamge);
                    break;

                case GestureStatus.Completed:
                    break;
            }
        }
        /// <summary>
        /// Recaculates the selecteed cell range for a pan change in X and/or Y`
        /// </summary>
        /// <param name="isGrip">Flag indicating whether the caclulation is for the grip. If not, it is for the box.</param>
        /// <param name="deltaX">Change in X (in DIPs)</param>
        /// <param name="deltaY">Change in Y (in DIPs)</param>
        private CellRange CalcCellSelectionForPanChange(bool isGrip, double deltaX, double deltaY)
        {
            return new CellRange(
                CalcTopCellForPanChange(isGrip, deltaY),
                CalcLeftCellForPanChange(isGrip, deltaX),
                CalcBottomCellForPanChange(isGrip, deltaY),
                CalcRightCellForPanChange(isGrip, deltaX));
        }
        /// <summary>
        /// Recaculates the new top cell row for a pan change in Y
        /// </summary>
        /// <param name="isGrip">Flag indicating whether the caclulation is for the grip. If not, it is for the box.</param>
        /// <param name="deltaY">Change in Y (in DIPs)</param>
        /// <returns>New top cell row</returns>
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
        /// <summary>
        /// Recaculates the new bottom cell row for a pan change in Y
        /// </summary>
        /// <param name="isGrip">Flag indicating whether the caclulation is for the grip. If not, it is for the box.</param>
        /// <param name="deltaY">Change in Y (in DIPs)</param>
        /// <returns>New bottom cell row</returns>
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
        /// <summary>
        /// Recaculates the new left cell column for a pan change in X
        /// </summary>
        /// <param name="isGrip">Flag indicating whether the caclulation is for the grip. If not, it is for the box.</param>
        /// <param name="deltaX">Change in X (in DIPs)</param>
        /// <returns>New left cell column</returns>
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
        /// <summary>
        /// Recaculates the new left right column for a pan change in X
        /// </summary>
        /// <param name="isGrip">Flag indicating whether the caclulation is for the grip. If not, it is for the box.</param>
        /// <param name="deltaX">Change in X (in DIPs)</param>
        /// <returns>New right cell column</returns>
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
        /// <summary>
        /// Updates the color to a new color
        /// </summary>
        /// <param name="newColor">New color</param>
        private void UpdateColor(Color newColor)
        {
            color = newColor;
            if (box is not null)
                box.Color = newColor;
            if (grip is not null)
                grip.Color = newColor;
        }
        /// <summary>
        /// Updates the edge thickness to a new thickmess
        /// </summary>
        /// <param name="newEdgeThickness">New thickness (in DIPs)</param>
        private void UpdateEdgeThickness(double newEdgeThickness)
        {
            edgeThickness = newEdgeThickness;
            box.DashThickness = (float)newEdgeThickness;
        }
        /// <summary>
        /// Updates the grip diameter to a new diameter
        /// </summary>
        /// <param name="newGripDiameter">New diameter (in DIPs)</param>
        private void UpdateGripDiameter(double newGripDiameter)
        {
            if (grip is null) return;
            grip.Diameter = newGripDiameter;
        }
        /// <summary>
        /// Shows or hides the object
        /// </summary>
        /// <param name="show">Flaf indicating whether to show the object</param>
        public void Show(bool show)
        {
            box.IsVisible = show;
            if (grip is not null)
                grip.IsVisible = show;
        }
        /// <summary>
        /// Adds the object's components to the parent grid
        /// </summary>
        public void AddToGrid()
        {
            AddViewToGrid(box);
            if (grip is not null)
                AddViewToGrid(grip);
        }
        /// <summary>
        /// Adds a view object to the data grid
        /// </summary>
        private void AddViewToGrid(IView view)
        {
            Microsoft.Maui.Controls.Grid.SetRow((BindableObject)view, 0);
            Microsoft.Maui.Controls.Grid.SetColumn((BindableObject)view, 0);
            ParentGrid.AddToDataGrid(view);
        }
        /// <summary>
        /// Removes the object's components from the parent grid
        /// </summary>
        public void RemoveFromGrid()
        {
            RemoveViewFromGrid(box);
            if (grip is not null)
                RemoveViewFromGrid(grip);
        }
        /// <summary>
        /// Removes a view object from the data grid
        /// </summary>
        private void RemoveViewFromGrid(IView view)
        {
            ParentGrid.RemoveFromDataGrid(view);
        }
        /// <summary>
        /// Sets the display position and margin for the object
        /// </summary>
        /// <param name="row">Display row</param>
        /// <param name="column">Display column</param>
        /// <param name="width">Display width (in DIPs)</param>
        /// <param name="height">Display height (in DIPs)</param>
        /// <param name="margin">Margin (in DIPs)</param>
        public void SetPosition(int row, int column, double width, double height, Thickness margin)
        {
            startRow = row;
            startColumn = column;

            Microsoft.Maui.Controls.Grid.SetRow(box, StartRow);
            Microsoft.Maui.Controls.Grid.SetColumn(box, StartColumn);
            box.WidthRequest = width;
            box.HeightRequest = height;
            box.Margin = margin;

            if (grip is not null)
            {
                Microsoft.Maui.Controls.Grid.SetRow(grip, row);
                Microsoft.Maui.Controls.Grid.SetColumn(grip, column);
                grip.TranslationX = GetGripTranslationX(width);
                grip.TranslationY = GetGripTranslationY(height);
                grip.Margin = margin;
            }
        }
        /// <summary>
        /// Gets the grip corner associated with a frame edge
        /// </summary>
        /// <param name="edge">Frame edge</param>
        /// <returns>Frame corner</returns>
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
        /// <summary>
        /// Gets the grip translation in the X direction for a given width
        /// </summary>
        /// <param name="width">Object width (in DIPs)</param>
        /// <returns>X translation to apply to object's X position to draw grip (in DIPs)</returns>
        private double GetGripTranslationX(double width)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Edge)
            {
                case FrameEdge.Top:
                    return -GripDiameter / 2.0;
                case FrameEdge.Left:
                    return -GripDiameter / 2.0;
                case FrameEdge.Bottom:
                    return width - GripDiameter / 2.0; ;
                case FrameEdge.Right:
                    return 1.0 - GripDiameter / 2.0;
            }
            return 0.0;
        }
        /// <summary>
        /// Gets the grip translation in the Y direction for a given height
        /// </summary>
        /// <param name="height">Object height (in DIPs)</param>
        /// <returns>Ytranslation to apply to object's Y position to draw grip (in DIPs)</returns>
        private double GetGripTranslationY(double height)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (Edge)
            {
                case FrameEdge.Top:
                    return -GripDiameter / 2.0;
                case FrameEdge.Left:
                    return height - GripDiameter / 2.0;
                case FrameEdge.Bottom:
                    return height - GripDiameter / 2.0;
                case FrameEdge.Right:
                    return -GripDiameter / 2.0;
            }
            return 0.0;
        }
        /// <summary>
        /// Enables or disables dash animation for the object
        /// </summary>
        /// <param name="enable">Enable animation?</param>
        public void EnableDashAnimation(bool enable)
        {
            box.EnableDashAnimation(enable);
        }
        /// <summary>
        /// Updates the dash animation for the object to the next animation
        /// display index
        /// </summary>
        public void UpdateDashAnimation()
        {
            box.UpdateDashAnimation();
        }

    }
}