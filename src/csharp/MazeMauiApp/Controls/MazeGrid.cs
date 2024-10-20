using Microsoft.Maui.Controls;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Maui.Graphics;
using Maze.Api;
using System.Runtime.CompilerServices;
using System.Reflection.Metadata.Ecma335;

namespace MazeMauiApp.Controls
{
    partial class MazeGrid : Grid
    {
        private Frame? activeCell = null;
        private int activeCellRow = 0;
        private int activeCellCol = 0;
        private Frame? anchorCell = null;
        private int anchorCellRow = 0;
        private int anchorCellCol = 0;
        private Microsoft.Maui.Graphics.Rect? currentSelectedCells;

        const double CELL_SIZE = 50.0;
        const double COL_HEADER_HEIGHT = 15.0;
        const double ROW_HEADER_WIDTH = 15.0;

        private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        private enum HeaderType
        {
            Corner = 0,
            Row = 1,
            Column = 2
        }

        public MazeGrid()
        {
            InitializePlatformSpecificCode();
            PopulateGrid();
        }

        public static readonly BindableProperty ContainerScrollViewProperty =
            BindableProperty.Create(nameof(ContainerScrollView), typeof(ScrollView), typeof(MazeGrid));

        public ScrollView ContainerScrollView
        {
            get => (ScrollView)GetValue(ContainerScrollViewProperty);
            set => SetValue(ContainerScrollViewProperty, value);
        }
        partial void InitializePlatformSpecificCode();  // Platform-specific method stub

        private int RowCount
        {
            get
            {
                return (int)maze.RowCount;
            }
        }

        private int ColCount
        {
            get
            {
                return (int)maze.ColCount;
            }
        }


        private void PopulateGrid()
        {

            this.IsVisible = false;

            this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(COL_HEADER_HEIGHT) });

