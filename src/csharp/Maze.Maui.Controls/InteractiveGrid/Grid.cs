using Microsoft.Maui.Controls.Shapes;

namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `Grid` class represents an interactive grid that suppports cell display and selection/highlighting via mouse/keyboard/touch
    /// </summary>
    public partial class Grid : Microsoft.Maui.Controls.Grid
    {
        // Private properties
        private CellFrame? activeCell = null;
        private CellPoint activeCellPoint = new CellPoint();
        private CellFrame? anchorCell = null;
        private CellPoint anchorCellPoint = new CellPoint();
        private CellRange? selectedCells;
        private SelectionFrame? selectionFrame;

        const double DEFAULT_COL_HEADER_HEIGHT = 35.0;
        const double DEFAULT_COL_HEADER_MARGIN = 0.0;
        const double DEFAULT_COL_HEADER_PADDING = 0.0;

        const double DEFAULT_ROW_HEADER_WIDTH = 35.0;
        const double DEFAULT_ROW_HEADER_MARGIN = 0.0;
        const double DEFAULT_ROW_HEADER_PADDING = 0.0;

        const double DEFAULT_CELL_HEIGHT = 50.0;
        const double DEFAULT_CELL_WIDTH = 50.0;
        const double DEFAULT_CELL_MARGIN = 0.0;
        const double DEFAULT_CELL_PADDING = 0.0;

        const double DEFAULT_SELECTION_FRAME_BORDER_WIDTH = 2.0;
        const double DEFAULT_SELECTION_FRAME_BORDER_GRIP_DIAMETER = 10.0;

        const bool DEFAULT_IS_PAN_SUPPORT_ENABLED = true;

        static private Color GRID_LIGHT_GREEN = Color.FromRgb(211, 240, 224);
        static private Color GRID_VERY_LIGHT_GRAY = Color.FromRgb(240, 240, 240);
        static private Color GRID_LIGHT_GRAY = Color.FromRgb(225, 225, 225);

        /// <summary>
        /// Represents a type of X offset within the grid
        /// </summary>
        public enum XOffsetType
        {
            /// <summary>
            /// Offset is from left edge
            /// </summary>
            LeftEdge = 0,
            /// <summary>
            /// Offset is from right edge
            /// </summary>
            RightEdge = 1
        }
        /// <summary>
        /// Represents a type of Y offset within the grid
        /// </summary>
        public enum YOffsetType
        {
            /// <summary>
            /// Offset is from top edge
            /// </summary>
            TopEdge = 0,
            /// <summary>
            /// Offset is from bottom edge
            /// </summary>
            BottomEdge = 1
        }
        /// <summary>
        /// Number of rows within the grid (excluding header rows)
        /// </summary>
        /// <returns>Number of rows</returns>
        public int RowCount { get; set; } = 0;
        /// <summary>
        /// Number of columns within the grid (excluding header columns)
        /// </summary>
        /// <returns>Number of columns</returns>
        public int ColumnCount { get; set; } = 0;
        /// <summary>
        /// Column header height (in DIPs)
        /// </summary>
        /// <returns>Column header height</returns>
        public double ColumnHeaderHeight { get; set; } = DEFAULT_COL_HEADER_HEIGHT;
        /// <summary>
        /// Column header margin (in DIPs)
        /// </summary>
        /// <returns>Column header margin</returns>
        public double ColumnHeaderMargin { get; set; } = DEFAULT_COL_HEADER_MARGIN;
        /// <summary>
        /// Column header padding (in DIPs)
        /// </summary>
        /// <returns>Column header padding</returns>
        public double ColumnHeaderPadding { get; set; } = DEFAULT_COL_HEADER_PADDING;
        /// <summary>
        /// Row header width (in DIPs)
        /// </summary>
        /// <returns>Row header width</returns>
        public double RowHeaderWidth { get; set; } = DEFAULT_ROW_HEADER_WIDTH;
        /// <summary>
        /// Row header margin (in DIPs)
        /// </summary>
        /// <returns>Row header margin</returns>
        public double RowHeaderMargin { get; set; } = DEFAULT_ROW_HEADER_MARGIN;
        /// <summary>
        /// Row header padding (in DIPs)
        /// </summary>
        /// <returns>Row header padding</returns>
        public double RowHeaderPadding { get; set; } = DEFAULT_ROW_HEADER_PADDING;
        /// <summary>
        /// Cell height (in DIPs)
        /// </summary>
        /// <returns>Cell height</returns>
        public double CellHeight { get; set; } = DEFAULT_CELL_HEIGHT;
        /// <summary>
        /// Cell width (in DIPs)
        /// </summary>
        /// <returns>Cell width</returns>
        public double CellWidth { get; set; } = DEFAULT_CELL_WIDTH;
        /// <summary>
        /// Cell margin (in DIPs)
        /// </summary>
        /// <returns>Cell margin</returns>
        public double CellMargin { get; set; } = DEFAULT_CELL_MARGIN;
        /// <summary>
        /// Cell padding (in DIPs)
        /// </summary>
        /// <returns>Cell padding</returns>
        public double CellPadding { get; set; } = DEFAULT_CELL_PADDING;
        /// <summary>
        /// Header border color
        /// </summary>
        /// <returns>Header border color</returns>
        public Color HeaderBorderColor { get; set; } = Colors.Gray;
        /// <summary>
        /// Header background color
        /// </summary>
        /// <returns>Header background color</returns>
        public Color HeaderBackgroundColor { get; set; } = GRID_VERY_LIGHT_GRAY;
        /// <summary>
        /// Header background color when selected
        /// </summary>
        /// <returns>Header background color when selected</returns>
        public Color HeaderSelectedBackgroundColor { get; set; } = GRID_LIGHT_GREEN;
        /// <summary>
        /// Header background color when active
        /// </summary>
        /// <returns>Header background color when active</returns>
        public Color HeaderActiveBackgroundColor { get; set; } = GRID_LIGHT_GRAY;
        /// <summary>
        /// Header text color
        /// </summary>
        /// <returns>Header text color</returns>
        public Color HeaderTextColor { get; set; } = Colors.Black;
        /// <summary>
        /// Cell border color
        /// </summary>
        /// <returns>Cell border color</returns>
        public Color CellBorderColor { get; set; } = Colors.Black;
        /// <summary>
        /// Cell background color
        /// </summary>
        /// <returns>Cell background color</returns>
        public Color CellBackgroundColor { get; set; } = Colors.White;
        /// <summary>
        /// Cell background color when highlighted
        /// </summary>
        /// <returns>Cell background color when highlighted</returns>
        public Color HighlightCellBackgroundColor { get; set; } = GRID_LIGHT_GREEN;
        /// <summary>
        /// Active cell background color
        /// </summary>
        /// <returns>Active cell background color</returns>
        public Color ActiveCellBackgroundColor { get; set; } = Colors.Yellow;
        /// <summary>
        /// Anchor cell background color
        /// </summary>
        /// <returns>Anchor cell background color</returns>
        public Color AnchorCellBackgroundColor { get; set; } = Colors.Yellow;
        /// <summary>
        /// Selection frame border color
        /// </summary>
        /// <returns>Selection frame border color</returns>
        public Color SelectionFrameBorderColor { get; set; } = Colors.DarkGreen;
        /// <summary>
        /// Selection frame border width
        /// </summary>
        /// <returns>Selection frame border width</returns>
        public double SelectionFrameBorderWidth { get; set; } = DEFAULT_SELECTION_FRAME_BORDER_WIDTH;
        /// <summary>
        /// Selection frame border grip diameter (in DIPs)
        /// </summary>
        /// <returns>Selection frame border grip diameter</returns>
        public double SelectionFrameBorderGripDiameter { get; set; } = DEFAULT_SELECTION_FRAME_BORDER_GRIP_DIAMETER;
        /// <summary>
        /// Indicates whether the grid is currently in extended selection mode
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsExtendedSelectionMode { get; set; } = false;
        /// <summary>
        /// Indicates whether the grid has pan support enabled
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsPanSupportEnabled { get; set; } = DEFAULT_IS_PAN_SUPPORT_ENABLED;

        //private CommunityToolkit.Maui.Behaviors.TouchBehavior longPressBehaviour;

        /// <summary>
        /// Indicates whether the grid currently has all columns selected
        /// </summary>
        /// <returns>Boolean</returns>
        public bool AllColumnsSelected
        {
            get
            {
                return (selectedCells is not null && selectedCells.Left == 1 && selectedCells.Right == ColumnCount) ||
                    (ColumnCount == 1 && activeCellPoint.Column == 1);
            }
        }
        /// <summary>
        /// Indicates whether the grid currently has all rows selected
        /// </summary>
        /// <returns>Boolean</returns>
        public bool AllRowsSelected
        {
            get
            {
                return (selectedCells is not null && selectedCells.Top == 1 && selectedCells.Bottom == RowCount) ||
                    (RowCount == 1 && activeCellPoint.Row == 1);
            }
        }
        /// <summary>
        /// The current active cell frame (if any)
        /// </summary>
        /// <returns>Active cell frame</returns>
        public CellFrame? ActiveCell { get => activeCell; }
        /// <summary>
        /// The current selected cell range (if any)
        /// </summary>
        /// <returns>Selected cell range</returns>
        public CellRange? CurrentSelection { get => selectedCells is not null ? selectedCells.Clone() : new CellRange(activeCellPoint); }
        /// <summary>
        /// Constructor
        /// </summary>
        public Grid()
        {
            InitializePlatformSpecificCode();
            AddPinchGesture();
        }
        /// <summary>
        /// The container scroll view linked to the object
        /// </summary>
        /// <returns>Container scroll view</returns>
        public static readonly BindableProperty ContainerScrollViewProperty =
            BindableProperty.Create(nameof(ContainerScrollView), typeof(ScrollView), typeof(Grid));

        /// <summary>
        /// The container scroll view linked to the object
        /// </summary>
        /// <returns>Container scroll view</returns>
        public ScrollView ContainerScrollView
        {
            get => (ScrollView)GetValue(ContainerScrollViewProperty);
            set => SetValue(ContainerScrollViewProperty, value);
        }
        /// <summary>
        /// The container content view linked to the object
        /// </summary>
        /// <returns>Container content view</returns>
        public static readonly BindableProperty ContainerContentViewProperty =
            BindableProperty.Create(nameof(ContainerContentView), typeof(ContentView), typeof(Grid));
        /// <summary>
        /// The container content view linked to the object
        /// </summary>
        /// <returns>Container content view</returns>
        public ContentView ContainerContentView
        {
            get => (ContentView)GetValue(ContainerContentViewProperty);
            set => SetValue(ContainerContentViewProperty, value);
        }
        /// <summary>
        /// Placeholder for platform-specific code
        /// </summary>
        partial void InitializePlatformSpecificCode();
        /// <summary>
        /// Adds support for the pinch gesture
        /// </summary>
        private void AddPinchGesture()
        {
            var pinchGesture = new PinchGestureRecognizer();
            pinchGesture.PinchUpdated += OnPinchUpdated;
            GestureRecognizers.Add(pinchGesture);
        }
        // Pinch status
        private double currentScale = 1;
        private double startScale = 1;
        private bool isPinching = false;
        /// <summary>
        /// Handles the pinch updated event
        /// </summary>
        /// <param name="sender">Object sender</param>
        /// <param name="e">Pinch gesture updated event arguments</param>
        private void OnPinchUpdated(object? sender, PinchGestureUpdatedEventArgs e)
        {
            if (sender is null) return;

            if (e.Status == GestureStatus.Started)
            {
                startScale = currentScale;
                isPinching = true;
                ContainerScrollView.IsEnabled = false;
            }
            else if (e.Status == GestureStatus.Running && isPinching)
            {
                if (Math.Abs(e.Scale - 1) > 0.01)
                {
                    currentScale = Math.Clamp(startScale * e.Scale, 0.5, 3.0);
                    ContainerContentView.Scale = currentScale;
                }
            }
            else if (e.Status == GestureStatus.Completed)
            {
                isPinching = false;
                ContainerScrollView.IsEnabled = true;
                startScale = currentScale;
            }
        }
        /// <summary>
        /// Creates a new row definition
        /// </summary>
        /// <param name="columnHeader">Create a column header row?</param>
        /// <returns>Row definition</returns>
        private RowDefinition NewRowDefinition(bool columnHeader)
        {
            return new RowDefinition { Height = new GridLength(columnHeader ? this.ColumnHeaderHeight : this.CellHeight) };
        }
        /// <summary>
        /// Creates a new column definition
        /// </summary>
        /// <param name="rowHeader">Create a row header column?</param>
        /// <returns>Column definition</returns>
        private ColumnDefinition NewColumnDefinition(bool rowHeader)
        {
            return new ColumnDefinition { Width = new GridLength(rowHeader ? this.RowHeaderWidth : this.CellWidth) };
        }
        /// <summary>
        /// Initializes the grid's content based on the number of rows and columns that have been specified.
        /// Will call <see cref="GetHeaderCellContent"/> and <see cref="CreateCellContent"/> to initialize individual header 
        /// and cell content. These methods should be overridden in your derived class. If this is not done, numbered headers
        /// will be inserted and each cell will be initialized with an empty label.
        /// </summary>
        public void InitializeContent()
        {
            IsVisible = false;
            ClearContent();
            AddContent();
            IsVisible = true;
            ReinitializeSelectionFrame();
        }
        /// <summary>
        /// Clears the grid's content
        /// </summary>
        private void ClearContent()
        {
            RowDefinitions.Clear();
            ColumnDefinitions.Clear();
            Children.Clear();
        }
        /// <summary>
        /// Adds the grid's content 
        /// </summary>
        private void AddContent()
        {
            AddRowDefinitions();
            AddColumnDefinitions();
            AddHeaderRow();
            AddRowsContent();
        }
        /// <summary>
        /// Adds the grid's row definitions 
        /// </summary>
        private void AddRowDefinitions()
        {
            RowDefinitions.Add(NewRowDefinition(true));

            for (int row = 0; row < RowCount; row++)
                RowDefinitions.Add(NewRowDefinition(false));
        }
        /// <summary>
        /// Adds the grid's column definitions 
        /// </summary>
        private void AddColumnDefinitions()
        {
            ColumnDefinitions.Add(NewColumnDefinition(true));

            for (int column = 0; column < ColumnCount; column++)
                ColumnDefinitions.Add(NewColumnDefinition(false));
        }
        /// <summary>
        /// Adds the grid's rows content (row header + cells)
        /// </summary>
        private void AddRowsContent()
        {
            for (int row = 0; row < RowCount; row++)
                AddRowContent(row, true);
        }
        /// <summary>
        /// Gets the content for a header cell
        /// </summary>
        /// <param name="type">Header cell type</param>
        /// <param name="index">Header cell index</param>
        /// <returns>View containing content</returns>
        virtual public View GetHeaderCellContent(HeaderType type, int index)
        {
            return new Label
            {
                Text = type != HeaderType.Corner ? $"{index + 1}" : "",
                FontAttributes = FontAttributes.Bold,
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center
            };
        }
        /// <summary>
        /// Creates a new cell frame for a given location
        /// </summary>
        /// <param name="row">Row index</param>
        /// <param name="column">Column index</param>
        /// <param name="gridInitializing">Grid is initializing?</param>
        /// <returns>Cell framw</returns>
        private CellFrame NewCellFrame(int row, int column, bool gridInitializing)
        {
            CellFrame frame = new CellFrame(row, column)
            {
                BackgroundColor = this.CellBackgroundColor,
                Padding = CellPadding,
                Margin = CellMargin == 0.0 ? -0.5 : CellMargin,
                Stroke = new SolidColorBrush(CellBorderColor),
                StrokeThickness = 1,
                StrokeShape = new RoundRectangle
                {
                    CornerRadius = 0
                },
            };
            frame.Content = CreateCellContent(frame, row, column, gridInitializing);
            AddSingleTapGesture(frame);
            AddDoubleTapGesture(frame);

            /*
            Gesture.SetLongPressPointCommand(cellFrame, new Command<PointEventArgs>(args =>
            {
                OnCellLongPressed(cellFrame, currentRow, currentCol);
            }));
            */

            return frame;
        }
        /// <summary>
        /// Creates cell content, where (0,0) corresponds to (1,1) in display terms
        /// </summary>
        /// <param name="frame">Container frame</param>
        /// <param name="row">Row index</param>
        /// <param name="column">Column index</param>
        /// <param name="gridInitializing">Grid is initializing?</param>
        /// <returns>Cell content view</returns>
        virtual public ContentView CreateCellContent(CellFrame frame, int row, int column, bool gridInitializing)
        {
            return new DefaultCellContent();
        }
        /// <summary>
        /// Sets cell content at a given location, where (0,0) corresponds to (1,1) in display terms
        /// </summary>
        /// <param name="row">Row index</param>
        /// <param name="column">Column index</param>
        /// <param name="contentView">Content to attach to cell</param>
        /// <returns>Cell frame</returns>
        public CellFrame? SetCellContent(int row, int column, ContentView contentView)
        {
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame is not null)
                SetCellContent(cellFrame, contentView);
            return cellFrame;
        }
        /// <summary>
        /// Sets cell content within a given cell frame
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="contentView">Content to attach to cell</param>
        public void SetCellContent(CellFrame? cellFrame, ContentView contentView)
        {
            if (cellFrame is not null)
                cellFrame.Content = contentView;
        }
        /// <summary>
        /// Adds single tap gesture support to a given cell frame. This will
        /// trigger the registered OnCellTapped() event handler to be called.
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        private void AddSingleTapGesture(CellFrame cellFrame)
        {
            var tapGesture = new TapGestureRecognizer
            {
                NumberOfTapsRequired = 1
            };
            tapGesture.Tapped += (s, e) => OnCellTapped(cellFrame, true);
            cellFrame.GestureRecognizers.Add(tapGesture);
        }
        /// <summary>
        /// Adds double-tap gesture support to a given cell frame. This will
        /// trigger the registered OnCellDoubleTapped() event handler to be called.
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        private void AddDoubleTapGesture(CellFrame cellFrame)
        {
            var tapGesture = new TapGestureRecognizer
            {
                NumberOfTapsRequired = 2
            };
            tapGesture.Tapped += (s, e) => OnCellDoubleTapped(cellFrame, true);
            cellFrame.GestureRecognizers.Add(tapGesture);
        }
        /// <summary>
        /// Adds a header row to the grid
        /// </summary>
        private void AddHeaderRow()
        {
            for (int column = 0; column < ColumnCount; column++)
                AddColumnHeader(column);
            AddCornerHeader();
        }
        /// <summary>
        /// Adds row content to the grid
        /// </summary>
        /// <param name="row">Row index</param>
        /// <param name="gridInitializing">Grid is initializing?</param>
        /// 
        private void AddRowContent(int row, bool gridInitializing)
        {
            AddRowHeader(row);

            for (int column = 0; column < ColumnCount; column++)
            {
                AddRowCell(row, column, gridInitializing);
            }
        }
        /// <summary>
        /// Adds column content to the grid
        /// </summary>
        /// <param name="column">Column index</param>
        private void AddColumnContent(int column)
        {
            AddColumnHeader(column);

            for (int row = 0; row < RowCount; row++)
            {
                AddRowCell(row, column, false);
            }
        }
        /// <summary>
        /// Adds a row cell to the grid
        /// </summary>
        /// <param name="row">Row index</param>
        /// <param name="column">Column index</param>
        /// <param name="gridInitializing">Grid is initializing?</param>
        /// <returns>Cell frame</returns>
        private CellFrame AddRowCell(int row, int column, bool gridInitializing)
        {
            CellFrame cellFrame = NewCellFrame(row, column, gridInitializing);

            this.Add(cellFrame, cellFrame.DisplayColumn, cellFrame.DisplayRow);

            return cellFrame;
        }
        /// <summary>
        /// Adds a corner header to the grid
        /// </summary>
        private void AddCornerHeader()
        {
            HeaderFrame frame = NewHeaderFrame(HeaderType.Corner, 0);
            var tapGesture = new TapGestureRecognizer();
            tapGesture.Tapped += (s, e) => OnCornerHeaderTapped();
            frame.GestureRecognizers.Add(tapGesture);

            /*
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnCornerHeaderLongPressed(frame);
            }));
            */

            this.Add(frame, 0, 0);
        }
        /// <summary>
        /// Adds a column header to the grid
        /// </summary>
        /// <param name="column">Column index</param>
        private void AddColumnHeader(int column)
        {
            HeaderFrame frame = NewHeaderFrame(HeaderType.Column, column);
            var tapGesture = new TapGestureRecognizer();
            tapGesture.Tapped += (s, e) => OnColumnHeaderTapped(frame);
            frame.GestureRecognizers.Add(tapGesture);

            /*
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnColumnHeaderLongPressed(frame, currentCol);
            }));
            */

            this.Add(frame, column + 1, 0);
        }
        /// <summary>
        /// Adds a row header to the grid
        /// </summary>
        /// <param name="row">Row index</param>
        private void AddRowHeader(int row)
        {
            HeaderFrame frame = NewHeaderFrame(HeaderType.Row, row);
            var tapGesture = new TapGestureRecognizer();
            tapGesture.Tapped += (s, e) => OnRowHeaderTapped(frame);
            frame.GestureRecognizers.Add(tapGesture);

            /*
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnRowHeaderLongPressed(frame, currentRow);
            }));
            */

            this.Add(frame, 0, frame.DisplayIndex);
        }
        /// <summary>
        /// Creates new header frame
        /// </summary>
        /// <param name="type">Header type</param>
        /// <param name="index">Header index</param>
        /// <returns>Header frame</returns>
        private HeaderFrame NewHeaderFrame(HeaderType type, int index)
        {
            HeaderFrame frame = new HeaderFrame(type, index)
            {
                WidthRequest = GetHeaderWidth(type),
                HeightRequest = GetHeaderHeight(type),
                Padding = GetHeaderPadding(type),
                Margin = CellMargin == 0.0 ? -0.5 : CellMargin,
                BackgroundColor = HeaderBackgroundColor,
                Stroke = new SolidColorBrush(HeaderBorderColor),
                StrokeThickness = 1,
                Content = GetHeaderCellContent(type, index),
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center,
            };
            return frame;
        }
        /// <summary>
        /// Gets the padding to be applied for a given header type
        /// </summary>
        /// <param name="type">Header type</param>
        /// <returns>Header padding</returns>
        private Thickness GetHeaderPadding(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return new Thickness(RowHeaderPadding);
                case HeaderType.Column:
                    return new Thickness(ColumnHeaderPadding);
                case HeaderType.Row:
                    return new Thickness(RowHeaderPadding);
            }
            return 0.0;
        }
        /// <summary>
        /// Gets the width to be used for a given header type
        /// </summary>
        /// <param name="type">Header type</param>
        /// <returns>Header width</returns>
        private double GetHeaderWidth(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return this.RowHeaderWidth - (2 * this.RowHeaderMargin);
                case HeaderType.Row:
                    return this.RowHeaderWidth - (2 * this.RowHeaderMargin);
                case HeaderType.Column:
                    return this.CellWidth - (2 * this.CellMargin);
            }
            return 0;
        }
        /// <summary>
        /// Gets the height to be used for a given header type
        /// </summary>
        /// <param name="type">Header type</param>
        /// <returns>Header height</returns>
        private double GetHeaderHeight(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return this.ColumnHeaderHeight - (2 * this.ColumnHeaderMargin);
                case HeaderType.Row:
                    return this.CellHeight - (2 * this.CellMargin);
                case HeaderType.Column:
                    return this.ColumnHeaderHeight - (2 * this.ColumnHeaderMargin);
            }
            return 0;
        }
        /// <summary>
        /// Handles the corner header tapped event
        /// </summary>
        private void OnCornerHeaderTapped()
        {
            SelectCorner();
        }

        /*
        private void OnCornerHeaderLongPressed(Frame frame)
        {
            Debug.WriteLine("Corner header - long pressed");
        }
        */

        /// <summary>
        /// Handles the column header tapped event
        /// </summary>
        /// <param name="headerFrame">Header frame that was tapped</param>
        private void OnColumnHeaderTapped(HeaderFrame headerFrame)
        {
            SelectColumn(headerFrame.DisplayIndex, IsExtendedSelectionMode || IsShiftKeyPressed());
        }

        /*
        private void OnColumnHeaderLongPressed(Frame frame, int column)
        {
            Debug.WriteLine($"Column header - long pressed (column = ${column})");
        }
        */

        /// <summary>
        /// Handles the row header tapped event
        /// </summary>
        /// <param name="headerFrame">Header frame that was tapped</param>
        private void OnRowHeaderTapped(HeaderFrame headerFrame)
        {
            SelectRow(headerFrame.DisplayIndex, IsExtendedSelectionMode || IsShiftKeyPressed());
        }

        /*
        private void OnRowHeaderLongPressed(Frame frame, int row)
        {
            Debug.WriteLine($"Row header - long pressed (row = ${row})");
        }
        */

        /// <summary>
        /// Handles the cell tapped event
        /// </summary>
        /// <param name="cellFrame">Cell frame that was tapped</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        public virtual void OnCellTapped(CellFrame cellFrame, bool triggerEvents)
        {
            ActivateCell(cellFrame, IsExtendedSelectionMode || IsShiftKeyPressed());
        }
        /// <summary>
        /// Handles the cell double-tapped event
        /// </summary>
        /// <param name="cellFrame">Cell frame that was double-tapped</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        public virtual void OnCellDoubleTapped(CellFrame cellFrame, bool triggerEvents)
        {
            ActivateCell(cellFrame, IsExtendedSelectionMode || IsShiftKeyPressed());
        }
        /// <summary>
        /// Handles the key down event
        /// </summary>
        /// <param name="state">Key state</param>
        /// <param name="key">Key pressed</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        public virtual void OnProcessKeyDown(Keyboard.KeyState state, Keyboard.Key key, bool triggerEvents)
        {
            bool shiftPressed = Keyboard.Utility.IsStateFlagSet(state, Keyboard.KeyState.Shift);
            bool ctrlPressed = Keyboard.Utility.IsStateFlagSet(state, Keyboard.KeyState.Ctrl);
            bool capsLockPressed = Keyboard.Utility.IsStateFlagSet(state, Keyboard.KeyState.CapsLock);

            switch (key)
            {
                case Keyboard.Key.Left:
                    MoveActiveCellLeft(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.Right:
                    MoveActiveCellRight(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.Up:
                    MoveActiveCellUp(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.Down:
                    MoveActiveCellDown(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.Home:
                    MoveActiveCellToRowStart(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.End:
                    MoveActiveCellToColumnEnd(shiftPressed, ctrlPressed);
                    break;
                case Keyboard.Key.Tab:
                    if (ctrlPressed) return;
                    if (anchorCell is null)
                    {
                        MoveActiveCellOffset(false, shiftPressed ? -1 : 1, 0);
                        return;
                    }
                    if (shiftPressed)
                        MoveAnchorCellToPrevWithinSelection();
                    else
                        MoveAnchorCellToNextWithinSelection();
                    break;
                default:
                    break;
            }
        }
        /// <summary>
        /// Activates a given cell based on a point definition
        /// </summary>
        /// <param name="point">Cell point</param>
        /// <param name="maintainSelection">Maintain current selection?</param>
        /// <returns>Boolean</returns>
        public bool ActivateCell(CellPoint point, bool maintainSelection)
        {
            return ActivateCell(point.Row, point.Column, maintainSelection);
        }
        /// <summary>
        /// Activates a given cell based on a display row and column where (1,1) is the top-left cell
        /// </summary>
        /// <param name="displayRow">Display row</param>
        /// <param name="displayColumn">Display column</param>
        /// <param name="maintainSelection">Maintain current selection?</param>
        /// <returns>Boolean</returns>
        public bool ActivateCell(int displayRow, int displayColumn, bool maintainSelection)
        {
            bool activated = false;
            if (displayColumn > 0 && displayColumn <= RowCount && displayColumn > 0 && displayColumn <= ColumnCount)
            {
                CellFrame? cellFrame = GetCell(displayRow, displayColumn) as CellFrame;
                if (cellFrame is not null)
                {
                    ActivateCell(cellFrame, maintainSelection);
                    activated = true;
                }
            }
            return activated;
        }
        /// <summary>
        /// Activates a given cell based on a cell frame
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="maintainSelection">Maintain current selection?</param>
        private void ActivateCell(CellFrame cellFrame, bool maintainSelection)
        {
            MoveActiveCell(maintainSelection, cellFrame.DisplayRow, cellFrame.DisplayColumn, true);
        }

        /*
        private void OnCellLongPressed(Frame cell, int row, int column)
        {
            Debug.WriteLine($"Cell - long pressed (row = ${row}, column = {column})");
        }
        */
        /// <summary>
        /// Selects the corner cell (1,1)
        /// </summary>
        private void SelectCorner()
        {
            ClearSelectedCells();
            MoveActiveCell(false, 1, 1, true);
            MoveActiveCell(true, RowCount, ColumnCount, false);
        }
        /// <summary>
        /// Selects a given row
        /// </summary>
        /// <param name="row">Row</param>
        /// <param name="maintainSelection">Maintain current selection?</param>
        private void SelectRow(int row, bool maintainSelection)
        {
            int displayRow = row;
            if (!maintainSelection || anchorCell is null)
            {
                bool hadAnchorCell = anchorCell is not null;
                CellPoint activePoint = activeCellPoint.Clone();
                ClearSelectedCells();
                MoveActiveCell(false, maintainSelection ? activePoint.Row : displayRow, 1, true);
                MoveActiveCell(true, displayRow, ColumnCount, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Column);
            }
            else if (selectedCells is not null)
            {
                int top = selectedCells.Top,
                    bottom = selectedCells.Bottom;
                if (displayRow > anchorCellPoint.Row)
                {
                    top = anchorCellPoint.Row;
                    bottom = displayRow;
                }
                else if (displayRow <= anchorCellPoint.Row)
                {
                    bottom = anchorCellPoint.Row;
                    top = displayRow;
                }
                ClearSelectedCells();
                SelectCells(top, 1, bottom, ColumnCount, false);
                activeCellPoint.Row = displayRow;
            }
        }
        /// <summary>
        /// Selects a given column
        /// </summary>
        /// <param name="column">Column</param>
        /// <param name="maintainSelection">Maintain current selection?</param>
        private void SelectColumn(int column, bool maintainSelection)
        {
            int displayCol = column;
            if (!maintainSelection || anchorCell is null)
            {
                bool hadAnchorCell = anchorCell is not null;
                CellPoint activePoint = activeCellPoint.Clone();
                ClearSelectedCells();
                MoveActiveCell(false, 1, maintainSelection ? activePoint.Column : displayCol, true);
                MoveActiveCell(true, RowCount, displayCol, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Column);
            }
            else if (selectedCells is not null)
            {
                int left = selectedCells.Left,
                    right = selectedCells.Right;
                if (displayCol > anchorCellPoint.Column)
                {
                    left = anchorCellPoint.Column;
                    right = displayCol;
                }
                else if (displayCol <= anchorCellPoint.Column)
                {
                    right = anchorCellPoint.Column;
                    left = displayCol;
                }
                ClearSelectedCells();
                SelectCells(1, left, RowCount, right, false);
                activeCellPoint.Column = displayCol;
            }
        }
        /// <summary>
        /// Enables extended selection mode
        /// </summary>
        public void EnableExtendedSelection()
        {
            if (IsExtendedSelectionMode) return;
            if (anchorCell is null)
                SetAnchorCellToActiveCell(true);
            IsExtendedSelectionMode = true;
            selectionFrame?.EnableDashAnimation(true);
        }
        /// <summary>
        /// Cancels extended selection mode
        /// </summary>
        public void CancelExtendedSelection()
        {
            if (!IsExtendedSelectionMode) return;
            if (anchorCell is not null)
                SetActiveCellToAnchorCell(false);
            ClearAnchorCell(true);
            ClearSelectedCells();
            if (activeCell is not null)
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
            IsExtendedSelectionMode = false;
            selectionFrame?.EnableDashAnimation(false);
            UpdateSelectionFrame(true);
        }
        /// <summary>
        /// Resets the selection to the given selection
        /// </summary>
        /// <param name="newSelection">New selection</param>
        public void ResetSelection(CellRange newSelection)
        {
            CellRange? prevSelection = selectedCells?.Clone();

            ClearSelectedCells();
            SelectCells(Math.Clamp(newSelection.Top, 1, RowCount),
                        Math.Clamp(newSelection.Left, 1, ColumnCount),
                        Math.Clamp(newSelection.Bottom, 1, RowCount),
                        Math.Clamp(newSelection.Right, 1, ColumnCount),
                        false);

            if (selectedCells is null) return;

            if (anchorCell is null && activeCell is not null)
            {
                // Initialize anchor cell
                SetAnchorCellToActiveCell(true);
            }

            if (anchorCell is not null && !selectedCells.ContainsPoint(anchorCellPoint))
            {
                // Move anchor cell
                int newRow = Math.Clamp(anchorCellPoint.Row, selectedCells.Top, selectedCells.Bottom);
                int newColumn = Math.Clamp(anchorCellPoint.Column, selectedCells.Left, selectedCells.Right);
                CellFrame prevAnchorCell = anchorCell;
                MoveAnchorCell(newRow, newColumn);
                prevAnchorCell.BackgroundColor = this.CellBackgroundColor;
            }

            if (prevSelection is not null)
            {
                // Modify active cell if needed
                if (selectedCells.Top != prevSelection.Top)
                    activeCellPoint.Row = selectedCells.Top;
                else if (selectedCells.Bottom != prevSelection.Bottom)
                    activeCellPoint.Row = selectedCells.Bottom;

                if (selectedCells.Left != prevSelection.Left)
                    activeCellPoint.Column = selectedCells.Left;
                else if (selectedCells.Right != prevSelection.Right)
                    activeCellPoint.Column = selectedCells.Right;
            }
        }
        /// <summary>
        /// Gets the current selection state
        /// </summary>
        /// <returns>Selection state</returns>
        public SelectionState GetSelectionState()
        {
            return new SelectionState(
                selectedCells?.Clone(),
                activeCell is not null ? activeCellPoint.Clone() : null,
                anchorCell is not null ? anchorCellPoint.Clone() : null);
        }
        /// <summary>
        /// Restores the selection to the given selection state
        /// </summary>
        /// <param name="selectionState">Selection state</param>
        /// <param name="retainAnchorCell">Reset anchor cell to that defined in the selection state?</param>
        /// <param name="show">Show the selection frame?</param>
        public void RestoreSelectionState(SelectionState selectionState, bool retainAnchorCell, bool show)
        {
            if (retainAnchorCell)
            {
                if (selectionState.AnchorCellPoint is not null)
                {
                    SetAnchorCell(selectionState.AnchorCellPoint);
                    SetActiveCellToAnchorCell(true);
                }
            }
            else if (selectionState.ActiveCellPoint is not null)
            {
                ActivateCell(selectionState.ActiveCellPoint, false);
            }

            if (selectionState.SelectedCells is not null)
            {
                UpdateSelectedCells(new CellPoint(selectionState.SelectedCells.Top, selectionState.SelectedCells.Left),
                                    new CellPoint(selectionState.SelectedCells.Bottom, selectionState.SelectedCells.Right),
                                    true, false);
            }
            ShowSelectionFrame(show);
        }
        /// <summary>
        /// Set ths selction to the given cell range
        /// </summary>
        /// <param name="top">Top row</param>
        /// <param name="left">Left column</param>
        /// <param name="bottom">Bottom row</param>
        /// <param name="right">Right column</param>
        /// <param name="clear">Clear any existing cell highlighting first?</param>
        private void SelectCells(int top, int left, int bottom, int right, bool clear)
        {
            selectedCells = new CellRange(top, left, bottom, right);
            HighlightCells(selectedCells, clear);
            UpdateSelectionFrame(true);
        }
        /// <summary>
        /// Move the active cell left
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToEnd">Move to end?</param>
        private void MoveActiveCellLeft(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int colOffset = moveToEnd ? (useActiveCell ? -activeCellPoint.Column : -anchorCellPoint.Column) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }
        /// <summary>
        /// Move the active cell right
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToEnd">Move to end?</param>
        private void MoveActiveCellRight(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int colOffset = moveToEnd ? this.ColumnCount - (useActiveCell ? activeCellPoint.Column : anchorCellPoint.Column) : 1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }
        /// <summary>
        /// Move the active cell up
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToEnd">Move to end?</param>
        private void MoveActiveCellUp(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int rowOffset = moveToEnd ? (useActiveCell ? -activeCellPoint.Row : -anchorCellPoint.Row) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }
        /// <summary>
        /// Move the active cell down
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToEnd">Move to end?</param>
        private void MoveActiveCellDown(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int rowOffset = moveToEnd ? this.RowCount - (useActiveCell ? activeCellPoint.Row : anchorCellPoint.Row) : 1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }
        /// <summary>
        /// Move the active cell to the start of the row
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToTop">Move to top row?</param>
        private void MoveActiveCellToRowStart(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int rowOffset = moveToTop ? (useActiveCell ? -activeCellPoint.Row : -anchorCellPoint.Row) + 1 : 0;
            int colOffset = useActiveCell ? -activeCellPoint.Column : -anchorCellPoint.Column;
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }
        /// <summary>
        /// Move the active cell to the end of the column
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="moveToTop">Move to top row?</param>
        private void MoveActiveCellToColumnEnd(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell is null);
            int rowOffset = moveToTop ? this.RowCount - (useActiveCell ? activeCellPoint.Row : anchorCellPoint.Row) : 0;
            int colOffset = this.ColumnCount - (useActiveCell ? activeCellPoint.Column : anchorCellPoint.Column);
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }
        /// <summary>
        /// Move the active cell by a given `x`, `y` offset
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="deltaX">X offset?</param>
        /// <param name="deltaY">Y offset?</param>
        private void MoveActiveCellOffset(bool maintainSelection, int deltaX, int deltaY)
        {
            int referenceRow = !maintainSelection && (anchorCell is not null) ? anchorCellPoint.Row : activeCellPoint.Row;
            int referenceCol = !maintainSelection && (anchorCell is not null) ? anchorCellPoint.Column : activeCellPoint.Column;
            int newRow = Math.Clamp(referenceRow + deltaY, 1, this.RowDefinitions.Count);
            int newCol = Math.Clamp(referenceCol + deltaX, 1, this.ColumnDefinitions.Count);

            if (maintainSelection && AllRowsSelected && deltaX != 0 && deltaY == 0)
                SelectColumn(newCol, true);
            else if (maintainSelection && AllColumnsSelected && deltaX == 0 && deltaY != 0)
                SelectRow(newRow, true);
            else
                MoveActiveCell(maintainSelection, newRow, newCol, true);
        }
        /// <summary>
        /// Move the anchor sell to the previous cell within the current selection
        /// </summary>
        private void MoveAnchorCellToPrevWithinSelection()
        {
            if (anchorCell is null || selectedCells is null) return;
            int newCol = anchorCellPoint.Column - 1;
            int newRow = anchorCellPoint.Row;
            if (newCol < selectedCells.Left)
            {
                newCol = selectedCells.Right;
                newRow--;
            }
            if (newRow < selectedCells.Top)
                newRow = selectedCells.Bottom;
            MoveAnchorCell(newRow, newCol);
        }
        /// <summary>
        /// Move the anchor sell to the next cell within the current selection
        /// </summary>
        private void MoveAnchorCellToNextWithinSelection()
        {
            if (anchorCell is null || selectedCells is null) return;
            int newCol = anchorCellPoint.Column + 1;
            int newRow = anchorCellPoint.Row;
            if (newCol > selectedCells.Right)
            {
                newCol = selectedCells.Left;
                newRow++;
            }
            if (newRow > selectedCells.Bottom)
                newRow = selectedCells.Top;
            MoveAnchorCell(newRow, newCol);
        }
        /// <summary>
        /// Move the anchor sell to a given row and column
        /// </summary>
        /// <param name="newRow">New row</param>
        /// <param name="newColumn">New column</param>
        private void MoveAnchorCell(int newRow, int newColumn)
        {
            if (anchorCell is null) return;
            anchorCell.BackgroundColor = this.HighlightCellBackgroundColor;
            ClearAnchorCell(false);
            SetAnchorCell(newRow, newColumn);
            if (anchorCell is not null)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }
        /// <summary>
        /// Move the active sell to a given row and column
        /// </summary>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="newRow">New row</param>
        /// <param name="newColumn">New column</param>
        /// <param name="scrollActiveCellIntoView">Scroll the active cell into view?</param>
        private void MoveActiveCell(bool maintainSelection, int newRow, int newColumn, bool scrollActiveCellIntoView)
        {
            bool wasExtendedSelection = false;
            if (!maintainSelection && anchorCell is not null)
            {
                // Clear anchor cell
                anchorCell.BackgroundColor = this.CellBackgroundColor;
                SetActiveCellToAnchorCell(false);
                ClearAnchorCell(true);
                ClearSelectedCells();
                if (activeCell is not null)
                    activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
                wasExtendedSelection = true;
            }
            // No change in position?
            if (activeCellPoint.IsPosition(newRow, newColumn))
            {
                if (wasExtendedSelection)
                    UpdateSelectionFrame(true);
                return;
            }
            // Find the new active cell
            var newActiveCell = this.Children
                .OfType<CellFrame>()
                .FirstOrDefault(cell => Microsoft.Maui.Controls.Grid.GetRow(cell) == newRow && Microsoft.Maui.Controls.Grid.GetColumn(cell) == newColumn);

            if (newActiveCell is not null && newActiveCell is CellFrame)
            {
                // Scroll the new active cell into view and/or update selection state as needed
                UpdateSelection(newActiveCell as CellFrame, newRow, newColumn, maintainSelection, scrollActiveCellIntoView);
            }
        }
        /// <summary>
        /// Updates the selection
        /// </summary>
        /// <param name="newActiveCell">New active cell</param>
        /// <param name="row">New row</param>
        /// <param name="column">New column</param>
        /// <param name="maintainSelection">Maintain selection?</param>
        /// <param name="scrollActiveCellIntoView">Scroll the active cell into view?</param>
        private void UpdateSelection(CellFrame newActiveCell, int row, int column, bool maintainSelection, bool scrollActiveCellIntoView)
        {
            // Reset the previously active cell if needed
            if (activeCell is not null)
                ClearActiveCellFormatting();

            if (maintainSelection)
            {
                if (anchorCell is null)
                {
                    if (activeCell is null)
                        SetAnchorCell(row, column);
                    else
                        SetAnchorCell(activeCellPoint.Row, activeCellPoint.Column);
                }
            }
            else
            {
                ClearSelectedCells();
                ClearAnchorCell(true);
            }

            // Set the new active cell
            activeCell = newActiveCell;
            if (anchorCell is not null)
                anchorCell.BackgroundColor = this.AnchorCellBackgroundColor;
            else
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;

            activeCellPoint.Set(row, column);

            if (anchorCell is not null)
                UpdateSelectedCells(anchorCellPoint, activeCellPoint, true, true);
            else
            {
                HighlightActiveCellHeaders(false);
                UpdateSelectionFrame(true);
            }

            if (scrollActiveCellIntoView)
                ScrollCellIntoView(newActiveCell);
        }
        /// <summary>
        /// Scrolls a cell frame into view as needed
        /// </summary>
        /// <param name="cell">Cell frame to scroll into view</param>
        private async void ScrollCellIntoView(CellFrame cell)
        {
            // Handle scroll
            double cellWidth = cell.Bounds.Width;
            double cellLeftX = cell.Bounds.X;
            double cellRightX = cellLeftX + cellWidth - 1;
            double cellHeight = cell.Bounds.Height;
            double cellTopY = cell.Bounds.Y;
            double cellBottomY = cellTopY + cellHeight - 1;
            double currentScrollX = ContainerScrollView.ScrollX;
            double scrollViewWidth = ContainerScrollView.Width;
            double scrollMaxVisibleX = currentScrollX + scrollViewWidth;
            double scrollViewHeight = ContainerScrollView.Height;
            double currentScrollY = ContainerScrollView.ScrollY;
            double scrolMaxlVisibleY = currentScrollY + scrollViewHeight;

            // If the cell is already fully visible, there is no need to scroll
            if (cellLeftX >= currentScrollX && cellRightX <= scrollMaxVisibleX &&
                cellBottomY >= currentScrollY && cellBottomY <= scrolMaxlVisibleY)
            {
                return;
            }

            // Calculate scroll adjustments (if any)
            double targetX = currentScrollX;
            double targetY = currentScrollY;

            if (cellLeftX < currentScrollX)
                targetX = cellLeftX;
            else if (cellRightX > scrollMaxVisibleX)
                targetX = cellRightX - scrollViewWidth;
            else
                targetX = currentScrollX;

            if (cellTopY < currentScrollY)
                targetY = cellTopY;
            else if (cellBottomY > (currentScrollY + scrollViewHeight))
                targetY = cellBottomY - scrollViewHeight;
            else
                targetY = currentScrollY;

            await ContainerScrollView.ScrollToAsync(targetX, targetY, true);
        }

#if !WINDOWS
        /// <summary>
        /// Returns whether the shift key is currently pressed. This is currently a place-holder for non-Windows builds.
        /// </summary>
        /// <returns>Boolean</returns>
        private static bool IsShiftKeyPressed()
        {
            return false;
        }
#endif
        /// <summary>
        /// Sets the anchor cell location based on row and column
        /// </summary>
        /// <param name="row">New row</param>
        /// <param name="column">New column</param>
        private void SetAnchorCell(int row, int column)
        {
            anchorCell = GetCell(row, column) as CellFrame;
            if (anchorCell is not null)
                anchorCellPoint.Set(row, column);
            else
                anchorCellPoint.Clear();
        }
        /// <summary>
        /// Sets the anchor cell location based on a cell point
        /// </summary>
        /// <param name="point">Cell point</param>
        private void SetAnchorCell(CellPoint point)
        {
            SetAnchorCell(point.Row, point.Column);
        }
        /// <summary>
        /// Clears any anchor cell formatting
        /// </summary>
        void ClearAnchorCellFormatting()
        {
            if (anchorCell is not null)
                anchorCell.BackgroundColor = this.CellBackgroundColor;
        }
        /// <summary>
        /// Clears the anchor cell
        /// </summary>
        /// <param name="clearFormatting">Clear formatting?</param>
        private void ClearAnchorCell(bool clearFormatting)
        {
            if (clearFormatting)
                ClearAnchorCellFormatting();
            anchorCell = null;
            anchorCellPoint.Clear();
        }
        /// <summary>
        /// Sets the anchor cell to the current active cell
        /// </summary>
        /// <param name="setBackgroundColor">Set background color?</param>
        private void SetAnchorCellToActiveCell(bool setBackgroundColor)
        {
            anchorCell = activeCell;
            anchorCellPoint = activeCellPoint.Clone();
            if (anchorCell is not null && setBackgroundColor)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }
        /// <summary>
        /// Sets the active cell to the current anchor cell
        /// </summary>
        /// <param name="setBackgroundColor">Set background color?</param>
        private void SetActiveCellToAnchorCell(bool setBackgroundColor)
        {
            activeCell = anchorCell;
            activeCellPoint = anchorCellPoint.Clone();
            if (activeCell is not null && setBackgroundColor)
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }
        /// <summary>
        /// Clears the current selection
        /// </summary>
        private void ClearSelectedCells()
        {
            if (selectedCells is not null)
            {
                HighlightCells(selectedCells, true);
                ShowSelectionFrame(false);
                selectedCells = null;
            }
        }
        /// <summary>
        /// Updates the current selection
        /// </summary>
        /// <param name="fromPoint">From cell point</param>
        /// <param name="toPoint">To cell point</param>
        /// <param name="updateSelectionFrame">Update selection frame?</param>
        /// <param name="triggerEvents">Trigger further events?</param>
        private void UpdateSelectedCells(CellPoint fromPoint, CellPoint toPoint, bool updateSelectionFrame, bool triggerEvents)
        {
            ClearSelectedCells();
            int startRow = Math.Min(fromPoint.Row, toPoint.Row);
            int startCol = Math.Min(fromPoint.Column, toPoint.Column);
            int width = Math.Abs(fromPoint.Column - toPoint.Column) + 1;
            int height = Math.Abs(fromPoint.Row - toPoint.Row) + 1;
            selectedCells = new CellRange(startRow, startCol, startRow + height - 1, startCol + width - 1);
            HighlightCells(selectedCells, false);
            if (updateSelectionFrame)
                UpdateSelectionFrame(triggerEvents);
        }
        /// <summary>
        /// Initializes the selection frame
        /// </summary>
        private void InitializeSelectionFrame()
        {
            selectionFrame = new SelectionFrame(this, IsPanSupportEnabled)
            {
                BorderColor = SelectionFrameBorderColor,
                BorderWidth = SelectionFrameBorderWidth,
                BorderGripDiameter = SelectionFrameBorderGripDiameter
            };
            selectionFrame.AddToGrid();
        }
        /// <summary>
        /// Removes the selection frame
        /// </summary>
        private void RemoveSelectionFrame()
        {
            selectionFrame?.RemoveFromGrid();
            selectionFrame = null;
        }
        /// <summary>
        /// Reinitializes the selection frame
        /// </summary>
        private void ReinitializeSelectionFrame()
        {
            RemoveSelectionFrame();
            InitializeSelectionFrame();
        }
        /// <summary>
        /// Updates the selection frame
        /// </summary>
        /// <param name="triggerEvents">Trigger further events?</param>
        /// <returns>True if the selection changed </returns>
        private bool UpdateSelectionFrame(bool triggerEvents)
        {
            if (selectionFrame is null)
                return false;

            CellRange? prevSelection = selectionFrame.CellRange?.Clone();
            selectionFrame.SetRange(selectedCells is not null ? selectedCells : new CellRange(activeCellPoint.Row, activeCellPoint.Column), true);
            CellRange? newSelection = selectionFrame.CellRange?.Clone();
            bool selectionChange = (prevSelection is null) || !prevSelection.Equals(newSelection);

            if (selectionChange && triggerEvents)
                OnSelectionChanged();

            return selectionChange;
        }
        /// <summary>
        /// Overrideable handler for selection changed event
        /// </summary>
        public virtual void OnSelectionChanged()
        {
        }
        /// <summary>
        /// Shows or hidea the selection frame
        /// </summary>
        /// <param name="show">Show?</param>
        private void ShowSelectionFrame(bool show)
        {
            if (selectionFrame is null) return;
            selectionFrame.Show(show);
        }
        /// <summary>
        /// Updates the highlighting of a cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <param name="clear">Clear the highlighting?</param>
        private void HighlightCells(CellRange range, bool clear)
        {
            HighlightHeaders(range, clear);

            for (int row = range.Top; row <= range.Bottom; row++)
            {
                for (int column = range.Left; column <= range.Right; column++)
                {
                    if (row != anchorCellPoint.Row || column != anchorCellPoint.Column)
                    {
                        CellFrame? cellFrame = GetCell(row, column) as CellFrame;
                        if (cellFrame is not null)
                            cellFrame.BackgroundColor = clear ? this.CellBackgroundColor : this.HighlightCellBackgroundColor;
                    }
                }
            }
        }
        /// <summary>
        /// Clears the active cell's formatting
        /// </summary>
        private void ClearActiveCellFormatting()
        {
            if (activeCell is null) return;
            activeCell.BackgroundColor = this.CellBackgroundColor;
            HighlightActiveCellHeaders(true);
        }
        /// <summary>
        /// Clears the active cell
        /// </summary>
        /// <param name="clearFormatting">Clear formatting?</param>
        private void ClearActiveCell(bool clearFormatting)
        {
            if (activeCell is not null)
            {
                if (clearFormatting)
                    ClearActiveCellFormatting();
                activeCellPoint.Clear();
                activeCell = null;
            }
        }
        /// <summary>
        /// Updates the highlighting of the active cell's headers
        /// </summary>
        /// <param name="clear">Clear highlighting?</param>
        private void HighlightActiveCellHeaders(bool clear)
        {
            if (activeCell is null) return;
            HighlightHeaders(new CellRange(activeCellPoint), clear);
        }
        /// <summary>
        /// Updates the highlighting of the headers associated with a given cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <param name="clear">Clear highlighting?</param>
        private void HighlightHeaders(CellRange range, bool clear)
        {
            HighlightRowHeaders(range, clear);
            HighlightColHeaders(range, clear);
        }
        /// <summary>
        /// Updates the highlighting of the row headers associated with a given cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <param name="clear">Clear highlighting?</param>
        private void HighlightRowHeaders(CellRange range, bool clear)
        {
            bool allColumnsSelected = range.Left == 1 && range.Right == ColumnCount;
            for (int row = range.Top; row <= range.Bottom; row++)
                HighlightRowHeader(row, clear, allColumnsSelected);
        }
        /// <summary>
        /// Updates the highlighting of a row header
        /// </summary>
        /// <param name="row">Row</param>
        /// <param name="clear">Clear highlighting?</param>
        /// <param name="allColumnsSelected">All columns selected?</param>
        private void HighlightRowHeader(int row, bool clear, bool allColumnsSelected)
        {
            HeaderFrame? header = GetRowHeaderCell(row);
            if (header is not null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allColumnsSelected ? this.HeaderSelectedBackgroundColor : this.HeaderActiveBackgroundColor);
        }
        /// <summary>
        /// Updates the highlighting of the column headers associated with a given cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <param name="clear">Clear highlighting?</param>
        private void HighlightColHeaders(CellRange range, bool clear)
        {
            bool allRowsSelected = range.Top == 1 && range.Bottom == RowCount;
            for (int column = range.Left; column <= range.Right; column++)
                HighlightColHeader(column, clear, allRowsSelected);
        }
        /// <summary>
        /// Updates the highlighting of a column header
        /// </summary>
        /// <param name="column">Column</param>
        /// <param name="clear">Clear highlighting?</param>
        /// <param name="allRowsSelected">All rows selected?</param>
        private void HighlightColHeader(int column, bool clear, bool allRowsSelected)
        {
            HeaderFrame? header = GetColHeaderCell(column);
            if (header is not null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allRowsSelected ? this.HeaderSelectedBackgroundColor : HeaderActiveBackgroundColor);
        }
        /// <summary>
        /// Gets the header associated with a given row
        /// </summary>
        /// <param name="row">Row</param>
        /// <returns>Header frame</returns>
        private HeaderFrame? GetRowHeaderCell(int row)
        {
            return GetCell(row, 0) as HeaderFrame;
        }
        /// <summary>
        /// Gets the header associated with a given column
        /// </summary>
        /// <param name="column">Column</param>
        /// <returns>Header frame</returns>
        private HeaderFrame? GetColHeaderCell(int column)
        {
            return GetCell(0, column) as HeaderFrame;
        }
        /// <summary>
        /// Gets the total display width associated with a given cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <returns>Total display width</returns>
        public double GetCellsWidth(CellRange range)
        {
            if (range is null) return 0.0;
            double width = 0.0;
            for (int column = range.Left; column <= range.Right && column <= ColumnCount; column++)
                width += GetColumnWidth(column);
            return width;
        }
        /// <summary>
        /// Gets the display width associated with a given column
        /// </summary>
        /// <param name="column">Column</param>
        /// <returns>Display width</returns>
        public double GetColumnWidth(int column)
        {
            if (column < 0 || column > ColumnCount) return 0.0;
            return ColumnDefinitions[column].Width.Value;
        }
        /// <summary>
        /// Gets the total display height associated with a given cell range
        /// </summary>
        /// <param name="range">Cell range</param>
        /// <returns>Total display height</returns>
        public double GetCellsHeight(CellRange range)
        {
            if (range is null) return 0.0;
            double height = 0.0;
            for (int row = range.Top; row <= range.Bottom && row <= RowCount; row++)
                height += GetRowHeight(row);
            return height;
        }
        /// <summary>
        /// Gets the display height associated with a given row
        /// </summary>
        /// <param name="row">Row</param>
        /// <returns>Display height</returns>
        public double GetRowHeight(int row)
        {
            if (row < 0 || row > RowCount) return 0.0;
            return RowDefinitions[row].Height.Value;
        }
        /// <summary>
        /// Locates the cell row number offset from a starting row
        /// </summary>
        /// <param name="startRow">Start row against which offset is measured</param>
        /// <param name="type">Offset type</param>
        /// <param name="offset">Offset amount</param>
        /// <returns>Row number</returns>
        public int FindCellRowAtYOffset(int startRow, YOffsetType type, double offset)
        {
            if (offset == 0 ||
                (offset > 0 && type == YOffsetType.BottomEdge && startRow == RowCount) ||
                (offset < 0 && type == YOffsetType.TopEdge && startRow <= 1))
            {
                return startRow;
            }
            int row = FindCellNextRowForYOffset(startRow, type, offset),
                rowIncrement = offset > 0 ? 1 : -1;
            double offsetRemaining = Math.Abs(offset), rowHeight = 0.0;

            while (row >= 1 && row <= RowCount)
            {
                rowHeight = GetRowHeight(row);
                if (offsetRemaining <= rowHeight)
                    break;
                offsetRemaining -= rowHeight;
                row += rowIncrement;
            }
            return Math.Clamp(row, 1, RowCount);
        }
        /// <summary>
        /// Locates the next cell row number offset row-wise from a starting row
        /// </summary>
        /// <param name="startRow">Start row against which offset is measured</param>
        /// <param name="type">Offset type</param>
        /// <param name="offset">Offset amount</param>
        /// <returns>Row number</returns>
        private int FindCellNextRowForYOffset(int startRow, YOffsetType type, double offset)
        {
            int nextRow = startRow;
            switch (type)
            {
                case YOffsetType.TopEdge:
                    nextRow = offset > 0 ? startRow : startRow - 1;
                    break;
                case YOffsetType.BottomEdge:
                    nextRow = offset > 0 ? startRow + 1 : startRow;
                    break;
            }
            return Math.Clamp(nextRow, 1, ColumnCount);
        }
        /// <summary>
        /// Locates the cell column number offset from a starting column
        /// </summary>
        /// <param name="startColumn">Start column against which offset is measured</param>
        /// <param name="type">Offset type</param>
        /// <param name="offset">Offset amount</param>
        /// <returns>Column number</returns>
        public int FindCellColumnAtXOffset(int startColumn, XOffsetType type, double offset)
        {
            if (offset == 0 ||
                (offset > 0 && type == XOffsetType.RightEdge && startColumn == ColumnCount) ||
                (offset < 0 && type == XOffsetType.LeftEdge && startColumn <= 1))
            {
                return startColumn;
            }
            int column = FindCellNextColumnForXOffset(startColumn, type, offset),
                columnIncrement = offset > 0 ? 1 : -1;
            double offsetRemaining = Math.Abs(offset), columnWidth = 0.0;

            while (column >= 1 && column <= ColumnCount)
            {
                columnWidth = GetColumnWidth(column);
                if (offsetRemaining <= columnWidth)
                    break;
                offsetRemaining -= columnWidth;
                column += columnIncrement;
            }
            return Math.Clamp(column, 1, ColumnCount);
        }
        /// <summary>
        /// Locates the next cell column number offset column-wise from a starting row
        /// </summary>
        /// <param name="startColumn">Start column against which offset is measured</param>
        /// <param name="type">Offset type</param>
        /// <param name="offset">Offset amount</param>
        /// <returns>Column number</returns>
        private int FindCellNextColumnForXOffset(int startColumn, XOffsetType type, double offset)
        {
            int nextColumn = startColumn;
            switch (type)
            {
                case XOffsetType.LeftEdge:
                    nextColumn = offset > 0 ? startColumn : startColumn - 1;
                    break;
                case XOffsetType.RightEdge:
                    nextColumn = offset > 0 ? startColumn + 1 : startColumn;
                    break;
            }
            return Math.Clamp(nextColumn, 1, ColumnCount);
        }
        /// <summary>
        /// Determines whether a given view object is part of a selection frame
        /// </summary>
        /// <param name="child">Child view</param>
        /// <returns>Boolean</returns>
        private bool IsSelectionFrameObject(IView child)
        {
            return (child is BorderGrip || child is BorderBox);
        }
        /// <summary>
        /// Determines whether a given view object is a grid cell or header
        /// </summary>
        /// <param name="child">Child view</param>
        /// <returns>Boolean</returns>
        private bool IsGridCellOrHeader(IView child)
        {
            return !IsSelectionFrameObject(child);
        }
        /// <summary>
        /// Gets the cell associated with a given row and column
        /// </summary>
        /// <param name="row">Row</param>
        /// <param name="column">Column</param>
        /// <returns>Cell border</returns>
        protected Border? GetCell(int row, int column)
        {
            foreach (var child in this.Children)
            {
                if (IsGridCellOrHeader(child) &&
                    this.GetRow(child) == row &&
                    this.GetColumn(child) == column)
                {
                    return child as Border;
                }
            }

            return null;
        }
        /// <summary>
        /// Deletes the selected rows, providing all columns are selected and there is more than one row
        /// </summary>
        /// <returns>Boolean</returns>
        public bool DeleteSelectedRows()
        {
            bool deleted = false;

            if (AllColumnsSelected && RowCount > 1)
            {
                if(selectedCells is not null)
                    deleted = DeleteRows(selectedCells.Top, selectedCells.Bottom);
                else if (activeCell is not null)
                    deleted = DeleteRows(activeCellPoint.Row, activeCellPoint.Row);
            }

            return deleted;
        }
        /// <summary>
        /// Inserts rows before the selection, providing all columns are selected
        /// </summary>
        /// <returns>Boolean</returns>
        public bool InsertSelectedRows()
        {
            bool inserted = false;

            if(AllColumnsSelected)
            {
                if (selectedCells is not null)
                    inserted = InsertRows(selectedCells.Top, selectedCells.Bottom);
                else if (activeCell is not null)
                    inserted = InsertRows(activeCellPoint.Row, activeCellPoint.Row);
            }

            return inserted;
        }
        /// <summary>
        /// Deletes the selected columns, providing all rows are selected and there is more than one column
        /// </summary>
        /// <returns>Boolean</returns>
        public bool DeleteSelectedColumns()
        {
            bool deleted = false;

            if (AllRowsSelected && ColumnCount > 1)
            {
                if (selectedCells is not null)
                    deleted = DeleteColumns(selectedCells.Left, selectedCells.Right);
                else if (activeCell is not null)
                    deleted = DeleteColumns(activeCellPoint.Column, activeCellPoint.Column);
            }

            return deleted;
        }
        /// <summary>
        /// Inserts columns before the selection, providing all rows are selected
        /// </summary>
        /// <returns>Boolean</returns>
        public bool InsertSelectedColumns()
        {
            bool inserted = false;

            if (AllRowsSelected )
            {
                if (selectedCells is not null)
                    inserted = InsertColumns(selectedCells.Left, selectedCells.Right);
                else if (activeCell is not null)
                    inserted = InsertColumns(activeCellPoint.Column, activeCellPoint.Column);
            }

            return inserted;
        }
        /// <summary>
        /// Deletes the given range of rows
        /// </summary>
        /// <param name="startDisplayRow">Start display row</param>
        /// <param name="endDisplayRow">End display row</param>
        /// <returns>Boolean</returns>
        public bool DeleteRows(int startDisplayRow, int endDisplayRow)
        {
            bool deleted = false;

            if (IsValidDisplayRow(startDisplayRow) && IsValidDisplayRow(endDisplayRow))
                deleted = Remove(Target.Row, startDisplayRow, endDisplayRow, true);

            return deleted;
        }
        /// <summary>
        /// Inserts rows before the given range of rows
        /// </summary>
        /// <param name="startDisplayRow">Start display row</param>
        /// <param name="endDisplayRow">End display row</param>
        /// <returns>Boolean</returns>
        public bool InsertRows(int startDisplayRow, int endDisplayRow)
        {
            bool inserted = false;

            if (IsValidDisplayRow(startDisplayRow) && IsValidDisplayRow(endDisplayRow))
                inserted = Insert(Target.Row, startDisplayRow, endDisplayRow, true);

            return inserted;
        }
        /// <summary>
        /// Deletes the given range of columns
        /// </summary>
        /// <param name="startDisplayColumn">Start display column</param>
        /// <param name="endDisplayColumn">End display column</param>
        /// <returns>Boolean</returns>
        public bool DeleteColumns(int startDisplayColumn, int endDisplayColumn)
        {
            bool deleted = false;

            if (IsValidDisplayColumn(startDisplayColumn) && IsValidDisplayColumn(endDisplayColumn))
                deleted = Remove(Target.Column, startDisplayColumn, endDisplayColumn, true);

            return deleted;
        }
        /// <summary>
        /// Inserts columns before the given range of columns
        /// </summary>
        /// <param name="startDisplayColumn">Start display column</param>
        /// <param name="endDisplayColumn">End display column</param>
        /// <returns>Boolean</returns>
        public bool InsertColumns(int startDisplayColumn, int endDisplayColumn)
        {
            bool inserted = false;

            if (IsValidDisplayColumn(startDisplayColumn) && IsValidDisplayColumn(endDisplayColumn))
                inserted = Insert(Target.Column, startDisplayColumn, endDisplayColumn, true);

            return inserted;
        }
        /// <summary>
        /// Represents a type of positional target
        /// </summary>
        private enum Target
        {
            /// <summary>
            /// Row target
            /// </summary>
            Row = 1,
            /// <summary>
            /// Column target
            /// </summary>
            Column = 2
        }
        /// <summary>
        /// Removes a range of targets from the grid
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="startPosition">Start position</param>
        /// <param name="endPosition">End positions</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        /// <returns>Boolean</returns>
        private bool Remove(Target target, int startPosition, int endPosition, bool triggerEvents)
        {
            bool removed = false;
            bool retainAnchorCell = anchorCell is not null;
            SelectionState selectionState = GetSelectionState();

            ClearSelectedCells();
            ClearAnchorCell(true);
            ClearActiveCell(true);

            removed = RemoveChildren(target, startPosition, endPosition);

            if (removed)
            {
                switch (target)
                {
                    case Target.Row:
                        selectionState.ClampRows(RowCount);
                        break;

                    case Target.Column:
                        selectionState.ClampColumns(ColumnCount);
                        break;
                }
            }

            RestoreSelectionState(selectionState, retainAnchorCell, true);

            if (removed && triggerEvents)
                OnSelectionChanged();

            return removed;
        }
        /// <summary>
        /// Removes a range of children from the grid
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="startPosition">Start position</param>
        /// <param name="endPosition">End positions</param>
        /// <returns>Boolean</returns>
        private bool RemoveChildren(Target target, int startPosition, int endPosition)
        {
            if (endPosition < startPosition)
            {
                return false;
            }

            bool removed = false;

            for (int position = startPosition; position <= endPosition; position++)
            {
                if (RemoveChildrenAt(target, position, false))
                    removed = true;
            }

            if (removed)
                ResetChildPositions(target, startPosition, -(endPosition - startPosition + 1));

            return removed;
        }
        /// <summary>
        /// Removes a children at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        /// <param name="resetPositions">Reset positions?</param>
        /// <returns>Boolean</returns>
        private bool RemoveChildrenAt(Target target, int position, bool resetPositions)
        {
            if (position <= 0)
                return false;

            RemoveChildElementsAt(target, position);
            RemoveDefinitionAt(target, position);

            if (resetPositions)
                ResetChildPositions(target, position, -1);

            AdjustTargetCount(target, -1);

            return true;
        }
        /// <summary>
        /// Removes child elements at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        private void RemoveChildElementsAt(Target target, int position)
        {
            for (int i = Children.Count - 1; i >= 0; i--)
            {
                var child = Children[i];
                if (IsGridCellOrHeader(child))
                {
                    if (IsValidRemoveTarget(target, child, position))
                        Children.RemoveAt(i);
                }
            }
        }
        /// <summary>
        /// Determines whether a given child view can be removed
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="child">Child view</param>
        /// <param name="position">Position index</param>
        /// <returns>Boolean</returns>
        private bool IsValidRemoveTarget(Target target, IView child, int position)
        {
            if (IsGridCellOrHeader(child))
            {
                switch (target)
                {
                    case Target.Row:
                        return GetRow(child) == position;
                    case Target.Column:
                        return GetColumn(child) == position;
                }
            }

            return false;
        }
        /// <summary>
        /// Resets the positions associated with the grid children
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="startPosition">Start position</param>
        /// <param name="positionChange">The change in position</param>
        private void ResetChildPositions(Target target, int startPosition, int positionChange)
        {
            if (positionChange == 0) return;

            foreach (var child in Children)
            {
                int currentPosition = GetChildPosition(target, child);
                if (currentPosition > startPosition)
                    SetChildPosition(target, child, currentPosition + positionChange);
            }
        }
        /// <summary>
        /// Inserts a range of targets into the grid
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="startPosition">Start position</param>
        /// <param name="endPosition">End positions</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        /// <returns>Boolean</returns>
        private bool Insert(Target target, int startPosition, int endPosition, bool triggerEvents)
        {
            bool inserted = false;
            bool retainAnchorCell = anchorCell is not null;
            SelectionState selectionState = GetSelectionState();

            ClearSelectedCells();
            ClearAnchorCell(true);
            ClearActiveCell(true);

            inserted = InsertChildren(target, startPosition, endPosition);

            ReinitializeSelectionFrame();

            RestoreSelectionState(selectionState, retainAnchorCell, true);

            if (inserted && triggerEvents)
                OnSelectionChanged();

            return inserted;
        }
        /// <summary>
        /// Inserts a range of children into the grid
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="startPosition">Start position</param>
        /// <param name="endPosition">End positions</param>
        /// <returns>Boolean</returns>
        private bool InsertChildren(Target target, int startPosition, int endPosition)
        {
            if (endPosition < startPosition)
            {
                return false;
            }

            bool inserted = false;

            for (int position = startPosition; position <= endPosition; position++)
            {
                if (InsertChildrenAt(target, position))
                    inserted = true;
            }

            return inserted;
        }
        /// <summary>
        /// Inserts child elements at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        /// <returns>Boolean</returns>
        private bool InsertChildrenAt(Target target, int position)
        {
            if (position <= 0)
                return false;

            InsertDefinitionAt(target, position);
            ResetChildPositions(target, position - 1, 1);
            InsertChildContentAt(target, position);
            AdjustTargetCount(target, 1);

            return true;
        }
        /// <summary>
        /// Inserts child content at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        private void InsertChildContentAt(Target target, int position)
        {
            switch (target)
            {
                case Target.Row:
                    AddRowContent(position - 1, false);
                    break;
                case Target.Column:
                    AddColumnContent(position - 1);
                    break;
            }
        }
        /// <summary>
        /// Removes a definition at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        private void RemoveDefinitionAt(Target target, int position)
        {
            switch (target)
            {
                case Target.Column:
                    if (ColumnDefinitions.Count > position)
                        ColumnDefinitions.RemoveAt(position);
                    break;
                case Target.Row:
                    if (RowDefinitions.Count > position)
                        RowDefinitions.RemoveAt(position);
                    break;
            }
        }
        /// <summary>
        /// Inserts a definition at a given position
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="position">Position index</param>
        private void InsertDefinitionAt(Target target, int position)
        {
            switch (target)
            {
                case Target.Column:
                    if (position <= ColumnDefinitions.Count)
                        ColumnDefinitions.Insert(position, NewColumnDefinition(false));
                    break;
                case Target.Row:
                    if (position <= RowDefinitions.Count)
                        RowDefinitions.Insert(position, NewRowDefinition(false));
                    break;
            }
        }
        /// <summary>
        /// Gets  the position associated with a given child view
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="child">Child view</param>
        /// <returns>Position index</returns>
        private int GetChildPosition(Target target, IView child)
        {
            switch (target)
            {
                case Target.Row:
                    return GetRow(child);
                case Target.Column:
                    return GetColumn(child);
            }
            return -1;
        }
        /// <summary>
        /// Sets the position associated with a given child view
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="child">Child view</param>
        /// <param name="newPosition">New position</param>
        private void SetChildPosition(Target target, IView child, int newPosition)
        {
            switch (target)
            {
                case Target.Column:
                    SetChildColumn(child, newPosition);
                    break;
                case Target.Row:
                    SetChildRow(child, newPosition);
                    break;
            }
        }
        /// <summary>
        /// Sets the column associated with a given child view
        /// </summary>
        /// <param name="child">Child view</param>
        /// <param name="newColumn">New column</param>
        private void SetChildColumn(IView child, int newColumn)
        {
            int row = GetRow(child);
            SetColumn(child, newColumn);
            if (row == 0)
                UpdateHeaderIndex(HeaderType.Column, child, newColumn - 1);
            else
                UpdateCellIndex(Target.Column, child, newColumn - 1);
        }
        /// <summary>
        /// Sets the row associated with a given child view
        /// </summary>
        /// <param name="child">Child view</param>
        /// <param name="newRow">New row</param>
        private void SetChildRow(IView child, int newRow)
        {
            int column = GetColumn(child);
            SetRow(child, newRow);
            if (column == 0)
                UpdateHeaderIndex(HeaderType.Row, child, newRow - 1);
            else
                UpdateCellIndex(Target.Row, child, newRow - 1);
        }
        /// <summary>
        /// Updates the cell index associated with a given child view
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="child">Child view</param>
        /// <param name="newIndex">New index</param>
        private void UpdateCellIndex(Target target, IView child, int newIndex)
        {
            CellFrame? cellFrame = child as CellFrame;
            if (cellFrame is not null)
            {
                switch (target)
                {
                    case Target.Row:
                        cellFrame.Row = newIndex;
                        break;
                    case Target.Column:
                        cellFrame.Column = newIndex;
                        break;
                }
            }
        }
        /// <summary>
        /// Updates the header index associated with a given child view
        /// </summary>
        /// <param name="expectedType">Expected header type</param>
        /// <param name="child">Child view</param>
        /// <param name="newIndex">New index</param>
        private void UpdateHeaderIndex(HeaderType expectedType, IView child, int newIndex)
        {
            HeaderFrame? headerFrame = child as HeaderFrame;
            if (headerFrame is not null && headerFrame.Type == expectedType)
            {
                headerFrame.Index = newIndex;
                headerFrame.Content = GetHeaderCellContent(headerFrame.Type, headerFrame.Index);
            }
        }
        /// <summary>
        /// Updates the row or column counts by the given amount
        /// </summary>
        /// <param name="target">Target type</param>
        /// <param name="addAmount">Amount to add</param>
        private void AdjustTargetCount(Target target, int addAmount)
        {
            switch (target)
            {
                case Target.Column:
                    ColumnCount = ColumnCount + addAmount;
                    break;
                case Target.Row:
                    RowCount = RowCount + addAmount;
                    break;
            }
        }
        /// <summary>
        /// Tests whether a given display row is valid
        /// </summary>
        /// <param name="displayRow">Display row</param>
        /// <returns>Boolean</returns>
        public bool IsValidDisplayRow(int displayRow)
        {
            return displayRow >= 0 && displayRow <= RowCount;
        }
        /// <summary>
        /// Tests whether a given display column is valid
        /// </summary>
        /// <param name="displayColumn">Display column</param>
        /// <returns>Boolean</returns>
        public bool IsValidDisplayColumn(int displayColumn)
        {
            return displayColumn >= 0 && displayColumn <= ColumnCount;
        }

    }
}
