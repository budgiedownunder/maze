using MauiGestures;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public partial class Grid : Microsoft.Maui.Controls.Grid
    {
        private Frame? activeCell = null;
        private CellPoint activeCellPoint = new CellPoint();
        private Frame? anchorCell = null;
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

        public enum HeaderType
        {
            Corner = 0,
            Row = 1,
            Column = 2
        }

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

        public double SelectionFrameBorderWidth { get; set; } = 2.0;

        public double SelectionFrameBorderGripSize { get; set; } = 10.0;

        public bool IsExtendedSelectionMode { get; set; } = false;

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

        public CellRange? SelectedCells { get => selectedCells; }

        public Grid()
        {
            InitializePlatformSpecificCode();
        }

        public static readonly BindableProperty ContainerScrollViewProperty =
            BindableProperty.Create(nameof(ContainerScrollView), typeof(ScrollView), typeof(Grid));

        public ScrollView ContainerScrollView
        {
            get => (ScrollView)GetValue(ContainerScrollViewProperty);
            set => SetValue(ContainerScrollViewProperty, value);
        }

        partial void InitializePlatformSpecificCode();  // Platform-specific method stub


        public void PopulateGrid()
        {
            this.IsVisible = false;
            this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(this.ColumnHeaderHeight) });

            for (int row = 0; row < RowCount; row++)
                this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(this.CellHeight) });

            this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(this.RowHeaderWidth) });

            for (int col = 0; col < ColumnCount; col++)
                this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(this.CellWidth) });

            for (int row = 0; row < RowCount; row++)
            {
                if (row == 0) AddHeaderRow();

                AddRowHeader(row);

                for (int col = 0; col < ColumnCount; col++)
                {
                    Frame cellFrame = new Frame
                    {
                        BorderColor = this.CellBorderColor,
                        BackgroundColor = this.CellBackgroundColor,
                        Content = GetCellContent(row, col),
                        Padding = CellPadding,
                        Margin = CellMargin,
                        CornerRadius = 0,
                        HasShadow = false,
                    };

                    var tapGesture = new TapGestureRecognizer();
                    int currentRow = row + 1, currentCol = col + 1;
                    tapGesture.Tapped += (s, e) => OnCellTapped(cellFrame, currentRow, currentCol);
                    cellFrame.GestureRecognizers.Add(tapGesture);

                    /*
                    Gesture.SetLongPressPointCommand(cellFrame, new Command<PointEventArgs>(args =>
                    {
                        OnCellLongPressed(cellFrame, currentRow, currentCol);
                    }));
                    */

                    this.Add(cellFrame, currentCol, currentRow);
                }
            }

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

        // (0,0) = (1,1) in display terms
        virtual public View GetCellContent(int row, int column)
        {
            return new Label
            {
                Text = ""
            };
        }

        private void AddHeaderRow()
        {
            for (int col = 0; col < ColumnCount; col++)
                AddColumnHeader(col);
            AddCornerHeader();
        }

        private void AddCornerHeader()
        {
            Frame frame = NewHeaderFrame(HeaderType.Corner, 0);
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
            Frame frame = NewHeaderFrame(HeaderType.Column, column);
            var tapGesture = new TapGestureRecognizer();
            int currentCol = column + 1;
            tapGesture.Tapped += (s, e) => OnColumnHeaderTapped(currentCol);
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
            Frame frame = NewHeaderFrame(HeaderType.Row, row);
            var tapGesture = new TapGestureRecognizer();
            int currentRow = row + 1;
            tapGesture.Tapped += (s, e) => OnRowHeaderTapped(currentRow);
            frame.GestureRecognizers.Add(tapGesture);

            /*
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnRowHeaderLongPressed(frame, currentRow);
            }));
            */

            this.Add(frame, 0, row + 1);
        }
        private Frame NewHeaderFrame(HeaderType type, int index)
        {
            var frame = new Frame
            {
                WidthRequest = GetHeaderWidth(type),
                HeightRequest = GetHeaderHeight(type),
                CornerRadius = 5,
                Padding = GetHeaderPadding(type),
                BackgroundColor = this.HeaderBackgroundColor,
                Content = GetHeaderCellContent(type, index),
                BorderColor = this.HeaderBorderColor,
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

        private void OnColumnHeaderTapped(int column)
        {
            SelectColumn(IsExtendedSelectionMode || IsShiftKeyPressed(), column);
        }

        /*
        private void OnColumnHeaderLongPressed(Frame frame, int column)
        {
            Debug.WriteLine($"Column header - long pressed (column = ${column})");
        }
        */

        private void OnRowHeaderTapped(int row)
        {
            SelectRow(IsExtendedSelectionMode || IsShiftKeyPressed(), row);
        }

        /*
        private void OnRowHeaderLongPressed(Frame frame, int row)
        {
            Debug.WriteLine($"Row header - long pressed (row = ${row})");
        }
        */

        private void OnCellTapped(Frame cell, int row, int column)
        {
            MoveActiveCell(this.IsExtendedSelectionMode || IsShiftKeyPressed(), row, column, true);
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

        public void EnableExtendedSelection(bool enable) 
        {
            if(enable != IsExtendedSelectionMode)
            {
                if(enable)
                {
                    if(anchorCell == null)
                        SetAnchorCellToActiveCell(true);
                }
                else
                {
                    if (anchorCell != null)
                        SetActiveCellToAnchorCell(false);
                    ClearAnchorCell();
                    ClearSelectedCells();
                    UpdateSelectionFrame();
                    if(activeCell != null)
                        activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
                }
                IsExtendedSelectionMode = enable;
            }
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
                Frame prevAnchorCell = anchorCell;
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

        private void SelectCells(int top, int left, int bottom, int right, bool clear)
        {
            selectedCells = new CellRange(top, left, bottom, right);
            HighlightCells(selectedCells, clear);
            UpdateSelectionFrame();
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
            ClearAnchorCell();
            SetAnchorCell(newRow, newColumn);
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;

        }

        private void MoveActiveCell(bool maintainSelection, int newRow, int newColumn, bool scrollActiveCellIntoView)
        {
            if (!maintainSelection && anchorCell != null)
            {
                // Clear anchor cell
                anchorCell.BackgroundColor = this.CellBackgroundColor;
                SetActiveCellToAnchorCell(false);
                ClearAnchorCell();
                ClearSelectedCells();
                if(activeCell != null)
                    activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
            }
            // No change in position?
            if (newRow == activeCellPoint.Row && newColumn == activeCellPoint.Column) return;

            // Find the new active cell
            var newActiveCell = this.Children
                .OfType<Frame>()
                .FirstOrDefault(cell => Microsoft.Maui.Controls.Grid.GetRow(cell) == newRow && Microsoft.Maui.Controls.Grid.GetColumn(cell) == newColumn);

            if (newActiveCell != null)
            {
                // Scroll the new active cell into view and/or update selection state as needed
                UpdateSelection(newActiveCell, newRow, newColumn, maintainSelection, scrollActiveCellIntoView);
            }
        }

        private void UpdateSelection(Frame newActiveCell, int row, int column, bool maintainSelection, bool scrollActiveCellIntoView)
        {
            // Reset the previously active cell if needed
            if (activeCell != null)
            {
                activeCell.BackgroundColor = this.CellBackgroundColor;
                HighlightActiveCellHeaders(true);
            }

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
                ClearAnchorCell();
            }

            // Set the new active cell
            activeCell = newActiveCell;
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.AnchorCellBackgroundColor;
            else
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;

            activeCellPoint.Row = row;
            activeCellPoint.Column = column;

            if (anchorCell != null)
                UpdateSelectedCells();
            else
            {
                HighlightActiveCellHeaders(false);
                UpdateSelectionFrame();
            }

            if (scrollActiveCellIntoView)
                ScrollCellIntoView(newActiveCell);
        }

        private async void ScrollCellIntoView(Frame cell)
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
            anchorCell = GetCell(row, column);
            anchorCellPoint.Row = anchorCell != null ? row : -1;
            anchorCellPoint.Column = anchorCell != null ? column : -1;
        }

        private void ClearAnchorCell()
        {
            anchorCell = null;
            anchorCellPoint.Row = -1;
            anchorCellPoint.Column = -1;
        }

        private void SetAnchorCellToActiveCell(bool setBackgroundColor)
        {
            anchorCell = activeCell;
            anchorCellPoint = activeCellPoint.Clone();
            if(anchorCell != null && setBackgroundColor)
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
        private void UpdateSelectedCells()
        {
            ClearSelectedCells();
            int startRow = Math.Min(anchorCellPoint.Row, activeCellPoint.Row);
            int startCol = Math.Min(anchorCellPoint.Column, activeCellPoint.Column);
            int width = Math.Abs(anchorCellPoint.Column - activeCellPoint.Column) + 1;
            int height = Math.Abs(anchorCellPoint.Row - activeCellPoint.Row) + 1;
            selectedCells = new CellRange(startRow, startCol, startRow + height - 1, startCol + width - 1);
            HighlightCells(selectedCells, false);
            UpdateSelectionFrame();
        }

        private void InitializeSelectionFrame()
        {
            selectionFrame = new SelectionFrame(this)
            {
                BorderColor = SelectionFrameBorderColor,
                BorderWidth = SelectionFrameBorderWidth,
                BorderGripSize = SelectionFrameBorderGripSize
            };
            selectionFrame.AddToGrid();
        }

        private void UpdateSelectionFrame()
        {
            if (selectionFrame == null) return;
            selectionFrame.SetRange(selectedCells != null ? selectedCells : new CellRange(activeCellPoint.Row, activeCellPoint.Column), true);
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
                for (int col = range.Left; col <= range.Right; col++)
                {
                    if (row != anchorCellPoint.Row || col != anchorCellPoint.Column)
                    {
                        Frame? cellFrame = GetCell(row, col);
                        if (cellFrame != null)
                            cellFrame.BackgroundColor = clear ? this.CellBackgroundColor : this.HighlightCellBackgroundColor;
                    }
                }
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
            Frame? header = GetRowHeaderCell(row);
            if (header != null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allColumnsSelected ? this.HeaderSelectedBackgroundColor : this.HeaderActiveBackgroundColor);
        }

        private void HighlightColHeaders(CellRange range, bool clear)
        {
            bool allRowsSelected = range.Top == 1 && range.Bottom == RowCount;
            for (int col = range.Left; col <= range.Right; col++)
                HighlightColHeader(col, clear, allRowsSelected);
        }

        private void HighlightColHeader(int col, bool clear, bool allRowsSelected)
        {
            Frame? header = GetColHeaderCell(col);
            if (header != null)
                header.BackgroundColor = clear ? this.HeaderBackgroundColor : (allRowsSelected ? this.HeaderSelectedBackgroundColor : HeaderActiveBackgroundColor);
        }

        private Frame? GetRowHeaderCell(int row)
        {
            return GetCell(row, 0);
        }

        private Frame? GetColHeaderCell(int column)
        {
            return GetCell(0, column);
        }

        public double GetCellsWidth(CellRange range)
        {
            if (range == null) return 0.0;
            double width = 0.0;
            for (int col = range.Left; col <= range.Right && col <= ColumnCount; col++)
                width += GetColumnWidth(col);
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

        private Frame? GetCell(int row, int column)
        {
            foreach (var child in this.Children)
            {
                if (this.GetRow(child) == row && this.GetColumn(child) == column)
                    return (Frame)child;
            }
            return null;
        }
    }
}