            for (int row = 0; row < RowCount; row++)
            {
                this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(CELL_SIZE) });
            }


            this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(COL_HEADER_HEIGHT) });

            for (int col = 0; col < ColCount; col++)
            {
                this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(CELL_SIZE) });
            }

            // Populate the grid with Frames
            for (int row = 0; row < RowCount; row++)
            {
                if (row == 0)
                {
                    AddHeaderRow();
                }

                AddRowHeader(row);

                for (int col = 0; col < ColCount; col++)
                {
                    // Create a new Frame for each cell
                    Frame cellFrame = new Frame
                    {
                        BorderColor = Colors.Black,
                        BackgroundColor = Colors.White,
                        Content = new Label
                        {
                            Text = $"({row},{col})",
                            HorizontalOptions = LayoutOptions.Center,
                            VerticalOptions = LayoutOptions.Center
                        },
                        Padding = 0,
                        Margin = 0,
                        CornerRadius = 0,
                        HasShadow = false,
                    };

                    // Add a TapGestureRecognizer to each cell
                    var tapGesture = new TapGestureRecognizer();
                    int currentRow = row + 1, currentCol = col + 1;
                    tapGesture.Tapped += (s, e) => OnCellTapped(cellFrame, currentRow, currentCol);
                    cellFrame.GestureRecognizers.Add(tapGesture);


                    // Add the Frame to the grid at the specified row and column
                    this.Add(cellFrame, currentCol, currentRow);

                }
            }
            this.IsVisible = true;
        }
        private void AddHeaderRow()
        {
            AddCornerHeader();
            for (int col = 0; col < ColCount; col++)
            {
                AddColumnHeader(col);
            }
        }

        private void AddCornerHeader()
        {
            Button button = NewHeaderButton(HeaderType.Corner);
            var tapGesture = new TapGestureRecognizer();
            tapGesture.Tapped += (s, e) => OnCornerHeaderTapped();
            button.GestureRecognizers.Add(tapGesture);
            this.Add(button, 0, 0);
        }

        private void AddColumnHeader(int col)
        {
            Button button = NewHeaderButton(HeaderType.Column);
            var tapGesture = new TapGestureRecognizer();
            int currentCol = col;
            tapGesture.Tapped += (s, e) => OnColumnHeaderTapped(currentCol);
            button.GestureRecognizers.Add(tapGesture);
            this.Add(button, col + 1, 0);
        }

        private void AddRowHeader(int row)
        {
            Button button = NewHeaderButton(HeaderType.Row);
            var tapGesture = new TapGestureRecognizer();
            int currentRow = row;
            tapGesture.Tapped += (s, e) => OnRowHeaderTapped(currentRow);
            button.GestureRecognizers.Add(tapGesture);
            this.Add(button, 0, row + 1);
        }

        private Frame NewHeaderCell()
        {
            return new Frame
            {
                BorderColor = Colors.Black,
                BackgroundColor = Colors.LightGray,
                Padding = 0,
                Margin = 0,
                CornerRadius = 0,
                HasShadow = false,
            };
        }

        private Button NewHeaderButton(HeaderType type)
        {
            var button = new Button
            {
                WidthRequest = GetHeaderWidth(type),
                HeightRequest = GetHeaderHeight(type),
                CornerRadius = 5,
                Padding = new Thickness(5),
                BackgroundColor = Colors.LightGray,
                BorderWidth = 2,
                BorderColor = Colors.Gray,
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center,
                TextColor = Colors.Black
            };
            return button;
        }

        private double GetHeaderWidth(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return ROW_HEADER_WIDTH;
                case HeaderType.Row:
                    return ROW_HEADER_WIDTH;
                case HeaderType.Column:
                    return CELL_SIZE;
            }
            return 0;
        }

        private double GetHeaderHeight(HeaderType type)
        {
            switch (type)
            {
                case HeaderType.Corner:
                    return COL_HEADER_HEIGHT;
                case HeaderType.Row:
                    return CELL_SIZE;
                case HeaderType.Column:
                    return COL_HEADER_HEIGHT;
            }
            return 0;
        }

        private void OnCornerHeaderTapped()
        {
            SelectCorner();
        }


        private void OnColumnHeaderTapped(int col)
        {
            SelectColumn(col);
        }

        private void OnRowHeaderTapped(int row)
        {
            SelectRow(row);
        }


        private void OnCellTapped(Frame cell, int row, int col)
        {
            MoveActiveCell(IsShiftKeyPressed(), row, col);
        }

        private void SelectCorner()
        {
            ClearSelectedCells();
            MoveActiveCell(false, 1, 1);
            MoveActiveCell(true, RowCount, ColCount);
        }

        private void SelectRow(int row)
        {
            ClearSelectedCells();
            MoveActiveCell(false, row + 1, 1);
            MoveActiveCell(true, row + 1, ColCount);
        }

        private void SelectColumn(int col)
        {
            ClearSelectedCells();
            MoveActiveCell(false, 1, col + 1);
            MoveActiveCell(true, RowCount, col + 1);
        }

        private void MoveActiveCellLeft(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int colOffset = moveToEnd ? (useActiveCell ? -activeCellCol : -anchorCellCol) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }

        private void MoveActiveCellRight(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int colOffset = moveToEnd ? this.ColCount - (useActiveCell ? activeCellCol : anchorCellCol) : 1;
            MoveActiveCellOffset(maintainSelection, colOffset, 0);
        }

        private void MoveActiveCellUp(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToEnd ? (useActiveCell ? -activeCellRow : -anchorCellRow) + 1 : -1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }

        private void MoveActiveCellDown(bool maintainSelection, bool moveToEnd)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToEnd ? this.RowCount - (useActiveCell ? activeCellRow : anchorCellRow) : 1;
            MoveActiveCellOffset(maintainSelection, 0, rowOffset);
        }

        private void MoveActiveCellToRowStart(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToTop ? (useActiveCell ? -activeCellRow : -anchorCellRow) + 1 : 0;
            int colOffset = useActiveCell ? -activeCellCol : -anchorCellCol;
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellToColumnEnd(bool maintainSelection, bool moveToTop)
        {
            bool useActiveCell = maintainSelection || (anchorCell == null);
            int rowOffset = moveToTop ? this.RowCount - (useActiveCell ? activeCellRow : anchorCellRow) : 0;
            int colOffset = this.ColCount - (useActiveCell ? activeCellCol : anchorCellCol);
            MoveActiveCellOffset(maintainSelection, colOffset, rowOffset);
        }

        private void MoveActiveCellOffset(bool maintainSelection, int deltaX, int deltaY)
        {
            int referenceRow = !maintainSelection && (anchorCell != null) ? anchorCellRow : activeCellRow;
            int referenceCol = !maintainSelection && (anchorCell != null) ? anchorCellCol : activeCellCol;
            int newRow = Math.Clamp(referenceRow + deltaY, 1, this.RowDefinitions.Count);
            int newCol = Math.Clamp(referenceCol + deltaX, 1, this.ColumnDefinitions.Count);

            MoveActiveCell(maintainSelection, newRow, newCol);
        }

        private void MoveAnchorCellToPrevWithinSelection()
        {
            if (anchorCell == null || !currentSelectedCells.HasValue) return;
            Rect selection = currentSelectedCells.Value;
            int newCol = anchorCellCol - 1;
            int newRow = anchorCellRow;
            if (newCol < selection.Left)
            {
                newCol = (int)(selection.Right) - 1;
                newRow--;
            }
            if (newRow < selection.Top)
                newRow = (int)selection.Bottom - 1;
            MoveAnchorCell(newRow, newCol);
        }

        private void MoveAnchorCellToNextWithinSelection()
        {
            if (anchorCell == null || !currentSelectedCells.HasValue) return;
            Rect selection = currentSelectedCells.Value;
            int newCol = anchorCellCol + 1;
            int newRow = anchorCellRow;
            if (newCol >= selection.Right)
            {
                newCol = (int)selection.Left;
                newRow++;
            }
            if (newRow >= selection.Bottom)
                newRow = (int)selection.Top;
            MoveAnchorCell(newRow, newCol);
        }

        private void MoveAnchorCell(int newRow, int newCol)
        {
            if (anchorCell == null) return;
            anchorCell.BackgroundColor = Colors.Yellow;
            ClearAnchorCell();
            SetAnchorCell(newRow, newCol);
            if (anchorCell != null)
                anchorCell.BackgroundColor = Colors.HotPink;

        }

        private void MoveActiveCell(bool maintainSelection, int newRow, int newCol)
        {
            if (!maintainSelection && anchorCell != null)
            {
                anchorCell.BackgroundColor = Colors.White;
                activeCell = anchorCell;
                activeCellCol = anchorCellCol;
                activeCellRow = anchorCellRow;
                ClearAnchorCell();
                ClearSelectedCells();
                activeCell.BackgroundColor = Colors.HotPink;
            }
            // If the position hasn't changed, return
            if (newRow == activeCellRow && newCol == activeCellCol) return;

            // Find the new active cell
            var newActiveCell = this.Children
                .OfType<Frame>()
                .FirstOrDefault(cell => Grid.GetRow(cell) == newRow && Grid.GetColumn(cell) == newCol);

            if (newActiveCell != null)
            {
                // Scroll the new active cell into view and update its background
                UpdateSelection(newActiveCell, newRow, newCol, maintainSelection);
            }
        }

        private async void UpdateSelection(Frame newActiveCell, int row, int col, bool maintainSelection)
        {
            if (activeCell != null)
            {
                // Reset the previously active cell
                //activeCell.BorderColor = Colors.Black;
                activeCell.BackgroundColor = Colors.White;
            }

            if (maintainSelection)
            {
                if (anchorCell == null)
                {
                    if (activeCell == null)
                    {
                        SetAnchorCell(row, col);
                    }
                    else
                        SetAnchorCell(activeCellRow, activeCellCol);
                }
            }
            else
            {
                ClearSelectedCells();
                ClearAnchorCell();
            }

            // Set the new active cell
            activeCell = newActiveCell;
            //activeCell.Padding = 20;
            //activeCell.BorderColor = Colors.HotPink;
            if (anchorCell != null)
                anchorCell.BackgroundColor = Colors.HotPink;
            else
                activeCell.BackgroundColor = Colors.HotPink;

            activeCellRow = row;
            activeCellCol = col;

            if (anchorCell != null)
            {
                UpdateSelectedCells();
            }


            // Handle scroll
            double cellWidth = newActiveCell.Bounds.Width;
            double cellLeftX = newActiveCell.Bounds.X;
            double cellRightX = cellLeftX + cellWidth - 1;
            double cellHeight = newActiveCell.Bounds.Height;
            double cellTopY = newActiveCell.Bounds.Y;
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
            {
                targetX = cellLeftX;
            }
            else if (cellRightX > scrollMaxVisibleX)
            {
                targetX = cellRightX - scrollViewWidth;
            }
            else
                targetX = currentScrollX;


            if (cellTopY < currentScrollY)
            {
                targetY = cellTopY;
            }
            else if (cellBottomY > (currentScrollY + scrollViewHeight))
            {
                targetY = cellBottomY - scrollViewHeight;
            }
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
            anchorCellRow = anchorCell != null ? row : -1;
            anchorCellCol = anchorCell != null ? col : -1;
        }

        private void ClearAnchorCell()
        {
            anchorCell = null;
            anchorCellRow = -1;
            anchorCellCol = -1;
        }

        private void ClearSelectedCells()
        {
            if (currentSelectedCells != null)
            {
                HighlightCells(currentSelectedCells.Value, true);
                currentSelectedCells = null;
            }

        }
        private void UpdateSelectedCells()
        {
            ClearSelectedCells();
            int startRow = Math.Min(anchorCellRow, activeCellRow);
            int startCol = Math.Min(anchorCellCol, activeCellCol);
            int width = Math.Abs(anchorCellCol - activeCellCol) + 1;
            int height = Math.Abs(anchorCellRow - activeCellRow) + 1;
            currentSelectedCells = new Rect(startCol, startRow, width, height);
            HighlightCells(currentSelectedCells.Value, false);
        }
        private void HighlightCells(Microsoft.Maui.Graphics.Rect region, bool clear)
        {
            for (int row = (int)region.Top; row < (int)region.Bottom; row++)
            {
                for (int col = (int)region.Left; col < (int)region.Right; col++)
                {
                    if (row != anchorCellRow || col != anchorCellCol)
                    {
                        Frame? cell = GetCell(row, col);
                        if (cell != null)
                        {
                            cell.BackgroundColor = clear ? Colors.White : Colors.Yellow;
                        }
                    }
                }
            }
        }
        private Frame? GetCell(int row, int col)
        {
            if (row == 0 || col == 0) return null;
            foreach (var child in this.Children)
            {
                if (this.GetRow(child) == row && this.GetColumn(child) == col)
                {
                    return (Frame)child;
                }
            }
            return null;  // Return null if no element found
        }
    }
}
