
using Maze.Api;
using Maze.Maui.App.Controls.InteractiveGrid;
using Microsoft.Maui.Controls;
using System.Data.Common;
using static Maze.Api.Maze;

namespace Maze.Maui.App.Controls
{
    public class MazeGrid : InteractiveGrid.Grid
    {
        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellTappedEventHandler? CellTapped;

        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellDoubleTappedEventHandler? CellDoubleTapped;

        public delegate void SelectionChangedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        public event SelectionChangedEventHandler? SelectionChanged;

        private CellFrame? startCell;
        private CellFrame? finishCell;

        public CellFrame? StartCell { get => startCell; }

        public CellFrame? FinishCell { get => finishCell; }

        public MazeGrid()
        {
        }

        public void Initialize(bool enablePanSupport)
        {
            this.IsPanSupportEnabled = enablePanSupport;
            this.RowCount = 30; //(int)maze.RowCount;
            this.ColumnCount = 15; //(int)maze.ColCount;
            PopulateGrid();
        }

        public CellStatus GetCurrentSelectionStatus()
        {
            InteractiveGrid.CellRange? currentSelection = CurrentSelection;
            int cellCount = 0;
            bool singleCell = false, containsStart = false, containsFinish = false, containsWall = false;
            int numWalls = 0;
            if (currentSelection != null)
            {
                Maze.Api.Maze.CellType cellType = Maze.Api.Maze.CellType.Empty;

                cellCount = currentSelection.CellCount;
                singleCell = cellCount == 1;
                for (int row = currentSelection.Top; row <= currentSelection.Bottom; row++)
                {
                    for (int column = currentSelection.Left; column <= currentSelection.Right; column++)
                    {
                        cellType = GetCellType(row, column);
                        switch (cellType)
                        {
                            case Api.Maze.CellType.Start:
                                containsStart = true;
                                break;
                            case Api.Maze.CellType.Finish:
                                containsFinish = true;
                                break;
                            case Api.Maze.CellType.Wall:
                                containsWall = true;
                                numWalls++;
                                break;
                        }
                    }
                }
            }
            return new CellStatus()
            {
                IsSingleCell = singleCell,
                ContainsWall = containsWall,
                ContainsStart = containsStart,
                ContainsFinish = containsFinish,
                IsAllWalls = containsWall && numWalls == cellCount
            };
        }

        public Maze.Api.Maze.CellType GetCellType(int row, int column)
        {
            if (row >= 0 && column >= 0)
            {
                InteractiveGrid.CellFrame? cellFrame = GetCell(row, column) as InteractiveGrid.CellFrame;
                if (cellFrame != null)
                {
                    MazeCellContent? cellContent = cellFrame.Content as MazeCellContent;
                    if (cellContent != null)
                    {
                        return cellContent.CellType;
                    }
                }
            }
            return Api.Maze.CellType.Empty;
        }

        public override ContentView InitialzeCellContent(int row, int column)
        {
            return new MazeCellContent(Maze.Api.Maze.CellType.Empty);
        }

        public override void OnCellTapped(InteractiveGrid.CellFrame cellFrame, bool triggerEvents)
        {
            if (triggerEvents && CellTapped != null)
            {
                CellTapped.Invoke(this, new MazeGridCellTappedEventArgs(cellFrame, 1));
            }
            else
            {
                base.OnCellTapped(cellFrame, false);
            }
        }

        public override void OnCellDoubleTapped(InteractiveGrid.CellFrame cellFrame, bool triggerEvents)
        {
            if (triggerEvents && CellDoubleTapped != null)
            {
                CellDoubleTapped.Invoke(this, new MazeGridCellTappedEventArgs(cellFrame, 2));
            }
            else
            {
                base.OnCellDoubleTapped(cellFrame, false);
            }
        }

        public override void OnSelectionChanged()
        {
            SelectionChanged?.Invoke(this, new MazeGridSelectionChangedEventArgs());
        }

