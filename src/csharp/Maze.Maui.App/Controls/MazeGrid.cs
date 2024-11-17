
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
                if (startCell != null && startCell.IsDisplayPosition(currentSelection.Top, currentSelection.Left))
                    return;
                if (startCell != null)
                    SetCellContent(startCell, Maze.Api.Maze.CellType.Empty);
                startCell = SetCellContent(currentSelection.Top, currentSelection.Left, Maze.Api.Maze.CellType.Start);
                if (startCell != null && finishCell != null && finishCell.IsDisplayPosition(startCell.DisplayRow, startCell.DisplayColumn))
                    finishCell = null;
            }
        }
        private void SetSelectionToFinishCell()
        {
            InteractiveGrid.CellRange? currentSelection = CurrentSelection;
            if (currentSelection != null && currentSelection.IsSingleCell)
            {
                if (finishCell != null && finishCell.IsDisplayPosition(currentSelection.Top, currentSelection.Left))
                    return;
                if (finishCell != null)
                    SetCellContent(finishCell, Maze.Api.Maze.CellType.Empty);
                finishCell = SetCellContent(currentSelection.Top, currentSelection.Left, Maze.Api.Maze.CellType.Finish);
                if (finishCell != null && startCell != null && startCell.IsDisplayPosition(finishCell.DisplayRow, finishCell.DisplayColumn))
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
            MazeCellContent.PathDirection prevCellDirection = MazeCellContent.PathDirection.None;
            Api.Maze.Point? nextPoint = null;
            Api.Maze.Point thisPoint;
            MazeCellContent.PathDirection thisCellDirection = MazeCellContent.PathDirection.None;

            for (int i = 0; i < points.Count; i++)
            {
                thisPoint = points[i];
                thisCellDirection = MazeCellContent.PathDirection.None;
                nextPoint = (i + 1) < points.Count ? points[i + 1] : null;
                thisCellDirection = GetCellPathDirection(prevCellDirection, thisPoint, nextPoint);
                SetSolutionCell((int)(thisPoint.Row) + 1, (int)(thisPoint.Column) + 1, thisCellDirection);
                prevCellDirection = thisCellDirection;
            }

            haveSolutionCells = points.Count > 0;

            return true;
        }

        private MazeCellContent.PathDirection GetCellPathDirection(
            MazeCellContent.PathDirection prevCellDirection,
            Api.Maze.Point cellPoint,
            Api.Maze.Point? nextCellPoint
        )
        {
            MazeCellContent.PathDirection direction = MazeCellContent.PathDirection.None;

            if (nextCellPoint != null)
            {
                Api.Maze.Point nextPoint = nextCellPoint.Value;

                direction = GetCellOffsetDirection(prevCellDirection, cellPoint, nextPoint);
            }
            else
            {
                direction = GetContinueDirection(prevCellDirection);
            }

            return direction;
        }

        private MazeCellContent.PathDirection GetCellOffsetDirection(MazeCellContent.PathDirection prevDirection, Api.Maze.Point from, Api.Maze.Point to)
        {
            MazeCellContent.PathDirection direction = MazeCellContent.PathDirection.None;
            bool sameRow = from.Row == to.Row;
            bool sameColumn = from.Column == to.Column;

            if (!(sameRow && sameColumn))
            {
                if (sameColumn)
                {
                    direction = to.Row > from.Row ? GetDownDirection(prevDirection) : GetUpDirection(prevDirection);
                }
                if (sameRow)
                {
                    direction = to.Column > from.Column ? GetRightDirection(prevDirection) : GetLeftDirection(prevDirection);
                }
            }

            return direction;
        }

        private MazeCellContent.PathDirection GetUpDirection(MazeCellContent.PathDirection prevDirection)
        {
            switch (prevDirection)
            {
                case MazeCellContent.PathDirection.Left:
                    return MazeCellContent.PathDirection.UpFromLeft;
                case MazeCellContent.PathDirection.Right:
                    return MazeCellContent.PathDirection.UpFromRight;
            }

            return MazeCellContent.PathDirection.Up;
        }

        private MazeCellContent.PathDirection GetDownDirection(MazeCellContent.PathDirection prevDirection)
        {
            switch (prevDirection)
            {
                case MazeCellContent.PathDirection.Left:
                    return MazeCellContent.PathDirection.DownFromLeft;
                case MazeCellContent.PathDirection.Right:
                    return MazeCellContent.PathDirection.DownFromRight;
            }

            return MazeCellContent.PathDirection.Down;
        }

        private MazeCellContent.PathDirection GetLeftDirection(MazeCellContent.PathDirection prevDirection)
        {
            switch (prevDirection)
            {
                case MazeCellContent.PathDirection.Up:
                    return MazeCellContent.PathDirection.LeftFromUp;
                case MazeCellContent.PathDirection.Down:
                    return MazeCellContent.PathDirection.LeftFromDown;
            }

            return MazeCellContent.PathDirection.Left;
        }

        private MazeCellContent.PathDirection GetRightDirection(MazeCellContent.PathDirection prevDirection)
        {
            switch (prevDirection)
            {
                case MazeCellContent.PathDirection.Up:
                    return MazeCellContent.PathDirection.RightFromUp;
                case MazeCellContent.PathDirection.Down:
                    return MazeCellContent.PathDirection.RightFromDown;
            }

            return MazeCellContent.PathDirection.Right;
        }

        private MazeCellContent.PathDirection GetContinueDirection(MazeCellContent.PathDirection currentDirection)
        {
            MazeCellContent.PathDirection direction = MazeCellContent.PathDirection.None;

            switch (currentDirection)
            {
                case MazeCellContent.PathDirection.Left:
                case MazeCellContent.PathDirection.LeftFromDown:
                case MazeCellContent.PathDirection.LeftFromUp:
                    direction = MazeCellContent.PathDirection.Left;
                    break;
                case MazeCellContent.PathDirection.Right:
                case MazeCellContent.PathDirection.RightFromDown:
                case MazeCellContent.PathDirection.RightFromUp:
                    direction = MazeCellContent.PathDirection.Right;
                    break;
                case MazeCellContent.PathDirection.Up:
                case MazeCellContent.PathDirection.UpFromLeft:
                case MazeCellContent.PathDirection.UpFromRight:
                    direction = MazeCellContent.PathDirection.Up;
                    break;
                case MazeCellContent.PathDirection.Down:
                case MazeCellContent.PathDirection.DownFromLeft:
                case MazeCellContent.PathDirection.DownFromRight:
                    direction = MazeCellContent.PathDirection.Down;
                    break;
            }

            return direction;
        }

        public bool ClearLastSolution()
        {
            if (haveSolutionCells)
            {
                for (int row = 0; row < RowCount; row++)
                {
                    for (int column = 0; column < ColumnCount; column++)
                    {
                        SetSolutionCell(row + 1, column + 1, MazeCellContent.PathDirection.None);
                    }
                }
                haveSolutionCells = false;
            }
            return true;
        }

        private void SetSolutionCell(int row, int column, MazeCellContent.PathDirection direction)
        {
            MazeCellContent? cellContent = GetCellContent(row, column);
            if (cellContent != null)
            {
                cellContent.SetSolutionPath(direction);
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
        public enum PathDirection
        {
            None = 0,
            Left = 1,
            LeftFromDown = 2,
            LeftFromUp = 3,
            Right = 4,
            RightFromDown = 5,
            RightFromUp = 6,
            Up = 7,
            UpFromLeft = 8,
            UpFromRight = 9,
            Down = 10,
            DownFromLeft = 11,
            DownFromRight = 12
        }

        static private Color SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR = Colors.White;

        static private Color SOLUTION_PATH_CELL_HIGHLIGHT_COLOR = Colors.LightGreen;

        Maze.Api.Maze.CellType cellType = Api.Maze.CellType.Empty;
        MazeCellContent.PathDirection solutionPathDirection = MazeCellContent.PathDirection.None;

        public PathDirection SolutionPathDirection { get => solutionPathDirection; }

        public bool ContainsSolutionPath { get => solutionPathDirection != PathDirection.None; }

        public Maze.Api.Maze.CellType CellType { get => cellType; }

        public bool IsEmpty { get => CellType == CellType.Empty; }

        public bool IsStart { get => CellType == CellType.Start; }

        public bool IsFinish { get => CellType == CellType.Finish; }

        public bool IsStartOrFinish { get => IsStart || IsFinish; }

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

        public void SetSolutionPath(PathDirection pathDirection)
        {
            if (IsEmpty || IsStartOrFinish)
            {
                solutionPathDirection = pathDirection;

                if (IsEmpty)
                {
                    Content = ContainsSolutionPath ? Content = new Image
                    {
                        Source = GetSolutionPathImage(),
                        Aspect = Aspect.AspectFit,
                        HorizontalOptions = LayoutOptions.Fill,
                        VerticalOptions = LayoutOptions.Fill
                    } : new Label();
                }
                Content.BackgroundColor = GetSolutionPathHighlightColor();
            }
        }

        private Color GetSolutionPathHighlightColor()
        {
            return ContainsSolutionPath
                ? (IsStartOrFinish ? SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR : SOLUTION_PATH_CELL_HIGHLIGHT_COLOR) 
                : Colors.Transparent;
        }

        private string GetSolutionPathImage()
        {
            switch (SolutionPathDirection)
            {
                case PathDirection.Left:
                case PathDirection.LeftFromDown:
                case PathDirection.LeftFromUp:
                    return "footsteps_left.png";
                case PathDirection.Right:
                case PathDirection.RightFromDown:
                case PathDirection.RightFromUp:
                    return "footsteps_right.png";
                case PathDirection.Up:
                case PathDirection.UpFromLeft:
                case PathDirection.UpFromRight:
                    return "footsteps_up.png";
                case PathDirection.Down:
                case PathDirection.DownFromLeft:
                case PathDirection.DownFromRight:
                    return "footsteps_down.png";
                case PathDirection.None:
                default:
                    return "";
            }
        }
    }

}
