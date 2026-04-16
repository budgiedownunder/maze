using static Maze.Api.Maze;
using Maze.Maui.Controls.InteractiveGrid;
using Maze.Maui.App.Models;
using Maze.Maui.Controls.Keyboard;

namespace Maze.Maui.App
{
    /// <summary>
    /// The `MazeGrid` class represents an interactive maze grid
    /// </summary>
    public class MazeGrid : Controls.InteractiveGrid.Grid
    {
        private const int DEFAULT_ROW_COUNT = 5;
        private const int DEFAULT_COLUMN_COUNT = 5;
        private MazeItem? mazeItem;
        private bool haveSolutionCells = false;

        // Logical cell state (independent of the visual tree — required for virtualization)
        private CellType[,] _cellTypes = new CellType[0, 0];
        private MazeCellContent.PathDirection[,] _solutionDirections = new MazeCellContent.PathDirection[0, 0];
        // 1-based positions of start/finish cells (-1 = not set)
        private int _startRow = -1, _startCol = -1;
        private int _finishRow = -1, _finishCol = -1;
        // 1-based position of the current walker cell (-1 = no walker)
        private int _walkerRow = -1, _walkerCol = -1;

        /// <summary>
        /// Cell tapped event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid cell tapped event arguments</param>
        /// <returns>Event handler</returns>
        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        /// <summary>
        /// Registered cell tapped event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event CellTappedEventHandler? CellTapped;
        /// <summary>
        /// Cell double-tapped event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid cell tapped event arguments</param>
        /// <returns>Event handler</returns>
        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        /// <summary>
        /// Registered cell double-tapped event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event CellDoubleTappedEventHandler? CellDoubleTapped;
        /// <summary>
        /// Key down event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid key down event arguments</param>
        /// <returns>Event handler</returns>
        public delegate void ProcessKeyDownEventHandler(object sender, MazeGridKeyDownEventArgs e);
        /// <summary>
        /// Registered key down event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event ProcessKeyDownEventHandler? KeyDown;
        /// <summary>
        /// Selection changed event handler delegate
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Maze grid selection changed event arguments</param>
        /// <returns>Event handler</returns>
        public delegate void SelectionChangedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        /// <summary>
        /// Registered selection changed event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event SelectionChangedEventHandler? SelectionChanged;
        /// <summary>
        /// Start cell frame (if currently visible), or null when off-screen or not placed
        /// </summary>
        public CellFrame? StartCell { get => _startRow > 0 ? GetCell(_startRow, _startCol) as CellFrame : null; }
        /// <summary>
        /// Finish cell frame (if currently visible), or null when off-screen or not placed
        /// </summary>
        public CellFrame? FinishCell { get => _finishRow > 0 ? GetCell(_finishRow, _finishCol) as CellFrame : null; }
        /// <summary>
        /// Whether a start cell has been placed in the grid
        /// </summary>
        public bool HasStartCellPlaced { get => _startRow > 0; }
        /// <summary>
        /// Whether a finish cell has been placed in the grid
        /// </summary>
        public bool HasFinishCellPlaced { get => _finishRow > 0; }
        /// <summary>
        /// Constructor
        /// </summary>
        public MazeGrid()
        {
            SelectionFrameBorderColor = Colors.Red;
        }
        /// <summary>
        /// Initialize
        /// </summary>
        /// <param name="enablePanSupport">Enable pan support?</param>
        /// <param name="mazeItem">Maze item (nullable)</param>
        public void Initialize(bool enablePanSupport, MazeItem? mazeItem)
        {
            IsPanSupportEnabled = enablePanSupport;
            // On desktop (pan/pointer support enabled), exit extended selection when the user
            // moves or clicks without Shift. On touch, extended selection is sticky.
            ExitExtendedSelectionOnDeselect = enablePanSupport;
            this.mazeItem = mazeItem;
            RowCount    = (int)(mazeItem?.Definition?.RowCount   ?? DEFAULT_ROW_COUNT);
            ColumnCount = (int)(mazeItem?.Definition?.ColCount ?? DEFAULT_COLUMN_COUNT);

            // Populate the logical cell-type model before building the visual layer
            _cellTypes          = new CellType[RowCount, ColumnCount];
            _solutionDirections = new MazeCellContent.PathDirection[RowCount, ColumnCount];
            _startRow = _startCol = _finishRow = _finishCol = -1;

            for (int r = 0; r < RowCount; r++)
            {
                for (int c = 0; c < ColumnCount; c++)
                {
                    _cellTypes[r, c] = GetMazeItemCellType(r, c);
                    if      (_cellTypes[r, c] == CellType.Start)  { _startRow  = r + 1; _startCol  = c + 1; }
                    else if (_cellTypes[r, c] == CellType.Finish) { _finishRow = r + 1; _finishCol = c + 1; }
                }
            }

            haveSolutionCells = false;

#if ANDROID
            VirtualBuffer = RowCount * ColumnCount <= 900  ? 0 : 10;
#elif IOS || MACCATALYST
            VirtualBuffer = RowCount * ColumnCount <= 1600 ? 0 : 10;
#else
            VirtualBuffer = RowCount * ColumnCount <= 3600 ? 0 : 10;
#endif
            InitializeContent();
        }
        /// <summary>
        /// Gets the current selection status
        /// </summary>
        /// <returns>Selection status</returns>
        public CellStatus GetCurrentSelectionStatus()
        {
            CellRange? currentSelection = CurrentSelection;
            int cellCount = 0;
            bool singleCell = false, containsStart = false, containsFinish = false, containsWall = false;
            int numWalls = 0;
            if (currentSelection is not null)
            {
                CellType cellType = CellType.Empty;

                cellCount = currentSelection.CellCount;
                singleCell = cellCount == 1;
                for (int row = currentSelection.Top; row <= currentSelection.Bottom; row++)
                {
                    for (int column = currentSelection.Left; column <= currentSelection.Right; column++)
                    {
                        cellType = GetCellType(row, column);
                        switch (cellType)
                        {
                            case CellType.Start:
                                containsStart = true;
                                break;
                            case CellType.Finish:
                                containsFinish = true;
                                break;
                            case CellType.Wall:
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
                CellFrame? cellFrame = GetCell(row, column) as CellFrame;
                if (cellFrame is not null)
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
        public CellType GetCellType(int row, int column)
        {
            if (row >= 1 && row <= RowCount && column >= 1 && column <= ColumnCount)
                return _cellTypes[row - 1, column - 1];
            return CellType.Empty;
        }
        /// <summary>
        /// Creates the maze cell content for a given location. If the grid is initializing, then cell content
        /// is returned that reflects the content of the maze item (if any) - otherwise, an empty cell is returned.
        /// </summary>
        /// <param name="frame">Container frame</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <param name="gridInitializing">Grid is initializing?</param>
        /// <returns>Maze cell content</returns>
        public override ContentView CreateCellContent(CellFrame frame, int row, int column, bool gridInitializing)
        {
            // Logical model is already populated in Initialize() before InitializeContent() runs
            return new MazeCellContent(gridInitializing ? _cellTypes[row, column] : CellType.Empty);
        }
        /// <summary>
        /// Populates the content of a cell frame from the logical model.
        /// Called by the base class whenever a cell enters the visible viewport.
        /// Row and column are 0-based.
        /// </summary>
        protected override void UpdateCellContent(CellFrame frame, int row, int column)
        {
            var type      = _cellTypes[row, column];
            var direction = _solutionDirections[row, column];
            if (frame.Content is MazeCellContent existing)
                existing.Update(type, direction);
            else
            {
                var content = new MazeCellContent(type);
                if (direction != MazeCellContent.PathDirection.None)
                    content.SetSolutionPath(direction);
                frame.Content = content;
            }
        }
        protected override void OnBeforeRowsInserted(int startDisplayRow, int count)
        {
            ResizeLogicalArrayRows(RowCount + count, startDisplayRow - 1, count, true);
            if (_startRow  >= startDisplayRow) _startRow  += count;
            if (_finishRow >= startDisplayRow) _finishRow += count;
        }
        protected override void OnBeforeColumnsInserted(int startDisplayColumn, int count)
        {
            ResizeLogicalArrayCols(ColumnCount + count, startDisplayColumn - 1, count, true);
            if (_startCol  >= startDisplayColumn) _startCol  += count;
            if (_finishCol >= startDisplayColumn) _finishCol += count;
        }
        protected override void OnAfterRowsRemoved(int startDisplayRow, int count)
        {
            ResizeLogicalArrayRows(RowCount, startDisplayRow - 1, count, false);
            int removedEnd = startDisplayRow + count - 1;
            if      (_startRow >= startDisplayRow && _startRow <= removedEnd) _startRow = _startCol = -1;
            else if (_startRow > removedEnd)                                  _startRow -= count;
            if      (_finishRow >= startDisplayRow && _finishRow <= removedEnd) _finishRow = _finishCol = -1;
            else if (_finishRow > removedEnd)                                   _finishRow -= count;
        }
        protected override void OnAfterColumnsRemoved(int startDisplayColumn, int count)
        {
            ResizeLogicalArrayCols(ColumnCount, startDisplayColumn - 1, count, false);
            int removedEnd = startDisplayColumn + count - 1;
            if      (_startCol >= startDisplayColumn && _startCol <= removedEnd) _startRow = _startCol = -1;
            else if (_startCol > removedEnd)                                     _startCol -= count;
            if      (_finishCol >= startDisplayColumn && _finishCol <= removedEnd) _finishRow = _finishCol = -1;
            else if (_finishCol > removedEnd)                                      _finishCol -= count;
        }
        private void ResizeLogicalArrayRows(int newRowCount, int insertIdx, int count, bool insert)
        {
            var newTypes = new CellType[newRowCount, ColumnCount];
            var newDirs  = new MazeCellContent.PathDirection[newRowCount, ColumnCount];

            for (int r = 0; r < insertIdx; r++)
                for (int c = 0; c < ColumnCount; c++)
                { newTypes[r, c] = _cellTypes[r, c]; newDirs[r, c] = _solutionDirections[r, c]; }

            if (insert)
            {
                for (int r = insertIdx; r < RowCount; r++)
                    for (int c = 0; c < ColumnCount; c++)
                    { newTypes[r + count, c] = _cellTypes[r, c]; newDirs[r + count, c] = _solutionDirections[r, c]; }
            }
            else
            {
                for (int r = insertIdx + count; r < newRowCount + count; r++)
                    for (int c = 0; c < ColumnCount; c++)
                    { newTypes[r - count, c] = _cellTypes[r, c]; newDirs[r - count, c] = _solutionDirections[r, c]; }
            }

            _cellTypes = newTypes;
            _solutionDirections = newDirs;
        }
        private void ResizeLogicalArrayCols(int newColCount, int insertIdx, int count, bool insert)
        {
            var newTypes = new CellType[RowCount, newColCount];
            var newDirs  = new MazeCellContent.PathDirection[RowCount, newColCount];

            for (int r = 0; r < RowCount; r++)
            {
                for (int c = 0; c < insertIdx; c++)
                { newTypes[r, c] = _cellTypes[r, c]; newDirs[r, c] = _solutionDirections[r, c]; }

                if (insert)
                {
                    for (int c = insertIdx; c < ColumnCount; c++)
                    { newTypes[r, c + count] = _cellTypes[r, c]; newDirs[r, c + count] = _solutionDirections[r, c]; }
                }
                else
                {
                    for (int c = insertIdx + count; c < newColCount + count; c++)
                    { newTypes[r, c - count] = _cellTypes[r, c]; newDirs[r, c - count] = _solutionDirections[r, c]; }
                }
            }

            _cellTypes = newTypes;
            _solutionDirections = newDirs;
        }
        /// <summary>
        /// Returns the cell type associated with the current maze item for a given location
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <returns>Maze cell type</returns>
        private CellType GetMazeItemCellType(int row, int column)
        {
            return this.mazeItem?.Definition?.GetCellType((uint)row, (uint)column) ?? CellType.Empty;
        }
        /// <summary>
        /// Handles the cell tapped event
        /// </summary>
        /// <param name="cellFrame">Cell frame</param>
        /// <param name="triggerEvents">Flag indicating whether to trigger further events</param>
        public override void OnCellTapped(CellFrame cellFrame, bool triggerEvents)
        {
            if (triggerEvents && CellTapped is not null)
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
        public override void OnCellDoubleTapped(CellFrame cellFrame, bool triggerEvents)
        {
            if (triggerEvents && CellDoubleTapped is not null)
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
        public override void OnProcessKeyDown(Controls.Keyboard.KeyState state, Controls.Keyboard.Key key, bool triggerEvents)
        {
            if (triggerEvents && KeyDown is not null)
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
        public void SetSelectionContent(CellType cellType)
        {
            switch (cellType)
            {
                case CellType.Start:
                    SetSelectionToStartCell();
                    break;
                case CellType.Finish:
                    SetSelectionToFinishCell();
                    break;

                case CellType.Wall:
                case CellType.Empty:
                    SetSelectionContentToType(cellType);
                    break;
            }
        }
        /// <summary>
        /// Sets the content in the selected cells to be the start cell
        /// </summary>
        private void SetSelectionToStartCell()
        {
            CellRange? currentSelection = CurrentSelection;
            if (currentSelection is not null && currentSelection.IsSingleCell)
            {
                int selRow = currentSelection.Top, selCol = currentSelection.Left;
                if (_startRow == selRow && _startCol == selCol) return;
                if (_startRow > 0) SetCellContent(_startRow, _startCol, CellType.Empty);
                SetCellContent(selRow, selCol, CellType.Start);
                // Clear finish if it occupies the same cell
                if (_finishRow == selRow && _finishCol == selCol) { _finishRow = _finishCol = -1; }
            }
        }
        /// <summary>
        /// Sets the content in the selected cells to be the finish cell
        /// </summary>
        private void SetSelectionToFinishCell()
        {
            CellRange? currentSelection = CurrentSelection;
            if (currentSelection is not null && currentSelection.IsSingleCell)
            {
                int selRow = currentSelection.Top, selCol = currentSelection.Left;
                if (_finishRow == selRow && _finishCol == selCol) return;
                if (_finishRow > 0) SetCellContent(_finishRow, _finishCol, CellType.Empty);
                SetCellContent(selRow, selCol, CellType.Finish);
                // Clear start if it occupies the same cell
                if (_startRow == selRow && _startCol == selCol) { _startRow = _startCol = -1; }
            }
        }
        /// <summary>
        /// Sets the content in the selected cells to be a given type, providing it is not a start or finish cell type
        /// </summary>
        /// <param name="cellType">Cell type</param>
        private void SetSelectionContentToType(CellType cellType)
        {
            CellRange? currentSelection = CurrentSelection;
            if (currentSelection is not null && cellType != CellType.Start && cellType != CellType.Finish)
            {
                for (int row = currentSelection.Top; row <= currentSelection.Bottom; row++)
                {
                    for (int column = currentSelection.Left; column <= currentSelection.Right; column++)
                        SetCellContent(row, column, cellType);
                }
                if (_startRow  > 0 && currentSelection.ContainsPosition(_startRow,  _startCol))  { _startRow  = _startCol  = -1; }
                if (_finishRow > 0 && currentSelection.ContainsPosition(_finishRow, _finishCol)) { _finishRow = _finishCol = -1; }
            }
        }
        /// <summary>
        /// Sets the content in the cell location to be a given type
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <param name="cellType">Cell type</param>
        /// <returns>Cell frame</returns>
        private CellFrame? SetCellContent(int row, int column, CellType cellType)
        {
            if (row >= 1 && row <= RowCount && column >= 1 && column <= ColumnCount)
            {
                _cellTypes[row - 1, column - 1] = cellType;
                if      (cellType == CellType.Start)                              { _startRow  = row;  _startCol  = column; }
                else if (cellType == CellType.Finish)                             { _finishRow = row;  _finishCol = column; }
                else if (_startRow  == row && _startCol  == column) _startRow  = _startCol  = -1;
                else if (_finishRow == row && _finishCol == column) _finishRow = _finishCol = -1;
            }
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame is not null)
                SetCellContent(cellFrame, new MazeCellContent(cellType));
            return cellFrame;
        }
        /// <summary>
        /// Converts the maze grid content to a `Maze` object
        /// </summary>
        /// <returns>Maze object</returns>
        public Api.Maze ToMaze()
        {
            Api.Maze maze = new Api.Maze((uint)RowCount, (uint)ColumnCount);

            for (int row = 0; row < RowCount; row++)
            {
                for (int column = 0; column < ColumnCount; column++)
                {
                    switch (_cellTypes[row, column])
                    {
                        case CellType.Start:  maze.SetStartCell((uint)row, (uint)column);                              break;
                        case CellType.Finish: maze.SetFinishCell((uint)row, (uint)column);                             break;
                        case CellType.Wall:   maze.SetWallCells((uint)row, (uint)column, (uint)row, (uint)column);     break;
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
        public bool DisplaySolution(Api.Solution solution)
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
                nextPoint = i + 1 < points.Count ? points[i + 1] : null;
                thisCellDirection = GetCellPathDirection(prevCellDirection, thisPoint, nextPoint);
                SetSolutionCell((int)thisPoint.Row + 1, (int)thisPoint.Column + 1, thisCellDirection);
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

            if (nextCellPoint is not null)
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
        /// <param name="prevDirection">Previous cell direction</param>
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
        /// Animates a walker character stepping through the given solution path one cell at a time.
        /// Each visited cell receives a footstep overlay as the walker moves on. When the walk
        /// On successful completion the celebrate GIF remains visible until the caller clears the
        /// solution. On cancellation the walker cell is cleaned up and the partial walk is left for
        /// the caller to clear via <see cref="ClearLastSolution"/>.
        /// </summary>
        /// <param name="solution">Maze solution</param>
        /// <param name="getStepMs">Returns the milliseconds to wait between steps; read at each step so speed changes take effect immediately</param>
        /// <param name="ct">Cancellation token</param>
        public async Task WalkSolutionAsync(Api.Solution solution, Func<int> getStepMs, CancellationToken ct)
        {
            List<Api.Maze.Point> points = solution.GetPathPoints();
            if (points.Count == 0) return;

            // Pre-compute per-cell footstep directions (same logic as DisplaySolution)
            var directions = new MazeCellContent.PathDirection[points.Count];
            MazeCellContent.PathDirection prevDir = MazeCellContent.PathDirection.None;
            for (int i = 0; i < points.Count; i++)
            {
                Api.Maze.Point? next = i + 1 < points.Count ? points[i + 1] : null;
                directions[i] = GetCellPathDirection(prevDir, points[i], next);
                prevDir = directions[i];
            }

            try
            {
                for (int i = 0; i < points.Count; i++)
                {
                    ct.ThrowIfCancellationRequested();

                    int row = (int)points[i].Row + 1;
                    int col = (int)points[i].Column + 1;
                    bool isLast = i == points.Count - 1;

                    string walkerImage = isLast
                        ? "walker_celebrate.gif"
                        : GetWalkerDirectionImage(points[i], points[i + 1]);

                    SetWalkerCell(row, col, walkerImage);

                    // Mark the previous cell with its footstep overlay
                    if (i > 0)
                        SetSolutionCell((int)points[i - 1].Row + 1, (int)points[i - 1].Column + 1, directions[i - 1]);

                    ScrollCellIntoView(row, col);

                    await Task.Delay(getStepMs(), ct);
                }
                // Walk completed — celebrate GIF stays visible until Clear Solution is pressed
            }
            catch (OperationCanceledException)
            {
                ClearWalkerCell();
                throw;
            }
        }
        /// <summary>
        /// Returns the walker GIF filename for movement from one point to the next
        /// </summary>
        private static string GetWalkerDirectionImage(Api.Maze.Point from, Api.Maze.Point to)
        {
            if (to.Row    < from.Row)    return "walker_up.gif";
            if (to.Row    > from.Row)    return "walker_down.gif";
            if (to.Column < from.Column) return "walker_left.gif";
            return "walker_right.gif";
        }
        /// <summary>
        /// Moves the walker visual to the given cell, restoring the previous walker cell to its normal state
        /// </summary>
        private void SetWalkerCell(int row, int col, string walkerImage)
        {
            // Restore previous walker cell
            if (_walkerRow > 0)
            {
                MazeCellContent? prev = GetCellContent(_walkerRow, _walkerCol);
                prev?.Update(_cellTypes[_walkerRow - 1, _walkerCol - 1], _solutionDirections[_walkerRow - 1, _walkerCol - 1]);
            }
            _walkerRow = row;
            _walkerCol = col;
            GetCellContent(row, col)?.SetWalker(walkerImage);
        }
        /// <summary>
        /// Clears the walker visual and restores the cell to its normal state
        /// </summary>
        private void ClearWalkerCell()
        {
            if (_walkerRow > 0)
            {
                MazeCellContent? content = GetCellContent(_walkerRow, _walkerCol);
                content?.Update(_cellTypes[_walkerRow - 1, _walkerCol - 1], _solutionDirections[_walkerRow - 1, _walkerCol - 1]);
                _walkerRow = -1;
                _walkerCol = -1;
            }
        }
        /// <summary>
        /// Clears the last displayed solution
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ClearLastSolution()
        {
            ClearWalkerCell();
            if (haveSolutionCells)
            {
                Array.Clear(_solutionDirections, 0, _solutionDirections.Length);
                // Refresh all visible cells so the solution overlay is removed
                for (int row = 1; row <= RowCount; row++)
                {
                    for (int column = 1; column <= ColumnCount; column++)
                    {
                        MazeCellContent? cellContent = GetCellContent(row, column);
                        if (cellContent is not null)
                            cellContent.SetSolutionPath(MazeCellContent.PathDirection.None);
                    }
                }
                haveSolutionCells = false;
            }
            return true;
        }
        /// <summary>
        /// Sets a solution cell direction in the logical model and updates the visible frame (if any)
        /// </summary>
        private void SetSolutionCell(int row, int column, MazeCellContent.PathDirection direction)
        {
            if (row >= 1 && row <= RowCount && column >= 1 && column <= ColumnCount)
            {
                _solutionDirections[row - 1, column - 1] = direction;
                if (direction != MazeCellContent.PathDirection.None)
                    haveSolutionCells = true;
                MazeCellContent? cellContent = GetCellContent(row, column);
                if (cellContent is not null)
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
        public CellFrame Cell { get; }
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
        public MazeGridCellTappedEventArgs(CellFrame cellFrame, int numberTaps)
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
        Controls.Keyboard.KeyState keyState = Controls.Keyboard.KeyState.None;
        Controls.Keyboard.Key key = Controls.Keyboard.Key.None;

        /// <summary>
        /// Additional key state information
        /// </summary>
        /// <returns>Key state</returns>
        public Controls.Keyboard.KeyState KeyState { get => keyState; }
        /// <summary>
        /// Key that was pressed
        /// </summary>
        /// <returns>Key</returns>
        public Controls.Keyboard.Key Key { get => key; }
        /// <summary>
        /// Indicates whether the shift key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsShiftKeyPressed { get => Controls.Keyboard.Utility.IsStateFlagSet(KeyState, Controls.Keyboard.KeyState.Shift); }
        /// <summary>
        /// Indicates whether the Ctrl key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsCtrlKeyPressed { get => Controls.Keyboard.Utility.IsStateFlagSet(KeyState, Controls.Keyboard.KeyState.Ctrl); }
        /// <summary>
        /// Indicates whether the Caps Lock key was down at the time the key was pressed
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsCapsLockKeyPressed { get => Controls.Keyboard.Utility.IsStateFlagSet(KeyState, Controls.Keyboard.KeyState.CapsLock); }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="keyState">Additional key state information</param>
        /// <param name="key">Key that was pressed</param>
        public MazeGridKeyDownEventArgs(Controls.Keyboard.KeyState keyState, Controls.Keyboard.Key key)
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
            /// <returns>No direction</returns>
            None = 0,
            /// <summary>
            /// To left
            /// </summary>
            /// <returns>To left</returns>
            Left = 1,
            /// <summary>
            /// To left from down
            /// </summary>
            /// <returns>To left from down</returns>
            LeftFromDown = 2,
            /// <summary>
            /// To left from up
            /// </summary>
            /// <returns>To left from up</returns>
            LeftFromUp = 3,
            /// <summary>
            /// To right
            /// </summary>
            /// <returns>To right</returns>
            Right = 4,
            /// <summary>
            /// To right from down
            /// </summary>
            /// <returns>To right from down</returns>
            RightFromDown = 5,
            /// <summary>
            /// To right from up
            /// </summary>
            /// <returns>To right from up</returns>
            RightFromUp = 6,
            /// <summary>
            /// Upwards
            /// </summary>
            /// <returns>Upwards</returns>
            Up = 7,
            /// <summary>
            /// Upwards from left
            /// </summary>
            /// <returns>Upwards from left</returns>
            UpFromLeft = 8,
            /// <summary>
            /// Upwards from right
            /// </summary>
            /// <returns>Upwards from right</returns>
            UpFromRight = 9,
            /// <summary>
            /// Downwards
            /// </summary>
            /// <returns>Downwards</returns>
            Down = 10,
            /// <summary>
            /// Downwards from left
            /// </summary>
            /// <returns>Downwards from left</returns>
            DownFromLeft = 11,
            /// <summary>
            /// Downwards from right
            /// </summary>
            /// <returns>Downwards from right</returns>
            DownFromRight = 12
        }

        static private Color SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR = Colors.White;
        static private Color SOLUTION_PATH_CELL_HIGHLIGHT_COLOR = Colors.LightGreen;

        CellType cellType = CellType.Empty;
        PathDirection solutionPathDirection = PathDirection.None;

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
        public CellType CellType { get => cellType; }
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
        public MazeCellContent(CellType cellType)
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
                case CellType.Start:
                    return preferFlag ? "start_flag.png" : "start_sign.png";
                case CellType.Finish:
                    return preferFlag ? "finish_flag.png" : "finish_sign.png";
                case CellType.Wall:
                    return "wall.png";
            }
            return "";
        }
        /// <summary>
        /// Updates the cell content in-place, reusing the existing Image or Label where possible
        /// to avoid a new async image-load cycle on pool-recycled frames.
        /// </summary>
        /// <param name="newCellType">New cell type</param>
        /// <param name="newDirection">New solution path direction</param>
        public void Update(CellType newCellType, PathDirection newDirection)
        {
            cellType = newCellType;
            solutionPathDirection = newDirection;

            bool needsImage = cellType != CellType.Empty || solutionPathDirection != PathDirection.None;
            string? source = needsImage
                ? (cellType == CellType.Empty ? GetSolutionPathImage() : GetImageName(true))
                : null;

            if (needsImage)
            {
                if (Content is Image img)
                    img.Source = source;
                else
                    Content = new Image
                    {
                        Source = source,
                        Aspect = Aspect.AspectFit,
                        HorizontalOptions = LayoutOptions.Fill,
                        VerticalOptions = LayoutOptions.Fill
                    };
                Content.BackgroundColor = GetSolutionPathHighlightColor();
            }
            else
            {
                if (Content is not Label)
                    Content = new Label();
                Content.BackgroundColor = Colors.Transparent;
            }
        }
        /// <summary>
        /// Displays a walker GIF in this cell, overriding the normal cell content
        /// </summary>
        /// <param name="source">Image filename (e.g. "walker_down.gif")</param>
        public void SetWalker(string source)
        {
            if (Content is Image img)
            {
                img.Source = source;
                img.IsAnimationPlaying = true;
            }
            else
                Content = new Image
                {
                    Source = source,
                    Aspect = Aspect.AspectFit,
                    HorizontalOptions = LayoutOptions.Fill,
                    VerticalOptions = LayoutOptions.Fill,
                    IsAnimationPlaying = true
                };
            Content.BackgroundColor = Colors.Transparent;
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
                ? IsStartOrFinish ? SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR : SOLUTION_PATH_CELL_HIGHLIGHT_COLOR
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
