
using Maze.Api;
using Maze.Maui.App.Controls.InteractiveGrid;
using Microsoft.Maui.Controls;
using System.Data.Common;
using static Maze.Api.Maze;

namespace Maze.Maui.App.Controls
{
    public class MazeGrid : InteractiveGrid.Grid
    {
        private const int DEFAULT_ROW_COUNT = 5;
        private const int DEFAULT_COLUMN_COUNT = 5;

        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellTappedEventHandler? CellTapped;

        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellDoubleTappedEventHandler? CellDoubleTapped;

        public delegate void ProcessKeyDownEventHandler(object sender, MazeGridKeyDownEventArgs e);
        public event ProcessKeyDownEventHandler? KeyDown;

        public delegate void SelectionChangedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        public event SelectionChangedEventHandler? SelectionChanged;

        private CellFrame? startCell;
        private CellFrame? finishCell;
        private bool haveSolutionCells = false;

        public CellFrame? StartCell { get => startCell; }

        public CellFrame? FinishCell { get => finishCell; }

        public MazeGrid()
        {
            this.SelectionFrameBorderColor = Colors.Red;
        }

        public void Initialize(bool enablePanSupport)
        {
            this.IsPanSupportEnabled = enablePanSupport;
            this.RowCount = DEFAULT_ROW_COUNT;
            this.ColumnCount = DEFAULT_COLUMN_COUNT;
            InitializeContent();
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

        public MazeCellContent? GetCellContent(int row, int column)
        {
            MazeCellContent? content = null;
            if (row >= 0 && column >= 0)
            {
                InteractiveGrid.CellFrame? cellFrame = GetCell(row, column) as InteractiveGrid.CellFrame;
                if (cellFrame != null)
                    content = cellFrame.Content as MazeCellContent;
            }
            return content;
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

        public override ContentView CreateCellContent(int row, int column)
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


        public override void OnProcessKeyDown(Keyboard.KeyState state, Keyboard.Key key, bool triggerEvents)
        {
            if (triggerEvents && KeyDown != null)
            {
                KeyDown.Invoke(this, new MazeGridKeyDownEventArgs(state, key));
            }
            else
            {
                base.OnProcessKeyDown(state, key, false);
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
                if (startCell != null && finishCell != null && finishCell.IsPosition(startCell.DisplayRow, startCell.DisplayColumn))
                    finishCell = null;
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
                if (finishCell != null && startCell != null && startCell.IsPosition(finishCell.DisplayRow, finishCell.DisplayColumn))
                    startCell = null;
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

        public Maze.Api.Maze ToMaze()
        {
            Maze.Api.Maze maze = new Maze.Api.Maze((uint)RowCount, (uint)ColumnCount);
            CellType cellType;

            for (int row = 0; row < RowCount; row++)
            {
                for (int column = 0; column < ColumnCount; column++)
                {
                    cellType = GetCellType(row + 1, column + 1);
                    switch (cellType)
                    {
                        case Api.Maze.CellType.Start:
                            maze.SetStartCell((uint)row, (uint)column);
                            break;
                        case Api.Maze.CellType.Finish:
                            maze.SetFinishCell((uint)row, (uint)column);
                            break;
                        case Api.Maze.CellType.Wall:
                            maze.SetWallCells((uint)row, (uint)column, (uint)row, (uint)column);
                            break;
                    }
                }
            }

            return maze;
        }

        public bool DisplaySolution(Maze.Api.Solution solution)
        {
            if (haveSolutionCells)
                ClearLastSolution();

            List<Api.Maze.Point> points = solution.GetPathPoints();
            foreach (Api.Maze.Point point in points)
            {
                SetSolutionCell((int)point.Row + 1, (int)point.Column + 1, true);
            }
            
            haveSolutionCells = true;

            return true;
        }

        public bool ClearLastSolution()
        {
            if (haveSolutionCells)
            {
                for (int row = 0; row < RowCount; row++)
                {
                    for (int column = 0; column < ColumnCount; column++)
                    {
                        SetSolutionCell(row + 1, column + 1, false);
                    }
                }
                haveSolutionCells = false;
            }
            return true;
        }

        private void SetSolutionCell(int row, int column, bool isSolutionCell)
        {
            MazeCellContent? cellContent = GetCellContent(row, column);
            if (cellContent != null)
            {
                cellContent.IsSolutionCell = isSolutionCell;
            }
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

    public class MazeGridKeyDownEventArgs : EventArgs
    {
        Keyboard.KeyState keyState = Keyboard.KeyState.None;
        Keyboard.Key key = Keyboard.Key.None;

        public Keyboard.KeyState KeyState { get => keyState; }

        public Keyboard.Key Key { get => key; }

        public bool IsShiftKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.Shift); }

        public bool IsCtrlKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.Ctrl); }

        public bool IsCapsLockKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.CapsLock); }

        public MazeGridKeyDownEventArgs(Keyboard.KeyState keyState, Keyboard.Key key)
        {
            this.keyState = keyState;
            this.key = key;
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
        static private Color COLOR_REF_lIGHT_TURQUOISE = Color.FromRgb(153, 217, 234);

        Maze.Api.Maze.CellType cellType = Api.Maze.CellType.Empty;
        bool isSolutionCell = false;
        public bool IsSolutionCell
        {
            get => isSolutionCell;
            set
            {
                if (isSolutionCell != value)
                {
                    isSolutionCell = value;
                    Content.BackgroundColor = isSolutionCell ? COLOR_REF_lIGHT_TURQUOISE : Colors.White;
                }
            }
        }
        public Maze.Api.Maze.CellType CellType { get => cellType; }

        public MazeCellContent(Maze.Api.Maze.CellType cellType)
        {
            this.cellType = cellType;
            switch (cellType)
            {
                case CellType.Start:
                case CellType.Finish:
                case CellType.Wall:
                    Content = new Image
                    {
                        Source = GetImageName(true),
                        Aspect = Aspect.AspectFit,
                        HorizontalOptions = LayoutOptions.Fill,
                        VerticalOptions = LayoutOptions.Fill
                    };
                    break;
                case CellType.Empty:
                default:
                    Content = new Label();
                    break;
            }
        }
        private string GetImageName(bool preferFlag)
        {
            switch (cellType)
            {
                case Api.Maze.CellType.Start:
                    return preferFlag ? "start_flag.png" : "start_sign.png";
                case Api.Maze.CellType.Finish:
                    return preferFlag ? "finish_flag.png" : "finish_sign.png";
                case Api.Maze.CellType.Wall:
                    return "wall.png";
            }
            return "";
        }
    }
}
