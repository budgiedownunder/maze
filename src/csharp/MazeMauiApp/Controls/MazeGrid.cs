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

        private Maze.Api.Maze maze = new Maze.Api.Maze(3, 3);

        public MazeGrid()
        {
            initializePlatformSpecificCode();
            populateGrid();
        }

        public static readonly BindableProperty ContainerScrollViewProperty =
            BindableProperty.Create(nameof(ContainerScrollView), typeof(ScrollView), typeof(MazeGrid));

        public ScrollView ContainerScrollView
        {
            get => (ScrollView)GetValue(ContainerScrollViewProperty);
            set => SetValue(ContainerScrollViewProperty, value);
        }
        partial void initializePlatformSpecificCode();  // Platform-specific method stub

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


        private void populateGrid()
        {

            this.IsVisible = false;

            for (int row = 0; row < RowCount; row++)
            {
                this.RowDefinitions.Add(new RowDefinition { Height = new GridLength(CELL_SIZE) });
            }

            for (int col = 0; col < ColCount; col++)
            {
                this.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(CELL_SIZE) });
            }

            // Populate the grid with Frames
            for (int row = 0; row < RowCount; row++)
            {
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
                    this.Add(cellFrame, col, row);

                }
            }
            this.IsVisible = true;
        }
        private void OnCellTapped(Frame cell, int row, int col)
        {
            UpdateSelection(cell, row, col, isShiftKeyPressed());
        }

        // Move the active cell
        private void moveActiveCell(bool maintainSelection, int deltaX, int deltaY)
        {
            // Calculate the new position
            int newRow = Math.Clamp(activeCellRow + deltaY, 0, this.RowDefinitions.Count - 1);
            int newCol = Math.Clamp(activeCellCol + deltaX, 0, this.ColumnDefinitions.Count - 1);

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
                        setAnchorCell(row, col);
                    }
                    else
                        setAnchorCell(activeCellRow, activeCellCol);
                }
            }
            else
            {
                clearSelectedCells();
                clearAnchorCell();
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
                updateSelectedCells();
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
        private static bool isShiftKeyPressed()
        {
            return false;
        }
#endif
        private void setAnchorCell(int row, int col)
        {
            haveAnchorCell = true;
            anchorCellRow = row;
            anchorCellCol = col;
        }

        private void clearAnchorCell()
        {
            haveAnchorCell = false;
            anchorCellRow = -1;
            anchorCellCol = -1;
        }

        private void clearSelectedCells()
        {
            if (currentSelectedCells != null)
            {
                highlightCells(currentSelectedCells.Value, true);
                currentSelectedCells = null;
            }

        }
        private void updateSelectedCells()
        {
            clearSelectedCells();
            int startRow = Math.Min(anchorCellRow, activeCellRow);
            int startCol = Math.Min(anchorCellCol, activeCellCol);
            int width = Math.Abs(anchorCellCol - activeCellCol) + 1;
            int height = Math.Abs(anchorCellRow - activeCellRow) + 1;
            currentSelectedCells = new Rect(startCol, startRow, width, height);
            highlightCells(currentSelectedCells.Value, false);
        }
        private void highlightCells(Microsoft.Maui.Graphics.Rect region, bool clear)
        {
            for (int row = (int)region.Top; row < (int)region.Bottom; row++)
            {
                for (int col = (int)region.Left; col < (int)region.Right; col++)
                {
                    if (row != activeCellRow || col != activeCellCol)
                    {
                        Frame? cell = getCell(row, col);
                        if (cell != null)
                        {
                            cell.BackgroundColor = clear ? Colors.White : Colors.Yellow;
                        }
                    }
                }
            }
        }
        private Frame? getCell(int row, int col)
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
