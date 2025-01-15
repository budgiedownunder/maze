namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `SelectionFrame` class represents a selection frame that is used to
    /// border a rectangular or square region of cells. It consists of four 
    /// borders (with optional circular grips) that respond to pointer/pan events
    /// for resizing the selected aread. In addition, the frame can run in
    /// an animated dash mode e.g. for use on touch-only devices.
    /// </summary>
    public class SelectionFrame
    {
        // Private properties
        private const int BORDER_ID_TOP = 0;
        private const int BORDER_ID_LEFT = 1;
        private const int BORDER_ID_BOTTOM = 2;
        private const int BORDER_ID_RIGHT = 3;

        private const int BORDER_ID_FIRST = BORDER_ID_TOP;
        private const int BORDER_ID_LAST = BORDER_ID_RIGHT;

        static private Color DEFAULT_BORDER_COLOR = Colors.Black;
        const double DEFAULT_BORDER_WIDTH = 2.0;
        const double DEFAULT_BORDER_GRIP_DIAMETER = 10.0;
        const bool DEFAULT_IS_PAN_SUPPORT_ENABLED = true;
        const int ANIMATION_UPDATE_INTERVAL_MS = 100;

        private Color borderColor = DEFAULT_BORDER_COLOR;
        private double borderWidth = DEFAULT_BORDER_WIDTH;
        private double borderGripDiameter = DEFAULT_BORDER_GRIP_DIAMETER;

        private readonly Grid parentGrid;
        private readonly FrameBorder[] frameBorders = new FrameBorder[4];
        private bool isPanSupportEnabled;
        private bool isAnimationRunning = false;
        private CellRange? cellRange;

        /// <summary>
        /// Border color
        /// </summary>
        /// <returns>Border color</returns>
        public Color BorderColor
        {
            get => borderColor;
            set
            {
                borderColor = value;
                UpdateBorders();
            }
        }
        /// <summary>
        /// Border width (in DIPs)
        /// </summary>
        /// <returns>Border width</returns>
        public double BorderWidth
        {
            get => borderWidth;
            set
            {
                borderWidth = value;
                UpdateBorders();
            }
        }
        /// <summary>
        /// Border grip diameter (in DIPs)
        /// </summary>
        /// <returns>Border grip diameter</returns>
        public double BorderGripDiameter
        {
            get => borderGripDiameter;
            set
            {
                borderGripDiameter = value;
                UpdateBorders();
            }
        }
        /// <summary>
        /// Cell range
        /// </summary>
        /// <returns>Cell range</returns>
        public CellRange? CellRange
        {
            get { return cellRange; }
            set
            {
                cellRange = value;
            }
        }
        /// <summary>
        /// Parent grid
        /// </summary>
        /// <returns>Parent grid</returns>
        public Grid ParentGrid { get => parentGrid; }
        /// <summary>
        /// Top row of cell range
        /// </summary>
        /// <returns>Top row</returns>
        public int TopRow { get { return CellRange is not null ? CellRange.Top : -1; } }
        /// <summary>
        /// Bottom row of cell range
        /// </summary>
        /// <returns>Bottom row</returns>
        public int BottomRow { get { return CellRange is not null ? CellRange.Bottom : -1; } }
        /// <summary>
        /// Number of rows in the cell range
        /// </summary>
        /// <returns>Number of rows in cell range</returns>
        public int RowCount { get { return CellRange is not null ? CellRange.Height : 0; } }
        /// <summary>
        /// Left column of cell range
        /// </summary>
        /// <returns>Left column</returns>
        public int LeftColumn { get { return CellRange is not null ? CellRange.Left : -1; } }
        /// <summary>
        /// Right column of cell range
        /// </summary>
        /// <returns>Right column</returns>
        public int RightColumn { get { return CellRange is not null ? CellRange.Right : -1; } }
        /// <summary>
        /// Number of columns in the cell range
        /// </summary>
        /// <returns>Number of columns in cell range</returns>
        public int ColumnCount { get { return CellRange is not null ? CellRange.Width : 0; } }
        /// <summary>
        /// Indicates whether pan support is enabled
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsPanSupportEnabled { get => isPanSupportEnabled; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="parentGrid">Parent grid</param>
        /// <param name="enablePanSupport">Enable pan support?</param>
        public SelectionFrame(Grid parentGrid, bool enablePanSupport)
        {
            this.parentGrid = parentGrid;
            this.isPanSupportEnabled = enablePanSupport;
            CreateBorders();
        }
        /// <summary>
        /// Creates the borders
        /// </summary>
        private void CreateBorders()
        {
            for (int id = BORDER_ID_FIRST; id <= BORDER_ID_LAST; id++)
                frameBorders[id] = NewFrameBorder(ToFrameBorderEdge(id));
        }
        /// <summary>
        /// Converts an internal border ID to a `FrameEdge` type
        /// </summary>
        /// <param name="borderID">Border ID</param>
        /// <returns>Frame edge type</returns>
        private FrameBorder.FrameEdge ToFrameBorderEdge(int borderID)
        {
            switch (borderID)
            {
                case BORDER_ID_TOP:
                    return FrameBorder.FrameEdge.Top;
                case BORDER_ID_LEFT:
                    return FrameBorder.FrameEdge.Left;
                case BORDER_ID_BOTTOM:
                    return FrameBorder.FrameEdge.Bottom;
                case BORDER_ID_RIGHT:
                    return FrameBorder.FrameEdge.Right;
            }
            return FrameBorder.FrameEdge.Top;
        }
        /// <summary>
        /// Creates a new frame border
        /// </summary>
        /// <param name="edge">Frame edge type</param>
        /// <returns>Frame border</returns>
        private FrameBorder NewFrameBorder(FrameBorder.FrameEdge edge)
        {
            return new FrameBorder(this, edge, BorderColor, BorderGripDiameter);
        }
        /// <summary>
        /// Updates the borders
        /// </summary>
        private void UpdateBorders()
        {
            foreach (var border in frameBorders)
            {
                border.Color = BorderColor;
                border.EdgeThickness = BorderWidth;
                border.GripDiameter = BorderGripDiameter;
            }
        }
        /// <summary>
        /// Shows or hides the object
        /// </summary>
        /// <param name="show">Show?</param>
        public void Show(bool show)
        {
            foreach (var border in frameBorders)
                border.Show(show);
        }
        /// <summary>
        /// Adds the object's components to the grid
        /// </summary>
        public void AddToGrid()
        {
            foreach (var border in frameBorders)
                border.AddToGrid();
        }
        /// <summary>
        /// Removes the object's components from the grid
        /// </summary>
        public void RemoveFromGrid()
        {
            foreach (var border in frameBorders)
                border.RemoveFromGrid();
        }
        /// <summary>
        /// Modifies the range associated with the object
        /// </summary>
        /// <param name="newRange">New cell range</param>
        /// <param name="show">Show?</param>
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
        /// <summary>
        /// Sets the details associated with a given border
        /// </summary>
        /// <param name="borderID">Border ID</param>
        /// <param name="row">Display row</param>
        /// <param name="column">Display column</param>
        /// <param name="width">Width (in DIPs)</param>
        /// <param name="height">Height (in DIPs)</param>
        /// <param name="margin">Margin (in DIPs)</param>
        private void SetBorder(int borderID, int row, int column, double width, double height, Thickness margin)
        {
            FrameBorder border = frameBorders[borderID];
            border.SetPosition(row, column, width, height, margin);
        }
        /// <summary>
        /// Enables or disables dash animation
        /// </summary>
        /// <param name="enable">Enable?</param>
        public void EnableDashAnimation(bool enable)
        {
            foreach (var border in frameBorders)
                border.EnableDashAnimation(enable);

            if(enable)
                StartDashAnimation();
            else
                StopDashAnimation();
        }
        /// <summary>
        /// Updates the dash animation for the object to the next animation
        /// display index
        /// </summary>
        private void UpdateDashAnimation()
        {
            foreach (var border in frameBorders)
                border.UpdateDashAnimation();
        }
        /// <summary>
        /// Starts dash animation
        /// </summary>
        private void StartDashAnimation() 
        { 
            if (isAnimationRunning || Application.Current is null) return;
            isAnimationRunning = true;

            var dispatcher = Application.Current.Dispatcher;

            dispatcher.StartTimer(TimeSpan.FromMilliseconds(ANIMATION_UPDATE_INTERVAL_MS), () =>
            {
                if (!isAnimationRunning)
                    return false;

                UpdateDashAnimation();

                return true;
            });
        }
        /// <summary>
        /// Stops dash animation
        /// </summary>
        private void StopDashAnimation() {
            isAnimationRunning = false;
        }

    }
}
