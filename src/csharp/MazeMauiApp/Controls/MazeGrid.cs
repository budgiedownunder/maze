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
        private Frame? activeCell = null;  // Keeps track of the active cell
        private int activeCellRow = 0;
        private int activeCellCol = 0;
        private bool haveAnchorCell = false;
        private int anchorCellRow = 0;
        private int anchorCellCol = 0;
        private Microsoft.Maui.Graphics.Rect? currentSelectedCells;

        const double CELL_SIZE = 50.0;
        const double COL_HEADER_HEIGHT = 15.0;
        const double ROW_HEADER_WIDTH = 15.0;

        private Maze.Api.Maze maze = new Maze.Api.Maze(3, 3);

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
                if(row == 0)
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
                    int currentRow = row, currentCol = col;
                    tapGesture.Tapped += (s, e) => OnCellTapped(cellFrame, currentRow, currentCol);
                    cellFrame.GestureRecognizers.Add(tapGesture);


                    // Add the Frame to the grid at the specified row and column
                    this.Add(cellFrame, col+1, row+1);

                }
            }
            this.IsVisible = true;
        }
        private void AddHeaderRow()
        {
            for (int col = 0; col < ColCount; col++)
            {
                AddColumnHeader(col);
            }
        }

        private void AddColumnHeader(int col)
        {
            Frame cellFrame = NewHeaderCell();
            var tapGesture = new TapGestureRecognizer();
            int currentCol = col;
            tapGesture.Tapped += (s, e) => OnColumnHeaderTapped(cellFrame, 0, currentCol);
            cellFrame.GestureRecognizers.Add(tapGesture);
            this.Add(cellFrame, col + 1, 0);
        }

        private void AddRowHeader(int row)
        {
            Frame cellFrame = NewHeaderCell();
            var tapGesture = new TapGestureRecognizer();
            int currentRow = row;
            tapGesture.Tapped += (s, e) => OnRowHeaderTapped(cellFrame, currentRow, 0);
            cellFrame.GestureRecognizers.Add(tapGesture);
            this.Add(cellFrame, 0, row + 1);
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

        private void OnCellTapped(Frame cell, int row, int col)
        {
            UpdateSelection(cell, row, col, IsShiftKeyPressed());
        }

        private void OnColumnHeaderTapped(Frame cell, int row, int col)
        {
        }

        private void OnRowHeaderTapped(Frame cell, int row, int col)
        {
        }

        // Move the active cell
        private void MoveActiveCell(bool maintainSelection, int deltaX, int deltaY)
        {
            // Calculate the new position
            int newRow = Math.Clamp(activeCellRow + deltaY, 1, this.RowDefinitions.Count);
            int newCol = Math.Clamp(activeCellCol + deltaX, 1, this.ColumnDefinitions.Count);

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
                if (!haveAnchorCell)
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
            activeCell.BackgroundColor = Colors.HotPink;

            activeCellRow = row;
            activeCellCol = col;

            if (haveAnchorCell)
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
            haveAnchorCell = true;
            anchorCellRow = row;
            anchorCellCol = col;
        }

        private void ClearAnchorCell()
        {
            haveAnchorCell = false;
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
                    if (row != activeCellRow || col != activeCellCol)
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
