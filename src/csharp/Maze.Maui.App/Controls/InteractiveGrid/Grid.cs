using MauiGestures;
using Microsoft.Maui.Controls;
using Microsoft.Maui.Controls.Shapes;
using System.Data.Common;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public partial class Grid : Microsoft.Maui.Controls.Grid
    {
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

        public enum XOffsetType
        {
            LeftEdge = 0,
            RightEdge = 1
        }

        public enum YOffsetType
        {
            TopEdge = 0,
            BottomEdge = 1
        }

        static private Color GRID_LIGHT_GREEN = Color.FromRgb(211, 240, 224);
        static private Color GRID_VERY_LIGHT_GRAY = Color.FromRgb(240, 240, 240);
        static private Color GRID_LIGHT_GRAY = Color.FromRgb(225, 225, 225);

        public int RowCount { get; set; } = 0;

        public int ColumnCount { get; set; } = 0;

        public double ColumnHeaderHeight { get; set; } = DEFAULT_COL_HEADER_HEIGHT;

        public double ColumnHeaderMargin { get; set; } = DEFAULT_COL_HEADER_MARGIN;

        public double ColumnHeaderPadding { get; set; } = DEFAULT_COL_HEADER_PADDING;

        public double RowHeaderWidth { get; set; } = DEFAULT_ROW_HEADER_WIDTH;

        public double RowHeaderMargin { get; set; } = DEFAULT_ROW_HEADER_MARGIN;

        public double RowHeaderPadding { get; set; } = DEFAULT_ROW_HEADER_PADDING;

        public double CellHeight { get; set; } = DEFAULT_CELL_HEIGHT;

        public double CellWidth { get; set; } = DEFAULT_CELL_WIDTH;

        public double CellMargin { get; set; } = DEFAULT_CELL_MARGIN;

        public double CellPadding { get; set; } = DEFAULT_CELL_PADDING;

        public Color HeaderBorderColor { get; set; } = Colors.Gray;

        public Color HeaderBackgroundColor { get; set; } = GRID_VERY_LIGHT_GRAY;

        public Color HeaderSelectedBackgroundColor { get; set; } = GRID_LIGHT_GREEN;

        public Color HeaderActiveBackgroundColor { get; set; } = GRID_LIGHT_GRAY;

        public Color HeaderTextColor { get; set; } = Colors.Black;

        public Color CellBorderColor { get; set; } = Colors.Black;

        public Color CellBackgroundColor { get; set; } = Colors.White;

        public Color HighlightCellBackgroundColor { get; set; } = GRID_LIGHT_GREEN;

        public Color ActiveCellBackgroundColor { get; set; } = Colors.Yellow;

        public Color AnchorCellBackgroundColor { get; set; } = Colors.Yellow;

        public Color SelectionFrameBorderColor { get; set; } = Colors.DarkGreen;

        public double SelectionFrameBorderWidth { get; set; } = DEFAULT_SELECTION_FRAME_BORDER_WIDTH;

        public double SelectionFrameBorderGripDiameter { get; set; } = DEFAULT_SELECTION_FRAME_BORDER_GRIP_DIAMETER;

        public bool IsExtendedSelectionMode { get; set; } = false;

        public bool IsPanSupportEnabled { get; set; } = DEFAULT_IS_PAN_SUPPORT_ENABLED;

        //private CommunityToolkit.Maui.Behaviors.TouchBehavior longPressBehaviour;

        public bool AllColumnsSelected
        {
            get
            {
                return (selectedCells != null && selectedCells.Left == 1 && selectedCells.Right == ColumnCount) ||
                    (ColumnCount == 1 && activeCellPoint.Column == 1);
            }
        }

        public bool AllRowsSelected
        {
            get
            {
                return (selectedCells != null && selectedCells.Top == 1 && selectedCells.Bottom == RowCount) ||
                    (RowCount == 1 && activeCellPoint.Row == 1);
            }
        }

        public CellFrame? ActiveCell { get => activeCell; }

        public CellRange? CurrentSelection { get => selectedCells != null ? selectedCells.Clone() : new CellRange(activeCellPoint); }

        public Grid()
        {
            InitializePlatformSpecificCode();
            AddPinchGesture();
        }

        public static readonly BindableProperty ContainerScrollViewProperty =
            BindableProperty.Create(nameof(ContainerScrollView), typeof(ScrollView), typeof(Grid));

        public ScrollView ContainerScrollView
        {
            get => (ScrollView)GetValue(ContainerScrollViewProperty);
            set => SetValue(ContainerScrollViewProperty, value);
        }

        public static readonly BindableProperty ContainerContentViewProperty =
            BindableProperty.Create(nameof(ContainerContentView), typeof(ContentView), typeof(Grid));

        public ContentView ContainerContentView
        {
            get => (ContentView)GetValue(ContainerContentViewProperty);
            set => SetValue(ContainerContentViewProperty, value);
        }

        partial void InitializePlatformSpecificCode();  // Platform-specific method stub

        private void AddPinchGesture()
        {
            var pinchGesture = new PinchGestureRecognizer();
            pinchGesture.PinchUpdated += OnPinchUpdated;
            GestureRecognizers.Add(pinchGesture);
        }

        private double currentScale = 1;
        private double startScale = 1;
        private bool isPinching = false;

        private void OnPinchUpdated(object? sender, PinchGestureUpdatedEventArgs e)
        {
            if (sender == null) return;

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

        public RowDefinition NewRowDefinition(bool columnHeader)
        {
            return new RowDefinition { Height = new GridLength(columnHeader ? this.ColumnHeaderHeight : this.CellHeight) };
        }

        public ColumnDefinition NewColumnDefinition(bool rowHeader)
        {
            return new ColumnDefinition { Width = new GridLength(rowHeader ? this.RowHeaderWidth : this.CellWidth) };
        }

        public void InitializeContent()
        {
            this.IsVisible = false;
            this.RowDefinitions.Add(NewRowDefinition(true));

            for (int row = 0; row < RowCount; row++)
                this.RowDefinitions.Add(NewRowDefinition(false));

            this.ColumnDefinitions.Add(NewColumnDefinition(true));

            for (int column = 0; column < ColumnCount; column++)
                this.ColumnDefinitions.Add(NewColumnDefinition(false));

            AddHeaderRow();

            for (int row = 0; row < RowCount; row++)
                AddRowContent(row);

            IsVisible = true;

            InitializeSelectionFrame();
        }

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

        public CellFrame NewCellFrame(int row, int column)
        {
            CellFrame frame = new CellFrame(row, column)
            {
                BackgroundColor = this.CellBackgroundColor,
                Content = CreateCellContent(row, column),
                Padding = CellPadding,
                Margin = CellMargin == 0.0 ? -0.5 : CellMargin,
                Stroke = new SolidColorBrush(CellBorderColor),
                StrokeThickness = 1,
                StrokeShape = new RoundRectangle
                {
                    CornerRadius = 0
                },
            };
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

        // (0,0) = (1,1) in display terms
        virtual public ContentView CreateCellContent(int row, int column)
        {
            return new DefaultCellContent();
        }

        public CellFrame? SetCellContent(int row, int column, ContentView contentView)
        {
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame != null)
                SetCellContent(cellFrame, contentView);
            return cellFrame;
        }

        public void SetCellContent(CellFrame? cellFrame, ContentView contentView)
        {
            if (cellFrame != null)
                cellFrame.Content = contentView;
        }

        private void AddSingleTapGesture(CellFrame cellFrame)
        {
            var tapGesture = new TapGestureRecognizer
            {
                NumberOfTapsRequired = 1
            };
            tapGesture.Tapped += (s, e) => OnCellTapped(cellFrame, true);
            cellFrame.GestureRecognizers.Add(tapGesture);
        }

        private void AddDoubleTapGesture(CellFrame cellFrame)
        {
            var tapGesture = new TapGestureRecognizer
            {
                NumberOfTapsRequired = 2
            };
            tapGesture.Tapped += (s, e) => OnCellDoubleTapped(cellFrame, true);
            cellFrame.GestureRecognizers.Add(tapGesture);
        }

        private void AddHeaderRow()
        {
            for (int column = 0; column < ColumnCount; column++)
                AddColumnHeader(column);
            AddCornerHeader();
        }

        private void AddRowContent(int row)
        {
            AddRowHeader(row);

            for (int column = 0; column < ColumnCount; column++)
            {
                AddRowCell(row, column);
            }
        }

        private void AddColumnContent(int column)
        {
            AddColumnHeader(column);

            for (int row = 0; row < RowCount; row++)
            {
                AddRowCell(row, column);
            }
        }


        private CellFrame AddRowCell(int row, int column)
        {
            CellFrame cellFrame = NewCellFrame(row, column);

            this.Add(cellFrame, cellFrame.DisplayColumn, cellFrame.DisplayRow);

            return cellFrame;
        }

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

        private Thickness GetHeaderMargin(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return new Thickness(RowHeaderMargin, ColumnHeaderMargin);
                case HeaderType.Column:
                    return new Thickness(ColumnHeaderMargin);
                case HeaderType.Row:
                    return new Thickness(RowHeaderMargin);
            }
            return 0.0;
        }

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

        private void OnColumnHeaderTapped(HeaderFrame headerFrame)
        {
            SelectColumn(IsExtendedSelectionMode || IsShiftKeyPressed(), headerFrame.DisplayIndex);
        }

        /*
        private void OnColumnHeaderLongPressed(Frame frame, int column)
        {
            Debug.WriteLine($"Column header - long pressed (column = ${column})");
        }
        */

        private void OnRowHeaderTapped(HeaderFrame headerFrame)
        {
            SelectRow(IsExtendedSelectionMode || IsShiftKeyPressed(), headerFrame.DisplayIndex);
        }

        /*
        private void OnRowHeaderLongPressed(Frame frame, int row)
        {
            Debug.WriteLine($"Row header - long pressed (row = ${row})");
        }
        */

        public virtual void OnCellTapped(CellFrame cellFrame, bool triggerEvents)
        {
            ActivateCell(cellFrame, IsExtendedSelectionMode || IsShiftKeyPressed());
        }

        public virtual void OnCellDoubleTapped(CellFrame cellFrame, bool triggerEvents)
        {
            ActivateCell(cellFrame, IsExtendedSelectionMode || IsShiftKeyPressed());
        }

        public virtual void OnProcessKeyDown(Keyboard.KeyState state, Keyboard.Key key, bool triggerEvents)
        {
            bool shiftPressed = Keyboard.Utiility.IsStateFlagSet(state, Keyboard.KeyState.Shift);
            bool ctrlPressed = Keyboard.Utiility.IsStateFlagSet(state, Keyboard.KeyState.Ctrl);
            bool capsLockPressed = Keyboard.Utiility.IsStateFlagSet(state, Keyboard.KeyState.CapsLock);

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
                    if (anchorCell == null)
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

        public bool ActivateCell(CellPoint point, bool maintainSelection)
        {
            return ActivateCell(point.Row, point.Column, maintainSelection);
        }

        // (1, 1) = top-left cell
        public bool ActivateCell(int displayRow, int displayColumn, bool maintainSelection)
        {
            bool activated = false;
            if (displayColumn > 0 && displayColumn <= RowCount && displayColumn > 0 && displayColumn <= ColumnCount)
            {
                CellFrame? cellFrame = GetCell(displayRow, displayColumn) as CellFrame;
                if (cellFrame != null)
                {
                    ActivateCell(cellFrame, maintainSelection);
                    activated = true;
                }
            }
            return activated;
        }

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

        private void SelectCorner()
        {
            ClearSelectedCells();
            MoveActiveCell(false, 1, 1, true);
            MoveActiveCell(true, RowCount, ColumnCount, false);
        }

        private void SelectRow(bool maintainSelection, int row)
        {
            int displayRow = row;
            if (!maintainSelection || anchorCell == null)
            {
                bool hadAnchorCell = anchorCell != null;
                CellPoint activePoint = activeCellPoint.Clone();
                ClearSelectedCells();
                MoveActiveCell(false, maintainSelection ? activePoint.Row : displayRow, 1, true);
                MoveActiveCell(true, displayRow, ColumnCount, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Column);
            }
            else if (selectedCells != null)
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

        private void SelectColumn(bool maintainSelection, int column)
        {
            int displayCol = column;
            if (!maintainSelection || anchorCell == null)
            {
                bool hadAnchorCell = anchorCell != null;
                CellPoint activePoint = activeCellPoint.Clone();
                ClearSelectedCells();
                MoveActiveCell(false, 1, maintainSelection ? activePoint.Column : displayCol, true);
                MoveActiveCell(true, RowCount, displayCol, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Column);
            }
            else if (selectedCells != null)
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

        public void EnableExtendedSelection()
        {
            if (IsExtendedSelectionMode) return;
            if (anchorCell == null)
                SetAnchorCellToActiveCell(true);
            IsExtendedSelectionMode = true;
            selectionFrame?.EnableDashAnimation(true);
        }

        public void CancelExtendedSelection()
        {
            if (!IsExtendedSelectionMode) return;
            if (anchorCell != null)
                SetActiveCellToAnchorCell(false);
            ClearAnchorCell(true);
            ClearSelectedCells();
            if (activeCell != null)
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
            IsExtendedSelectionMode = false;
            selectionFrame?.EnableDashAnimation(false);
            UpdateSelectionFrame(true);
        }

        public void ResetSelection(CellRange newSelection)
        {
            CellRange? prevSelection = selectedCells?.Clone();

            ClearSelectedCells();
            SelectCells(Math.Clamp(newSelection.Top, 1, RowCount),
                        Math.Clamp(newSelection.Left, 1, ColumnCount),
                        Math.Clamp(newSelection.Bottom, 1, RowCount),
                        Math.Clamp(newSelection.Right, 1, ColumnCount),
                        false);

            if (selectedCells == null) return;

            if (anchorCell == null && activeCell != null)
            {
                // Initialize anchor cell
                SetAnchorCellToActiveCell(true);
            }

            if (anchorCell != null && !selectedCells.ContainsPoint(anchorCellPoint))
            {
                // Move anchor cell
                int newRow = Math.Clamp(anchorCellPoint.Row, selectedCells.Top, selectedCells.Bottom);
                int newColumn = Math.Clamp(anchorCellPoint.Column, selectedCells.Left, selectedCells.Right);
                CellFrame prevAnchorCell = anchorCell;
                MoveAnchorCell(newRow, newColumn);
                prevAnchorCell.BackgroundColor = this.CellBackgroundColor;
            }

            if (prevSelection != null)
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

        public SelectionState GetSelectionState()
        {
            return new SelectionState(
                selectedCells?.Clone(),
                activeCell != null ? activeCellPoint.Clone() : null,
                anchorCell != null ? anchorCellPoint.Clone() : null);
        }

        public void RestoreSelectionState(SelectionState selectionState, bool retainAnchorCell, bool show)
        {
            if (retainAnchorCell)
            {
                if (selectionState.AnchorCellPoint != null)
                {
                    SetAnchorCell(selectionState.AnchorCellPoint);
                    SetActiveCellToAnchorCell(true);
                }
            }
            else if (selectionState.ActiveCellPoint != null)
            {
                ActivateCell(selectionState.ActiveCellPoint, false);
            }

            if (selectionState.SelectedCells != null)
            {
                UpdateSelectedCells(new CellPoint(selectionState.SelectedCells.Top, selectionState.SelectedCells.Left),
                                    new CellPoint(selectionState.SelectedCells.Bottom, selectionState.SelectedCells.Right),
                                    true, false);
            }
            ShowSelectionFrame(show);
        }


        private void SelectCells(int top, int left, int bottom, int right, bool clear)
        {
            selectedCells = new CellRange(top, left, bottom, right);
            HighlightCells(selectedCells, clear);
            UpdateSelectionFrame(true);
        }

        private void MoveActiveCellLeft(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int colOffset = moveToEnd ? (useActiveCell ? -activeCellPoint.Column : -anchorCellPoint.Column) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }

        private void MoveActiveCellRight(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int colOffset = moveToEnd ? this.ColumnCount - (useActiveCell ? activeCellPoint.Column : anchorCellPoint.Column) : 1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }

        private void MoveActiveCellUp(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToEnd ? (useActiveCell ? -activeCellPoint.Row : -anchorCellPoint.Row) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }

        private void MoveActiveCellDown(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToEnd ? this.RowCount - (useActiveCell ? activeCellPoint.Row : anchorCellPoint.Row) : 1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }

        private void MoveActiveCellToRowStart(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToTop ? (useActiveCell ? -activeCellPoint.Row : -anchorCellPoint.Row) + 1 : 0;
            int colOffset = useActiveCell ? -activeCellPoint.Column : -anchorCellPoint.Column;
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellToColumnEnd(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToTop ? this.RowCount - (useActiveCell ? activeCellPoint.Row : anchorCellPoint.Row) : 0;
            int colOffset = this.ColumnCount - (useActiveCell ? activeCellPoint.Column : anchorCellPoint.Column);
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellOffset(bool maintainSelection, int deltaX, int deltaY)
        {
            int referenceRow = !maintainSelection && (anchorCell != null) ? anchorCellPoint.Row : activeCellPoint.Row;
            int referenceCol = !maintainSelection && (anchorCell != null) ? anchorCellPoint.Column : activeCellPoint.Column;
            int newRow = Math.Clamp(referenceRow + deltaY, 1, this.RowDefinitions.Count);
            int newCol = Math.Clamp(referenceCol + deltaX, 1, this.ColumnDefinitions.Count);

            if (maintainSelection && AllRowsSelected && deltaX != 0 && deltaY == 0)
                SelectColumn(true, newCol);
            else if (maintainSelection && AllColumnsSelected && deltaX == 0 && deltaY != 0)
                SelectRow(true, newRow);
            else
                MoveActiveCell(maintainSelection, newRow, newCol, true);
        }

        private void MoveAnchorCellToPrevWithinSelection()
        {
            if (anchorCell == null || selectedCells == null) return;
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

        private void MoveAnchorCellToNextWithinSelection()
        {
            if (anchorCell == null || selectedCells == null) return;
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

        private void MoveAnchorCell(int newRow, int newColumn)
        {
            if (anchorCell == null) return;
            anchorCell.BackgroundColor = this.HighlightCellBackgroundColor;
            ClearAnchorCell(false);
            SetAnchorCell(newRow, newColumn);
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }

        private void MoveActiveCell(bool maintainSelection, int newRow, int newColumn, bool scrollActiveCellIntoView)
        {
            bool wasExtendedSelection = false;
            if (!maintainSelection && anchorCell != null)
            {
                // Clear anchor cell
                anchorCell.BackgroundColor = this.CellBackgroundColor;
                SetActiveCellToAnchorCell(false);
                ClearAnchorCell(true);
                ClearSelectedCells();
                if (activeCell != null)
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

            if (newActiveCell != null && newActiveCell is CellFrame)
            {
                // Scroll the new active cell into view and/or update selection state as needed
                UpdateSelection(newActiveCell as CellFrame, newRow, newColumn, maintainSelection, scrollActiveCellIntoView);
            }
        }

        private void UpdateSelection(CellFrame newActiveCell, int row, int column, bool maintainSelection, bool scrollActiveCellIntoView)
        {
            // Reset the previously active cell if needed
            if (activeCell != null)
                ClearActiveCellFormatting();

            if (maintainSelection)
            {
                if (anchorCell == null)
                {
                    if (activeCell == null)
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
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.AnchorCellBackgroundColor;
            else
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;

            activeCellPoint.Set(row, column);

            if (anchorCell != null)
                UpdateSelectedCells(anchorCellPoint, activeCellPoint, true, true);
            else
            {
                HighlightActiveCellHeaders(false);
                UpdateSelectionFrame(true);
            }

            if (scrollActiveCellIntoView)
                ScrollCellIntoView(newActiveCell);
        }

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
        private static bool IsShiftKeyPressed()
        {
            return false;
        }
#endif
        private void SetAnchorCell(int row, int column)
        {
            anchorCell = GetCell(row, column) as CellFrame;
            if (anchorCell != null)
                anchorCellPoint.Set(row, column);
            else
                anchorCellPoint.Clear();
        }

        private void SetAnchorCell(CellPoint point)
        {
            SetAnchorCell(point.Row, point.Column);
        }

        void ClearAnchorCellFormatting()
        {
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.CellBackgroundColor;
        }

        private void ClearAnchorCell(bool clearFormatting)
        {
            if (clearFormatting)
                ClearAnchorCellFormatting();
            anchorCell = null;
            anchorCellPoint.Clear();
        }

        private void SetAnchorCellToActiveCell(bool setBackgroundColor)
        {
            anchorCell = activeCell;
            anchorCellPoint = activeCellPoint.Clone();
            if (anchorCell != null && setBackgroundColor)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }

        private void SetActiveCellToAnchorCell(bool setBackgroundColor)
        {
            activeCell = anchorCell;
            activeCellPoint = anchorCellPoint.Clone();
            if (activeCell != null && setBackgroundColor)
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
        }

        private void ClearSelectedCells()
        {
            if (selectedCells != null)
            {
                HighlightCells(selectedCells, true);
                ShowSelectionFrame(false);
                selectedCells = null;
            }
        }
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

        private void RemoveSelectionFrame()
        {
            selectionFrame?.RemoveFromGrid();
            selectionFrame = null;
        }

        private void ReinitializeSelectionFrame()
        {
            RemoveSelectionFrame();
            InitializeSelectionFrame();
        }


        // Return true if selection changed
        private bool UpdateSelectionFrame(bool triggerEvents)
        {
            if (selectionFrame == null)
                return false;

            CellRange? prevSelection = selectionFrame.CellRange?.Clone();
            selectionFrame.SetRange(selectedCells != null ? selectedCells : new CellRange(activeCellPoint.Row, activeCellPoint.Column), true);
            CellRange? newSelection = selectionFrame.CellRange?.Clone();
            bool selectionChange = (prevSelection == null) || !prevSelection.Equals(newSelection);

            if (selectionChange && triggerEvents)
                OnSelectionChanged();

            return selectionChange;
        }

        public virtual void OnSelectionChanged()
        {
        }

        private void ShowSelectionFrame(bool show)
        {
            if (selectionFrame == null) return;
            selectionFrame.Show(show);
        }

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
                        if (cellFrame != null)
                            cellFrame.BackgroundColor = clear ? this.CellBackgroundColor : this.HighlightCellBackgroundColor;
                    }
                }
            }
        }

        private void ClearActiveCellFormatting()
        {
            if (activeCell == null) return;
            activeCell.BackgroundColor = this.CellBackgroundColor;
            HighlightActiveCellHeaders(true);
        }

        private void ClearActiveCell(bool clearFormatting)
        {
            if (activeCell != null)
            {
                if (clearFormatting)
                    ClearActiveCellFormatting();
                activeCellPoint.Clear();
                activeCell = null;
            }
        }

        private void HighlightActiveCellHeaders(bool clear)
        {
            if (activeCell == null) return;
            HighlightHeaders(new CellRange(activeCellPoint), clear);
        }

        private void HighlightHeaders(CellRange range, bool clear)
        {
            HighlightRowHeaders(range, clear);
            HighlightColHeaders(range, clear);
        }

        private void HighlightRowHeaders(CellRange range, bool clear)
        {
            bool allColumnsSelected = range.Left == 1 && range.Right == ColumnCount;
            for (int row = range.Top; row <= range.Bottom; row++)
                HighlightRowHeader(row, clear, allColumnsSelected);
        }

        private void HighlightRowHeader(int row, bool clear, bool allColumnsSelected)
        {
            HeaderFrame? header = GetRowHeaderCell(row);
            if (header != null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allColumnsSelected ? this.HeaderSelectedBackgroundColor : this.HeaderActiveBackgroundColor);
        }

        private void HighlightColHeaders(CellRange range, bool clear)
        {
            bool allRowsSelected = range.Top == 1 && range.Bottom == RowCount;
            for (int column = range.Left; column <= range.Right; column++)
                HighlightColHeader(column, clear, allRowsSelected);
        }

        private void HighlightColHeader(int column, bool clear, bool allRowsSelected)
        {
            HeaderFrame? header = GetColHeaderCell(column);
            if (header != null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allRowsSelected ? this.HeaderSelectedBackgroundColor : HeaderActiveBackgroundColor);
        }

        private HeaderFrame? GetRowHeaderCell(int row)
        {
            return GetCell(row, 0) as HeaderFrame;
        }

        private HeaderFrame? GetColHeaderCell(int column)
        {
            return GetCell(0, column) as HeaderFrame;
        }

        public double GetCellsWidth(CellRange range)
        {
            if (range == null) return 0.0;
            double width = 0.0;
            for (int column = range.Left; column <= range.Right && column <= ColumnCount; column++)
                width += GetColumnWidth(column);
            return width;
        }

        public double GetColumnWidth(int column)
        {
            if (column < 0 || column > ColumnCount) return 0.0;
            return ColumnDefinitions[column].Width.Value;
        }

        public double GetCellsHeight(CellRange range)
        {
            if (range == null) return 0.0;
            double height = 0.0;
            for (int row = range.Top; row <= range.Bottom && row <= RowCount; row++)
                height += GetRowHeight(row);
            return height;
        }

        public double GetRowHeight(int row)
        {
            if (row < 0 || row > RowCount) return 0.0;
            return RowDefinitions[row].Height.Value;
        }

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

        private bool IsSelectionFrameObject(IView child)
        {
            return (child is BorderGrip || child is BorderBox);
        }
        private bool IsGridCellOrHeader(IView child)
        {
            return !IsSelectionFrameObject(child);
        }

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

        public bool DeleteSelectedRows()
        {
            bool deleted = false;

            if (AllColumnsSelected && RowCount > 1)
            {
                if(selectedCells != null)
                    deleted = DeleteRows(selectedCells.Top, selectedCells.Bottom);
                else if (activeCell != null)
                    deleted = DeleteRows(activeCellPoint.Row, activeCellPoint.Row);
            }

            return deleted;
        }

        public bool InsertSelectedRows()
        {
            bool inserted = false;

            if(AllColumnsSelected)
            {
                if (selectedCells != null)
                    inserted = InsertRows(selectedCells.Top, selectedCells.Bottom);
                else if (activeCell != null)
                    inserted = InsertRows(activeCellPoint.Row, activeCellPoint.Row);
            }

            return inserted;
        }

        public bool DeleteSelectedColumns()
        {
            bool deleted = false;

            if (AllRowsSelected && ColumnCount > 1)
            {
                if (selectedCells != null)
                    deleted = DeleteColumns(selectedCells.Left, selectedCells.Right);
                else if (activeCell != null)
                    deleted = DeleteColumns(activeCellPoint.Column, activeCellPoint.Column);
            }

            return deleted;
        }

        public bool InsertSelectedColumns()
        {
            bool inserted = false;

            if (AllRowsSelected )
            {
                if (selectedCells != null)
                    inserted = InsertColumns(selectedCells.Left, selectedCells.Right);
                else if (activeCell != null)
                    inserted = InsertColumns(activeCellPoint.Column, activeCellPoint.Column);
            }

            return inserted;
        }

        public bool DeleteRows(int startDisplayRow, int endDisplayRow)
        {
            bool deleted = false;

            if (IsValidDisplayRow(startDisplayRow) && IsValidDisplayRow(endDisplayRow))
                deleted = Remove(Target.Row, startDisplayRow, endDisplayRow, true);

            return deleted;
        }

        public bool InsertRows(int startDisplayRow, int endDisplayRow)
        {
            bool inserted = false;

            if (IsValidDisplayRow(startDisplayRow) && IsValidDisplayRow(endDisplayRow))
                inserted = Insert(Target.Row, startDisplayRow, endDisplayRow, true);

            return inserted;
        }

        public bool DeleteColumns(int startDisplayColumn, int endDisplayColumn)
        {
            bool deleted = false;

            if (IsValidDisplayColumn(startDisplayColumn) && IsValidDisplayColumn(endDisplayColumn))
                deleted = Remove(Target.Column, startDisplayColumn, endDisplayColumn, true);

            return deleted;
        }

        public bool InsertColumns(int startDisplayColumn, int endDisplayColumn)
        {
            bool inserted = false;

            if (IsValidDisplayColumn(startDisplayColumn) && IsValidDisplayColumn(endDisplayColumn))
                inserted = Insert(Target.Column, startDisplayColumn, endDisplayColumn, true);

            return inserted;
        }

        private enum Target
        {
            Row = 1,
            Column = 2
        }

        private bool Remove(Target target, int startPosition, int endPosition, bool triggerEvents)
        {
            bool removed = false;
            bool retainAnchorCell = anchorCell != null;
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

        private bool Insert(Target target, int startPosition, int endPosition, bool triggerEvents)
        {
            bool inserted = false;
            bool retainAnchorCell = anchorCell != null;
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

        private void InsertChildContentAt(Target target, int position)
        {
            switch (target)
            {
                case Target.Row:
                    AddRowContent(position - 1);
                    break;
                case Target.Column:
                    AddColumnContent(position - 1);
                    break;
            }
        }

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

        private void SetChildColumn(IView child, int newColumn)
        {
            int row = GetRow(child);
            SetColumn(child, newColumn);
            if (row == 0)
                UpdateHeaderIndex(HeaderType.Column, child, newColumn - 1);
            else
                UpdateCellIndex(Target.Column, child, newColumn - 1);
        }

        private void SetChildRow(IView child, int newRow)
        {
            int column = GetColumn(child);
            SetRow(child, newRow);
            if (column == 0)
                UpdateHeaderIndex(HeaderType.Row, child, newRow - 1);
            else
                UpdateCellIndex(Target.Row, child, newRow - 1);
        }

        private void UpdateCellIndex(Target target, IView child, int newIndex)
        {
            CellFrame? cellFrame = child as CellFrame;
            if (cellFrame != null)
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

        private void UpdateHeaderIndex(HeaderType expectedType, IView child, int newIndex)
        {
            HeaderFrame? headerFrame = child as HeaderFrame;
            if (headerFrame != null && headerFrame.Type == expectedType)
            {
                headerFrame.Index = newIndex;
                headerFrame.Content = GetHeaderCellContent(headerFrame.Type, headerFrame.Index);
            }
        }

        private void AdjustTargetCount(Target target, int amount)
        {
            switch (target)
            {
                case Target.Column:
                    ColumnCount = ColumnCount + amount;
                    break;
                case Target.Row:
                    RowCount = RowCount + amount;
                    break;
            }
        }

        public bool IsValidDisplayRow(int displayRow)
        {
            return displayRow >= 0 && displayRow <= RowCount;
        }

        public bool IsValidDisplayColumn(int displayColumn)
        {
            return displayColumn >= 0 && displayColumn <= ColumnCount;
        }

    }
}
