
using Maze.Api;
using Maze.Maui.App.Controls.InteractiveGrid;
using Microsoft.Maui.Controls;
using System.Data.Common;
using static Maze.Api.Maze;

namespace Maze.Maui.App.Controls
{
    /// <summary>
    /// The `MazeGrid` class represents an interactive maze grid
    /// </summary>
    public class MazeGrid : InteractiveGrid.Grid
    {
        // Private properties
        private const int DEFAULT_ROW_COUNT = 5;
        private const int DEFAULT_COLUMN_COUNT = 5;

        private CellFrame? startCell;
        private CellFrame? finishCell;
        private bool haveSolutionCells = false;

        /// <summary>
        /// Cell tapped event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid cell tapped event arguments</param>
        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        /// <summary>
        /// Registered cell tapped event handler
        /// </summary>
        public event CellTappedEventHandler? CellTapped;
        /// <summary>
        /// Cell double-tapped event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid cell tapped event arguments</param>
        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        /// <summary>
        /// Registered cell double-tapped event handler
        /// </summary>
        public event CellDoubleTappedEventHandler? CellDoubleTapped;
        /// <summary>
        /// Key down event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid key down event arguments</param>
        public delegate void ProcessKeyDownEventHandler(object sender, MazeGridKeyDownEventArgs e);
        /// <summary>
        /// Registered key down event handler
        /// </summary>
        public event ProcessKeyDownEventHandler? KeyDown;
        /// <summary>
        /// Selection changed event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid selection changed event arguments</param>
        public delegate void SelectionChangedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        /// <summary>
        /// Registered selection changed event handler
        /// </summary>
        public event SelectionChangedEventHandler? SelectionChanged;
        /// <summary>
        /// Start cell (if any)
        /// </summary>
        /// <returns>Start cell</returns>
        public CellFrame? StartCell { get => startCell; }
        /// <summary>
        /// Finish cell (if any)
        /// </summary>
        /// <returns>Finish cell</returns>
        public CellFrame? FinishCell { get => finishCell; }
        /// <summary>
        /// Constructor
        /// </summary>
        public MazeGrid()
        {
            this.SelectionFrameBorderColor = Colors.Red;
        }
        /// <summary>
        /// Initialize
        /// </summary>
        /// <param name="enablePanSupport">Enable pan support?</param>
        public void Initialize(bool enablePanSupport)
        {
            this.IsPanSupportEnabled = enablePanSupport;
            this.RowCount = DEFAULT_ROW_COUNT;
            this.ColumnCount = DEFAULT_COLUMN_COUNT;
            InitializeContent();
        }
        /// <summary>
        /// Gets the current selection status
        /// </summary>
        /// <returns>Selection status</returns>
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
        /// <summary>
        /// Gets the maze cell content at a given location
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <returns>Maze cell content</returns>
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
        /// <summary>
        /// Gets the maze cell type at a given location
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <returns>Maze cell type</returns>
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
        /// <summary>
        /// Creates the maze cell content for a given location
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <returns>Maze cell content</returns>
        public override ContentView CreateCellContent(int row, int column)
        {
            return new MazeCellContent(Maze.Api.Maze.CellType.Empty);
        }
        /// <summary>
        /// Handles the cell tapped event
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
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
        /// <summary>
        /// Handles the cell double-tapped event
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
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
        /// <summary>
        /// Handles the key down event
        /// </summary>
        /// <param name="state">Key state</param>
        /// <param name="key">Key pressed</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
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
        /// <summary>
        /// Handles the selection changed event
        /// </summary>
        public override void OnSelectionChanged()
        {
            SelectionChanged?.Invoke(this, new MazeGridSelectionChangedEventArgs());
        }
        /// <summary>
        /// Sets the content in the selected cells to the given cell type
        /// </summary>
        /// <param name="cellType">Cell type</param>
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
        /// <summary>
        /// Sets the content in the selected cells to be the start cell
        /// </summary>
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
        /// <summary>
        /// Sets the content in the selected cells to be the finish cell
        /// </summary>
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
        /// <summary>
        /// Sets the content in the selected cells to be a given type, providing it is not a start or finish cell type
        /// </summary>
        /// <param name="cellType">Cell type</param>
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
        /// <summary>
        /// Sets the content in the cell location to be a given type
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <param name="cellType">Cell type</param>
        /// <returns>Cell frame</returns>
        private CellFrame? SetCellContent(int row, int column, Maze.Api.Maze.CellType cellType)
        {
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame != null)
                cellFrame = SetCellContent(cellFrame, cellType);
            return cellFrame;
        }
        /// <summary>
        /// Sets the content in given cell frame to be a given type
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="cellType">Cell type</param>
        /// <returns>Cell frame</returns>
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
        /// <summary>
        /// Converts the maze grid content to a `Maze` object
        /// </summary>
        /// <returns>Maze object</returns>
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
        /// <summary>
        /// Adds the path associated with the given solution to the display
        /// </summary>
        /// <param name="solution">Maze solution</param>
        /// <returns>Boolean</returns>
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
        /// <summary>
        /// Gets the path direction to display for a given cell in the solution path
        /// </summary>
        /// <param name="prevCellDirection">Previous cell direction</param>
        /// <param name="cellPoint">Cell point</param>
        /// <param name="nextCellPoint">Next cell point</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the cell offset direction to display for moving from one cell to another
        /// </summary>
        /// <param name="prevCellDirection">Previous cell direction</param>
        /// <param name="from">From point</param>
        /// <param name="to">To point</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the up direction to be used following a previous direction
        /// </summary>
        /// <param name="prevDirection">Previous cell direction</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the down direction to be used following a previous direction
        /// </summary>
        /// <param name="prevDirection">Previous cell direction</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the left direction to be used following a previous direction
        /// </summary>
        /// <param name="prevDirection">Previous cell direction</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the right direction to be used following a previous direction
        /// </summary>
        /// <param name="prevDirection">Previous cell direction</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Gets the contiue direction to be used for the given current direction
        /// </summary>
        /// <param name="currentDirection">Current cell direction</param>
        /// <returns>Path direction</returns>
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
        /// <summary>
        /// Clears the last displayed solution
        /// </summary>
        /// <returns>Boolean</returns>
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
        /// <summary>
        /// Sets a solution cell direction
        /// /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <param name="direction">Direction</param>
        private void SetSolutionCell(int row, int column, MazeCellContent.PathDirection direction)
        {
            MazeCellContent? cellContent = GetCellContent(row, column);
            if (cellContent != null)
            {
                cellContent.SetSolutionPath(direction);
            }
        }
    }
    /// <summary>
    /// The `MazeGridCellTappedEventArgs` class contains the details of a cell tapped event
    /// </summary>
    public class MazeGridCellTappedEventArgs : EventArgs
    {
        /// <summary>
        /// The cell frame that was tapped
        /// </summary>
        /// <returns>Cell frame</returns>
        public InteractiveGrid.CellFrame Cell { get; }
        /// <summary>
        /// The display row that was tapped
        /// </summary>
        /// <returns>Display row</returns>
        public int Row { get => Cell.DisplayRow; }
        /// <summary>
        /// The display column that was tapped
        /// </summary>
        /// <returns>Display column</returns>
        public int Column { get => Cell.DisplayColumn; }
        /// <summary>
        /// The number of taps that were made
        /// </summary>
        /// <returns>Number of taps</returns>
        public int NumberTaps { get; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="numberTaps">Number of taps</param>
        public MazeGridCellTappedEventArgs(InteractiveGrid.CellFrame cellFrame, int numberTaps)
        {
            Cell = cellFrame;
            NumberTaps = numberTaps;
        }
    }
    /// <summary>
    /// The `MazeGridKeyDownEventArgs` class contains the details of a key down event
    /// </summary>
    public class MazeGridKeyDownEventArgs : EventArgs
    {
        // Private properties
        Keyboard.KeyState keyState = Keyboard.KeyState.None;
        Keyboard.Key key = Keyboard.Key.None;

        /// <summary>
        /// Additional key state information
        /// </summary>
        /// <returns>Key state</returns>
        public Keyboard.KeyState KeyState { get => keyState; }
        /// <summary>
        /// Key that was pressed
        /// </summary>
        /// <returns>Key</returns>
        public Keyboard.Key Key { get => key; }
        /// <summary>
        /// Indicates whether the shift key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsShiftKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.Shift); }
        /// <summary>
        /// Indicates whether the Ctrl key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsCtrlKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.Ctrl); }
        /// <summary>
        /// Indicates whether the Caps Lock key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsCapsLockKeyPressed { get => Keyboard.Utiility.IsStateFlagSet(KeyState, Keyboard.KeyState.CapsLock); }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="keyState">Additional key state information</param>
        /// <param name="key">Key that was pressed</param>
        public MazeGridKeyDownEventArgs(Keyboard.KeyState keyState, Keyboard.Key key)
        {
            this.keyState = keyState;
            this.key = key;
        }
    }
    /// <summary>
    /// The `MazeGridSelectionChangedEventArgs` class represents a selection change event
    /// </summary>
    public class MazeGridSelectionChangedEventArgs : EventArgs
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public MazeGridSelectionChangedEventArgs()
        {
        }
    }
    /// <summary>
    /// The `CellStatus` class represents the status associated with a maze cell selection
    /// </summary>
    public class CellStatus
    {
        /// <summary>
        /// Indicates whether the selection contains a wall
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsWall { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains a start cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsStart { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains a finish cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsFinish { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection is a single cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsSingleCell { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains all wall cells
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsAllWalls { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection is the start cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsStart { get => IsSingleCell && ContainsStart; }
        /// <summary>
        /// Indicates whether the selection is the finish cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsFinish { get => IsSingleCell && ContainsFinish; }
        /// <summary>
        /// Indicates whether the selection contains all empty cells
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsEmpty { get => !ContainsWall && !ContainsStart && !ContainsFinish; }
        /// <summary>
        /// Constructor
        /// </summary>
        public CellStatus() { }
    }
    /// <summary>
    /// The `MazeCellContent` class defines the content in a maze cell
    /// </summary>
    public class MazeCellContent : ContentView
    {
        /// <summary>
        /// Represents a path direction
        /// </summary>
        public enum PathDirection
        {
            /// <summary>
            /// No direction
            /// </summary>
            None = 0,
            /// <summary>
            /// To left
            /// </summary>
            Left = 1,
            /// <summary>
            /// To left from down
            /// </summary>
            LeftFromDown = 2,
            /// <summary>
            /// To left from up
            /// </summary>
            LeftFromUp = 3,
            /// <summary>
            /// To right
            /// </summary>
            Right = 4,
            /// <summary>
            /// To right from down
            /// </summary>
            RightFromDown = 5,
            /// <summary>
            /// To right from up
            /// </summary>
            RightFromUp = 6,
            /// <summary>
            /// Upwards
            /// </summary>
            Up = 7,
            /// <summary>
            /// Upwards from left
            /// </summary>
            UpFromLeft = 8,
            /// <summary>
            /// Upwards from right
            /// </summary>
            UpFromRight = 9,
            /// <summary>
            /// Downwards
            /// </summary>
            Down = 10,
            /// <summary>
            /// Downwards from left
            /// </summary>
            DownFromLeft = 11,
            /// <summary>
            /// Downwards from right
            /// </summary>
            DownFromRight = 12
        }

        // Private properties
        static private Color SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR = Colors.White;
        static private Color SOLUTION_PATH_CELL_HIGHLIGHT_COLOR = Colors.LightGreen;

        Maze.Api.Maze.CellType cellType = Api.Maze.CellType.Empty;
        MazeCellContent.PathDirection solutionPathDirection = MazeCellContent.PathDirection.None;

        /// <summary>
        /// The solution path direction associated with the cell (if any)
        /// </summary>
        /// <returns>Path direction</returns>
        public PathDirection SolutionPathDirection { get => solutionPathDirection; }
        /// <summary>
        /// Indicates whether the cell contains a solution path
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsSolutionPath { get => solutionPathDirection != PathDirection.None; }
        /// <summary>
        /// The cell type
        /// </summary>
        /// <returns>Cell type</returns>
        public Maze.Api.Maze.CellType CellType { get => cellType; }
        /// <summary>
        /// Indicates whether the cell is empty
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsEmpty { get => CellType == CellType.Empty; }
        /// <summary>
        /// Indicates whether the cell is a start cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsStart { get => CellType == CellType.Start; }
        /// <summary>
        /// Indicates whether the cell is a finish cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsFinish { get => CellType == CellType.Finish; }
        /// <summary>
        /// Indicates whether the cell is a start or finish cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsStartOrFinish { get => IsStart || IsFinish; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="cellType">Cell type</param>
        /// <returns>Boolean</returns>
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
        /// <summary>
        /// Gets the name of the image to display for the cell
        /// </summary>
        /// <param name="preferFlag">If the cell is a start or finish cell, returned a flag image (otherwise return a sign image)</param>
        /// <returns>Image name</returns>
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
        /// <summary>
        /// Sets the solution path direction in the cell
        /// </summary>
        /// <param name="pathDirection">Path direction</param>
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
        /// <summary>
        /// Gets the solution path highlight color for the cell
        /// </summary>
        /// <returns>Highlight color</returns>
        private Color GetSolutionPathHighlightColor()
        {
            return ContainsSolutionPath
                ? (IsStartOrFinish ? SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR : SOLUTION_PATH_CELL_HIGHLIGHT_COLOR) 
                : Colors.Transparent;
        }
        /// <summary>
        /// Gets the solution path image for the cell
        /// </summary>
        /// <returns>Image name</returns>
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