        public void SetSelectionContent(Maze.Api.Maze.CellType cellType)
        {
            switch (cellType)
            {
                case Api.Maze.CellType.Start:
                    SetSelectionToStartCell();
                    break;
                case Api.Maze.CellType.Finish:
                    SetSelectionToFinishCell();
                    break;

                case Api.Maze.CellType.Wall:
                case Api.Maze.CellType.Empty:
                    SetSelectionContentToType(cellType);
                    break;
            }
        }
        private void SetSelectionToStartCell()
        {
            InteractiveGrid.CellRange? currentSelection = CurrentSelection;
            if (currentSelection != null && currentSelection.IsSingleCell)
            {
                if (startCell != null && startCell.IsPosition(currentSelection.Top, currentSelection.Left))
                    return;
                if (startCell != null)
                    SetCellContent(startCell, Maze.Api.Maze.CellType.Empty);
                startCell = SetCellContent(currentSelection.Top, currentSelection.Left, Maze.Api.Maze.CellType.Start);
            }
        }
        private void SetSelectionToFinishCell()
        {
            InteractiveGrid.CellRange? currentSelection = CurrentSelection;
            if (currentSelection != null && currentSelection.IsSingleCell)
            {
                if (finishCell != null && finishCell.IsPosition(currentSelection.Top, currentSelection.Left))
                    return;
                if (finishCell != null)
                    SetCellContent(finishCell, Maze.Api.Maze.CellType.Empty);
                finishCell = SetCellContent(currentSelection.Top, currentSelection.Left, Maze.Api.Maze.CellType.Finish);
            }
        }

        private void SetSelectionContentToType(Maze.Api.Maze.CellType cellType)
        {
            InteractiveGrid.CellRange? currentSelection = CurrentSelection;
            if (currentSelection != null && cellType != CellType.Start && cellType != CellType.Finish)
            {
                for (int row = currentSelection.Top; row <= currentSelection.Bottom; row++)
                {
                    for (int column = currentSelection.Left; column <= currentSelection.Right; column++)
                        SetCellContent(row, column, cellType);
                }
                if (startCell != null && currentSelection.ContainsPosition(startCell.DisplayRow, startCell.DisplayColumn))
                    startCell = null;
                if (finishCell != null && currentSelection.ContainsPosition(finishCell.DisplayRow, finishCell.DisplayColumn))
                    finishCell = null;
            }
        }

        private CellFrame? SetCellContent(int row, int column, Maze.Api.Maze.CellType cellType)
        {
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame != null)
                cellFrame = SetCellContent(cellFrame, cellType);
            return cellFrame;
        }

        private CellFrame? SetCellContent(CellFrame? cellFrame, Maze.Api.Maze.CellType cellType)
        {
            if (cellFrame != null)
            {
                MazeCellContent? cellContent = cellFrame.Content as MazeCellContent;
                if (cellContent != null && cellContent.CellType != cellType)
                    SetCellContent(cellFrame, new MazeCellContent(cellType));
            }
            return cellFrame;
        }
    }

    public class MazeGridCellTappedEventArgs : EventArgs
    {
        public InteractiveGrid.CellFrame Cell { get; }
        public int Row { get => Cell.DisplayRow; }
        public int Column { get => Cell.DisplayColumn; }
        public int NumberTaps { get; }

        public MazeGridCellTappedEventArgs(InteractiveGrid.CellFrame cellFrame, int numberTaps)
        {
            Cell = cellFrame;
            NumberTaps = numberTaps;
        }
    }

    public class MazeGridSelectionChangedEventArgs : EventArgs
    {
        public MazeGridSelectionChangedEventArgs()
        {
        }
    }

    public class CellStatus
    {
        public bool ContainsWall { get; set; } = false;

        public bool ContainsStart { get; set; } = false;

        public bool ContainsFinish { get; set; } = false;

        public bool IsSingleCell { get; set; } = false;

        public bool IsAllWalls { get; set; } = false;

        public bool IsStart { get => IsSingleCell && ContainsStart; }

        public bool IsFinish { get => IsSingleCell && ContainsFinish; }

        public bool IsEmpty { get => !ContainsWall && !ContainsStart && !ContainsFinish; }

        public CellStatus() { }
    }

    public class MazeCellContent : ContentView
    {
        Maze.Api.Maze.CellType cellType = Api.Maze.CellType.Empty;

        public Maze.Api.Maze.CellType CellType { get => cellType; }

        public MazeCellContent(Maze.Api.Maze.CellType cellType)
        {
            this.cellType = cellType;
            Content = new Label
            {
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center,
                Text = ToText()
            };
        }

        private string ToText()
        {
            switch (cellType)
            {
                case Api.Maze.CellType.Empty:
                    return "";
                case Api.Maze.CellType.Wall:
                    return "WALL";
                case Api.Maze.CellType.Start:
                    return "START";
                case Api.Maze.CellType.Finish:
                    return "FINISH";
            }
            return "?";
        }
    }
}
