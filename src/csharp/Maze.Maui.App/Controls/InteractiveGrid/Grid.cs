using MauiGestures;
using System.Diagnostics;

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

        const double DEFAULT_COL_HEADER_HEIGHT = 75.0;
        const double DEFAULT_COL_HEADER_MARGIN = 5.0;
        const double DEFAULT_COL_HEADER_PADDING = 0.0;

        const double DEFAULT_ROW_HEADER_WIDTH = 150.0;
        const double DEFAULT_ROW_HEADER_MARGIN = 5.0;
        const double DEFAULT_ROW_HEADER_PADDING = 0.0;

        const double DEFAULT_CELL_HEIGHT = 50.0;
        const double DEFAULT_CELL_WIDTH = 50.0;
        const double DEFAULT_CELL_MARGIN = 5.0;
        const double DEFAULT_CELL_PADDING = 0.0;

        public enum HeaderType
        {
            Corner = 0,
            Row = 1,
            Column = 2
        }

        static private Color GRID_LIGHT_GREEN = Color.FromRgb(211, 240, 224);
        static private Color GRID_VERY_LIGHT_GRAY = Color.FromRgb(240, 240, 240);
        static private Color GRID_LIGHT_GRAY = Color.FromRgb(225, 225, 225);

        public int RowCount { get; set; } = 0;

        public int ColCount { get; set; } = 0;

        public double ColHeaderHeight { get; set; } = DEFAULT_COL_HEADER_HEIGHT;

        public double ColHeaderMargin { get; set; } = DEFAULT_COL_HEADER_MARGIN;

        public double ColHeaderPadding { get; set; } = DEFAULT_COL_HEADER_PADDING;

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

        public Color SelectionFrameBorderColor { get; set; } = Colors.HotPink;

        private bool inExtendedSelectionMode = false;
        //private CommunityToolkit.Maui.Behaviors.TouchBehavior longPressBehaviour;

        public bool AllColumnsSelected
        {
            get
            {
                return (selectedCells != null && selectedCells.Left == 1 && selectedCells.Right == ColCount) ||
                    (ColCount == 1 && activeCellPoint.Col == 1);
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
            this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(this.ColHeaderHeight) });

            for (int row = 0; row < RowCount; row++)
                this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(this.CellHeight) });

            this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(this.RowHeaderWidth) });

            for (int col = 0; col < ColCount; col++)
                this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(this.CellWidth) });

            for (int row = 0; row < RowCount; row++)
            {
                if (row == 0) AddHeaderRow();

                AddRowHeader(row);

                for (int col = 0; col < ColCount; col++)
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

                    Gesture.SetLongPressPointCommand(cellFrame, new Command<PointEventArgs>(args =>
                     {
                         OnCellLongPressed(cellFrame, currentRow, currentCol);
                     }));

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
        virtual public View GetCellContent(int row, int col)
        {
            return new Label
            {
                Text = ""
            };
        }

        private void AddHeaderRow()
        {
            for (int col = 0; col < ColCount; col++)
                AddColumnHeader(col);
            AddCornerHeader();
        }

        private void AddCornerHeader()
        {
            Frame frame = NewHeaderFrame(HeaderType.Corner, 0);
            var tapGesture = new TapGestureRecognizer();
            tapGesture.Tapped += (s, e) => OnCornerHeaderTapped();
            frame.GestureRecognizers.Add(tapGesture);
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnCornerHeaderLongPressed(frame);
            }));
            this.Add(frame, 0, 0);
        }

        private void AddColumnHeader(int col)
        {
            Frame frame = NewHeaderFrame(HeaderType.Column, col);
            var tapGesture = new TapGestureRecognizer();
            int currentCol = col + 1;
            tapGesture.Tapped += (s, e) => OnColumnHeaderTapped(currentCol);
            frame.GestureRecognizers.Add(tapGesture);
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnColumnHeaderLongPressed(frame, currentCol);
            }));

            this.Add(frame, col + 1, 0);
        }

        private void AddRowHeader(int row)
        {
            Frame frame = NewHeaderFrame(HeaderType.Row, row);
            var tapGesture = new TapGestureRecognizer();
            int currentRow = row + 1;
            tapGesture.Tapped += (s, e) => OnRowHeaderTapped(currentRow);
            frame.GestureRecognizers.Add(tapGesture);
            Gesture.SetLongPressPointCommand(frame, new Command<PointEventArgs>(args =>
            {
                OnRowHeaderLongPressed(frame, currentRow);
            }));

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
                    return new Thickness(RowHeaderMargin, ColHeaderMargin);
                case HeaderType.Column:
                    return new Thickness(ColHeaderMargin);
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
                    return new Thickness(ColHeaderPadding);
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
                    return this.ColHeaderHeight - (2 * this.ColHeaderMargin);
                case HeaderType.Row:
                    return this.CellHeight - (2 * this.CellMargin);
                case HeaderType.Column:
                    return this.ColHeaderHeight - (2 * this.ColHeaderMargin);
            }
            return 0;
        }

        private void OnCornerHeaderTapped()
        {
            SelectCorner();
        }

        private void OnCornerHeaderLongPressed(Frame frame)
        {
            inExtendedSelectionMode = !inExtendedSelectionMode;
            SelectCorner();
        }

        private void OnColumnHeaderTapped(int col)
        {
            SelectColumn(inExtendedSelectionMode || IsShiftKeyPressed(), col);
        }

        private void OnColumnHeaderLongPressed(Frame frame, int col)
        {
            inExtendedSelectionMode = !inExtendedSelectionMode;
            OnColumnHeaderTapped(col);
        }

        private void OnRowHeaderTapped(int row)
        {
            SelectRow(inExtendedSelectionMode || IsShiftKeyPressed(), row);
        }

        private void OnRowHeaderLongPressed(Frame frame, int row)
        {
            inExtendedSelectionMode = !inExtendedSelectionMode;
            OnRowHeaderTapped(row);
        }

        private void OnCellTapped(Frame cell, int row, int col)
        {
            MoveActiveCell(this.inExtendedSelectionMode || IsShiftKeyPressed(), row, col, true);
        }

        private void OnCellLongPressed(Frame cell, int row, int col)
        {
            this.inExtendedSelectionMode = !this.inExtendedSelectionMode;
            OnCellTapped(cell, row, col);
        }

        private void SelectCorner()
        {
            ClearSelectedCells();
            MoveActiveCell(false, 1, 1, true);
            MoveActiveCell(true, RowCount, ColCount, false);
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
                MoveActiveCell(true, displayRow, ColCount, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Col);
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
                SelectCells(top, 1, bottom, ColCount, false);
                activeCellPoint.Row = displayRow;
            }
        }

        private void SelectColumn(bool maintainSelection, int col)
        {
            int displayCol = col;
            if (!maintainSelection || anchorCell == null)
            {
                bool hadAnchorCell = anchorCell != null;
                CellPoint activePoint = activeCellPoint.Clone();
                ClearSelectedCells();
                MoveActiveCell(false, 1, maintainSelection ? activePoint.Col : displayCol, true);
                MoveActiveCell(true, RowCount, displayCol, false);
                if (maintainSelection && !hadAnchorCell)
                    MoveAnchorCell(activePoint.Row, activePoint.Col);
            }
            else if (selectedCells != null)
            {
                int left = selectedCells.Left,
                    right = selectedCells.Right;
                if (displayCol > anchorCellPoint.Col)
                {
                    left = anchorCellPoint.Col;
                    right = displayCol;
                }
                else if (displayCol <= anchorCellPoint.Col)
                {
                    right = anchorCellPoint.Col;
                    left = displayCol;
                }
                ClearSelectedCells();
                SelectCells(1, left, RowCount, right, false);
                activeCellPoint.Col = displayCol;
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
            int colOffset = moveToEnd ? (useActiveCell ? -activeCellPoint.Col : -anchorCellPoint.Col) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }

        private void MoveActiveCellRight(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int colOffset = moveToEnd ? this.ColCount - (useActiveCell ? activeCellPoint.Col : anchorCellPoint.Col) : 1;
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
            int colOffset = useActiveCell ? -activeCellPoint.Col : -anchorCellPoint.Col;
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellToColumnEnd(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToTop ? this.RowCount - (useActiveCell ? activeCellPoint.Row : anchorCellPoint.Row) : 0;
            int colOffset = this.ColCount - (useActiveCell ? activeCellPoint.Col : anchorCellPoint.Col);
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellOffset(bool maintainSelection, int deltaX, int deltaY)
        {
            int referenceRow = !maintainSelection && (anchorCell != null) ? anchorCellPoint.Row : activeCellPoint.Row;
            int referenceCol = !maintainSelection && (anchorCell != null) ? anchorCellPoint.Col : activeCellPoint.Col;
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
            int newCol = anchorCellPoint.Col - 1;
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
            int newCol = anchorCellPoint.Col + 1;
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

        private void MoveAnchorCell(int newRow, int newCol)
        {
            if (anchorCell == null) return;
            anchorCell.BackgroundColor = this.HighlightCellBackgroundColor;
            ClearAnchorCell();
            SetAnchorCell(newRow, newCol);
            if (anchorCell != null)
                anchorCell.BackgroundColor = this.ActiveCellBackgroundColor;

        }

        private void MoveActiveCell(bool maintainSelection, int newRow, int newCol, bool scrollActiveCellIntoView)
        {
            if (!maintainSelection && anchorCell != null)
            {
                // Clear anchor cell
                anchorCell.BackgroundColor = this.CellBackgroundColor;
                activeCell = anchorCell;
                activeCellPoint.Col = anchorCellPoint.Col;
                activeCellPoint.Row = anchorCellPoint.Row;
                ClearAnchorCell();
                ClearSelectedCells();
                activeCell.BackgroundColor = this.ActiveCellBackgroundColor;
            }
            // No change in position?
            if (newRow == activeCellPoint.Row && newCol == activeCellPoint.Col) return;

            // Find the new active cell
            var newActiveCell = this.Children
                .OfType<Frame>()
                .FirstOrDefault(cell => Microsoft.Maui.Controls.Grid.GetRow(cell) == newRow && Microsoft.Maui.Controls.Grid.GetColumn(cell) == newCol);

            if (newActiveCell != null)
            {
                // Scroll the new active cell into view and/or update selection state as needed
                UpdateSelection(newActiveCell, newRow, newCol, maintainSelection, scrollActiveCellIntoView);
            }
        }

        private void UpdateSelection(Frame newActiveCell, int row, int col, bool maintainSelection, bool scrollActiveCellIntoView)
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
                        SetAnchorCell(row, col);
                    else
                        SetAnchorCell(activeCellPoint.Row, activeCellPoint.Col);
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
            activeCellPoint.Col = col;

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
        private void SetAnchorCell(int row, int col)
        {
            anchorCell = GetCell(row, col);
            anchorCellPoint.Row = anchorCell != null ? row : -1;
            anchorCellPoint.Col = anchorCell != null ? col : -1;
        }

        private void ClearAnchorCell()
        {
            anchorCell = null;
            anchorCellPoint.Row = -1;
            anchorCellPoint.Col = -1;
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
            int startCol = Math.Min(anchorCellPoint.Col, activeCellPoint.Col);
            int width = Math.Abs(anchorCellPoint.Col - activeCellPoint.Col) + 1;
            int height = Math.Abs(anchorCellPoint.Row - activeCellPoint.Row) + 1;
            selectedCells = new CellRange(startRow, startCol, startRow + height - 1, startCol + width - 1);
            HighlightCells(selectedCells, false);
            UpdateSelectionFrame();
        }

        private void InitializeSelectionFrame()
        {
            selectionFrame = new SelectionFrame(this)
            {
                BorderColor = SelectionFrameBorderColor
            };
            selectionFrame.AddToGrid();
        }

        private void UpdateSelectionFrame()
        {
            if (selectionFrame == null) return;
            selectionFrame.SetRange(selectedCells != null ? selectedCells : new CellRange(activeCellPoint.Row, activeCellPoint.Col), true);
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
                    if (row != anchorCellPoint.Row || col != anchorCellPoint.Col)
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
            bool allColumnsSelected = range.Left == 1 && range.Right == ColCount;
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

        private Frame? GetColHeaderCell(int col)
        {
            return GetCell(0, col);
        }

        public double GetCellsWidth(CellRange range)
        {
            if (range == null) return 0.0;
            double width = 0.0;
            for (int col = range.Left; col <= range.Right && col <= ColCount; col++)
                width += GetColumnWidth(col);
            return width;
        }

        public double GetColumnWidth(int col)
        {
            if (col < 0 || col > ColCount) return 0.0;
            return ColumnDefinitions[col].Width.Value;
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

        private Frame? GetCell(int row, int col)
        {
            //            if (row == 0 || col == 0) return null;
            foreach (var child in this.Children)
            {
                if (this.GetRow(child) == row && this.GetColumn(child) == col)
                    return (Frame)child;
            }
            return null;
        }
    }
}
