using static Maze.Api.Maze;
using Maze.Api;
using Maze.Maui.App.Services;
using Maze.Maui.Controls.InteractiveGrid;
using Maze.Maui.App.Models;
using Maze.Maui.Controls.Keyboard;

namespace Maze.Maui.App
{
    /// <summary>
    /// The `MazeGrid` class represents an interactive maze grid
    /// </summary>
    public class MazeGrid : Controls.InteractiveGrid.Grid, IMazeGridView, ICellOverrideEditor
    {
        private const int DEFAULT_ROW_COUNT = 5;
        private const int DEFAULT_COLUMN_COUNT = 5;
        private MazeItem? mazeItem;
        private bool haveSolutionCells = false;

        // Logical cell state (independent of the visual tree — required for virtualization)
        private CellType[,] _cellTypes = new CellType[0, 0];
        private MazeCellContent.PathDirection[,] _solutionDirections = new MazeCellContent.PathDirection[0, 0];
        // Per-cell editor overrides (non-default characteristics layered on a cell),
        // held alongside _cellTypes and kept aligned with it: structural edits remap
        // the keys, a rewritten cell drops its override, and the survivors are stamped
        // onto the maze in ToMaze().
        private readonly CellOverrides _overrides = new();
        // Game-mode runtime overrides for key/door cells (consulted by CreateCellContent /
        // UpdateCellContent so virtualized recycling re-applies the latest state).
        // _keyCollected[r,c] == true → the key at (r,c) has been picked up; hide its icon.
        // _doorRuntimeState[r,c] holds the current DoorState for that door cell (default Locked).
        private bool[,] _keyCollected = new bool[0, 0];
        private DoorState[,] _doorRuntimeState = new DoorState[0, 0];
        // Game-mode runtime overrides for enemy/health cells. Only consulted while
        // _gameMode is true (a live game session); in editor mode the static 'E' /
        // 'H' cells render as-is. _enemyAt[r,c] = count of live enemies on that cell
        // (the static spawn 'E' is suppressed and the live position rendered instead).
        // _healthCollected[r,c] == true → the pickup was consumed; hide its icon.
        private bool _gameMode;
        private int[,] _enemyAt = new int[0, 0];
        // The visual rigs (ghost vs the default goblin) of the enemies on each cell. A list
        // (not a single value) so a cell shared by enemies of different types resolves each
        // one's sprite correctly when they separate; the first entry is the rig shown for a
        // stack. Seeded from spawn overrides; each enemy carries its own rig as it moves.
        private List<EnemyType?>?[,] _enemyRigsAt = new List<EnemyType?>?[0, 0];
        private bool[,] _healthCollected = new bool[0, 0];
        // _treasureCollected[r,c] == true → the treasure at (r,c) was auto-collected on
        // walk-over; hide its icon.
        private bool[,] _treasureCollected = new bool[0, 0];
        // 1-based positions of start/finish cells (-1 = not set)
        private int _startRow = -1, _startCol = -1;
        private int _finishRow = -1, _finishCol = -1;
        // 1-based position of the current walker cell (-1 = no walker)
        private int _walkerRow = -1, _walkerCol = -1;
        // Current player walker GIF source, so cells that recycle into view (or an
        // enemy stacking onto the player's cell) can re-render the player on top.
        private string _walkerImage = "walker_down.gif";

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
            RowCount = (int)(mazeItem?.Definition?.RowCount ?? DEFAULT_ROW_COUNT);
            ColumnCount = (int)(mazeItem?.Definition?.ColCount ?? DEFAULT_COLUMN_COUNT);

            // Populate the logical cell-type model before building the visual layer
            _cellTypes = new CellType[RowCount, ColumnCount];
            _solutionDirections = new MazeCellContent.PathDirection[RowCount, ColumnCount];
            _keyCollected = new bool[RowCount, ColumnCount];
            _doorRuntimeState = new DoorState[RowCount, ColumnCount];
            _enemyAt = new int[RowCount, ColumnCount];
            _enemyRigsAt = new List<EnemyType?>?[RowCount, ColumnCount];
            _healthCollected = new bool[RowCount, ColumnCount];
            _treasureCollected = new bool[RowCount, ColumnCount];
            _gameMode = false;
            _startRow = _startCol = _finishRow = _finishCol = -1;

            for (int r = 0; r < RowCount; r++)
            {
                for (int c = 0; c < ColumnCount; c++)
                {
                    _cellTypes[r, c] = GetMazeItemCellType(r, c);
                    if (_cellTypes[r, c] == CellType.Start) { _startRow = r + 1; _startCol = c + 1; }
                    else if (_cellTypes[r, c] == CellType.Finish) { _finishRow = r + 1; _finishCol = c + 1; }
                }
            }

            // Load any per-cell overrides off the source definition. Only overridable
            // cell types can carry one, so the rest are skipped without an FFI hop.
            _overrides.Clear();
            Api.Maze? definition = this.mazeItem?.Definition;
            if (definition is not null)
            {
                for (int r = 0; r < RowCount; r++)
                {
                    for (int c = 0; c < ColumnCount; c++)
                    {
                        if (!IsOverridableType(_cellTypes[r, c]))
                        {
                            continue;
                        }
                        CellEntityInfo? entity = definition.GetCellEntity((uint)r, (uint)c);
                        if (entity is not null)
                        {
                            _overrides.Set(r, c, entity);
                        }
                    }
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
            bool containsKey = false, containsDoor = false;
            bool containsEnemy = false, containsHealth = false, containsTreasure = false;
            int numWalls = 0;
            if (currentSelection is not null)
            {
                cellCount = currentSelection.CellCount;
                singleCell = cellCount == 1;
                for (int row = currentSelection.Top; row <= currentSelection.Bottom; row++)
                {
                    for (int column = currentSelection.Left; column <= currentSelection.Right; column++)
                    {
                        CellType cellType = GetCellType(row, column);
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
                            case CellType.Key:
                                containsKey = true;
                                break;
                            case CellType.Door:
                                containsDoor = true;
                                break;
                            case CellType.Enemy:
                                containsEnemy = true;
                                break;
                            case CellType.Health:
                                containsHealth = true;
                                break;
                            case CellType.Treasure:
                                containsTreasure = true;
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
                ContainsKey = containsKey,
                ContainsDoor = containsDoor,
                ContainsEnemy = containsEnemy,
                ContainsHealth = containsHealth,
                ContainsTreasure = containsTreasure,
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
            CellType type = gridInitializing ? _cellTypes[row, column] : CellType.Empty;
            var content = new MazeCellContent(type, OverrideForRender(type, row, column), mazeItem?.GameSettings, showBadge: !_gameMode);
            ApplyGameRuntimeState(content, row, column);
            return content;
        }
        /// <summary>
        /// Provides the label content for header cells with an explicit black
        /// text colour. The base class default leaves <c>TextColor</c> unset,
        /// so on Windows the platform-native renderer flips the text to the
        /// OS theme's foreground (white in dark mode), which then sits
        /// invisibly on the light-grey header background. Pinning the colour
        /// here is the surgical fix: dark-mode users see the same readable
        /// "black on light grey" header strip as light-mode users (the
        /// spreadsheet idiom most apps follow regardless of theme).
        /// </summary>
        /// <param name="type">Header type</param>
        /// <param name="index">Header cell index</param>
        /// <returns>Header cell label</returns>
        public override View GetHeaderCellContent(HeaderType type, int index)
        {
            return new Label
            {
                Text = type != HeaderType.Corner ? $"{index + 1}" : "",
                FontAttributes = FontAttributes.Bold,
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center,
                TextColor = Colors.Black,
            };
        }
        /// <summary>
        /// Populates the content of a cell frame from the logical model.
        /// Called by the base class whenever a cell enters the visible viewport.
        /// Row and column are 0-based.
        /// </summary>
        protected override void UpdateCellContent(CellFrame frame, int row, int column)
        {
            var type = _cellTypes[row, column];
            var direction = _solutionDirections[row, column];
            CellEntityInfo? entity = OverrideForRender(type, row, column);
            MazeCellContent content;
            if (frame.Content is MazeCellContent existing)
            {
                existing.Update(type, direction, entity, mazeItem?.GameSettings, showBadge: !_gameMode);
                content = existing;
            }
            else
            {
                content = new MazeCellContent(type, entity, mazeItem?.GameSettings, showBadge: !_gameMode);
                if (direction != MazeCellContent.PathDirection.None)
                    content.SetSolutionPath(direction);
                frame.Content = content;
            }
            ApplyGameRuntimeState(content, row, column);
        }
        /// <summary>
        /// The override to render for a cell, or null. Overrides drive the variant sprite
        /// (and, in the editor only, the authoring badge). In game mode only the static
        /// wall (water/lava/iron_fence) and health (potion) variants are surfaced — enemies
        /// are live moving overlays and keys/doors have no 2D variant — so their overrides
        /// are suppressed. Row and column are 0-based.
        /// </summary>
        /// <param name="type">The cell's type</param>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="column">Column index (zero-based)</param>
        /// <returns>The cell's override, or null</returns>
        private CellEntityInfo? OverrideForRender(CellType type, int row, int column)
        {
            if (!IsOverridableType(type))
            {
                return null;
            }
            if (_gameMode && type != CellType.Wall && type != CellType.Health)
            {
                return null;
            }
            return GetCellOverride(row + 1, column + 1);
        }
        /// <summary>
        /// Re-applies any active game-mode runtime state (collected key, door
        /// state) to a cell's content. Called from <see cref="CreateCellContent"/>
        /// and <see cref="UpdateCellContent"/> so a cell that's recycled into
        /// view after a pickup or door event shows the correct visual.
        /// </summary>
        private void ApplyGameRuntimeState(MazeCellContent content, int row, int column)
        {
            if (_keyCollected.GetLength(0) <= row || _keyCollected.GetLength(1) <= column) return;
            // Resolve the static base first (collected key/health and spawn markers reduce
            // to an empty passage; doors carry an opacity), then layer any live enemy and
            // the player walker on top — so the underlying cell (e.g. a potion the enemy is
            // standing on) still shows through and reappears once they leave.
            if (_cellTypes[row, column] == CellType.Key && _keyCollected[row, column])
            {
                // Collected key renders as an empty passage so the visited-dot
                // marker fires correctly on revisit (the visited-dot path in
                // SetSolutionPath only paints the dot for IsEmpty cells).
                content.Update(CellType.Empty, content.SolutionPathDirection);
            }
            else if (_cellTypes[row, column] == CellType.Door)
            {
                switch (_doorRuntimeState[row, column])
                {
                    case DoorState.Opening: content.SetIconOpacity(0.5); break;
                    case DoorState.Open:
                        // Open D also renders as empty passage — same reason as collected K.
                        content.Update(CellType.Empty, content.SolutionPathDirection);
                        break;
                    default: content.SetIconOpacity(1.0); break;
                }
            }
            else if (_gameMode && _cellTypes[row, column] == CellType.Enemy)
            {
                // A spawn 'E' cell with no live enemy on it — the enemy has walked
                // away, so the static spawn marker is suppressed.
                content.Update(CellType.Empty, content.SolutionPathDirection);
            }
            else if (_gameMode && _cellTypes[row, column] == CellType.Health && _healthCollected[row, column])
            {
                // Consumed health pickup renders as an empty passage. The runtime
                // auto-consumes the cell but the static grid char never changes.
                content.Update(CellType.Empty, content.SolutionPathDirection);
            }
            else if (_gameMode && _cellTypes[row, column] == CellType.Treasure && _treasureCollected[row, column])
            {
                // Collected treasure renders as an empty passage — auto-collected on
                // walk-over while the static grid char stays 'T' (same as a key / pickup).
                content.Update(CellType.Empty, content.SolutionPathDirection);
            }

            bool playerHere = _walkerRow - 1 == row && _walkerCol - 1 == column;
            if (_gameMode && (_enemyAt[row, column] > 0 || playerHere))
            {
                content.SetEntityOverlay(_enemyAt[row, column], playerHere ? _walkerImage : null, StackEnemyRig(row, column));
            }
        }
        /// <summary>
        /// Enters game-runtime mode: enemy / health cells thereafter follow live
        /// game state rather than the static grid. Seeds the live-enemy positions
        /// from the spawn (<c>'E'</c>) cells. Called by the game view-model once a
        /// session has started.
        /// </summary>
        public void BeginGameRuntime()
        {
            _gameMode = true;
            for (int r = 0; r < RowCount; r++)
            {
                for (int c = 0; c < ColumnCount; c++)
                {
                    if (_cellTypes[r, c] == CellType.Enemy)
                    {
                        _enemyAt[r, c] = 1;
                        (_enemyRigsAt[r, c] ??= new List<EnemyType?>()).Add((GetCellOverride(r + 1, c + 1) as EnemyCellEntity)?.EnemyType);
                    }
                }
            }
            // Initialize rendered the cells in editor mode before the flag flipped, so
            // re-render the ones already on screen to pick up game-mode rendering (variant
            // walls/health without the authoring badge, suppressed spawn markers). Cells
            // that scroll in later already render in game mode.
            foreach (KeyValuePair<(int row, int col), CellFrame> entry in GetActiveCells())
            {
                UpdateCellContent(entry.Value, entry.Key.row, entry.Key.col);
            }
        }
        /// <summary>
        /// Moves the live enemy visual from its old cell to its new cell. The spawn
        /// <c>'E'</c> marker is suppressed in game mode, so enemies are tracked by a
        /// per-cell occupancy count rather than the static grid. Pass
        /// <c>oldRow</c> / <c>oldCol</c> of <c>-1</c> for an enemy that has no prior
        /// cell (initial placement).
        /// </summary>
        /// <param name="oldRow">Previous row (0-based), or -1 for none.</param>
        /// <param name="oldCol">Previous column (0-based), or -1 for none.</param>
        /// <param name="newRow">New row (0-based).</param>
        /// <param name="newCol">New column (0-based).</param>
        /// <param name="id">Stable enemy id (unused by the count-based model; kept for caller clarity).</param>
        /// <param name="enemyType">This enemy's own visual rig (ghost vs the default goblin), or null.</param>
        public void SetEnemyCell(int oldRow, int oldCol, int newRow, int newCol, uint id, EnemyType? enemyType)
        {
            _ = id;
            if (!_gameMode) return;
            if (oldRow >= 0 && oldCol >= 0 && oldRow < RowCount && oldCol < ColumnCount && _enemyAt[oldRow, oldCol] > 0)
            {
                // Remove this enemy's own rig from the old cell — so a cell shared by
                // differing enemies keeps the remaining one's rig, and a swap with a
                // neighbour can't carry the wrong rig.
                _enemyAt[oldRow, oldCol]--;
                _enemyRigsAt[oldRow, oldCol]?.Remove(enemyType);
                RefreshCellRuntime(oldRow, oldCol);
            }
            if (newRow >= 0 && newCol >= 0 && newRow < RowCount && newCol < ColumnCount)
            {
                _enemyAt[newRow, newCol]++;
                (_enemyRigsAt[newRow, newCol] ??= new List<EnemyType?>()).Add(enemyType);
                RefreshCellRuntime(newRow, newCol);
            }
        }
        /// <summary>
        /// The rig shown for a (possibly stacked) enemy cell — a distinctive rig (ghost)
        /// takes priority over the default goblin so a mixed stack surfaces the special
        /// enemy (see <see cref="CellSprite.DominantEnemyRig"/>).
        /// </summary>
        private EnemyType? StackEnemyRig(int row, int col) =>
            _enemyRigsAt[row, col] is { Count: > 0 } list ? CellSprite.DominantEnemyRig(list) : null;
        /// <summary>
        /// Marks the health pickup at the given 0-based cell as consumed — the icon
        /// disappears (mirrors <see cref="MarkKeyCollected"/> for <c>'H'</c> cells).
        /// </summary>
        public void MarkHealthCollected(int row, int col)
        {
            if (row < 0 || col < 0 || row >= _healthCollected.GetLength(0) || col >= _healthCollected.GetLength(1)) return;
            _healthCollected[row, col] = true;
            RefreshCellRuntime(row, col);
        }
        /// <summary>
        /// Marks the treasure at the given 0-based cell as collected — the icon
        /// disappears (mirrors <see cref="MarkKeyCollected"/> for <c>'T'</c> cells).
        /// </summary>
        public void MarkTreasureCollected(int row, int col)
        {
            if (row < 0 || col < 0 || row >= _treasureCollected.GetLength(0) || col >= _treasureCollected.GetLength(1)) return;
            _treasureCollected[row, col] = true;
            RefreshCellRuntime(row, col);
        }
        /// <summary>
        /// Rebuilds a cell's content from its static type (and per-cell override) plus the
        /// current game runtime state — the base cell is restored, then any live enemy and
        /// the player walker are layered back over it by <see cref="ApplyGameRuntimeState"/>.
        /// </summary>
        private void RefreshCellRuntime(int row, int col)
        {
            MazeCellContent? content = GetCellContent(row + 1, col + 1);
            if (content is null) return;
            content.Update(_cellTypes[row, col], _solutionDirections[row, col],
                OverrideForRender(_cellTypes[row, col], row, col), mazeItem?.GameSettings, showBadge: !_gameMode);
            ApplyGameRuntimeState(content, row, col);
        }
        /// <summary>
        /// Re-renders the base sprite of every realized wall / enemy / health cell from the
        /// maze's current game settings, so the editor grid reflects a changed wall / enemy /
        /// health default. Off-screen (virtualized) cells are not realized; they pick up the
        /// new settings when they scroll into view via <see cref="UpdateCellContent"/>.
        /// </summary>
        public void RefreshGameSettingsBaseSprites()
        {
            for (int row = 0; row < RowCount; row++)
            {
                for (int col = 0; col < ColumnCount; col++)
                {
                    CellType type = _cellTypes[row, col];
                    if (type is not (CellType.Wall or CellType.Enemy or CellType.Health))
                    {
                        continue;
                    }
                    MazeCellContent? content = GetCellContent(row + 1, col + 1);
                    content?.Update(type, _solutionDirections[row, col],
                        OverrideForRender(type, row, col), mazeItem?.GameSettings, showBadge: !_gameMode);
                }
            }
        }
        /// <summary>
        /// Marks the key at the given 0-based cell as collected — the cell
        /// thereafter renders as an empty passage (icon gone, visited markers
        /// work).
        /// </summary>
        public void MarkKeyCollected(int row, int col)
        {
            if (row < 0 || col < 0 || row >= _keyCollected.GetLength(0) || col >= _keyCollected.GetLength(1)) return;
            _keyCollected[row, col] = true;
            // Rebuild the cell now (as with a collected health pickup) so the key
            // disappears immediately. The player walker is layered back over the empty
            // passage by ApplyGameRuntimeState, so refreshing no longer hides the player.
            RefreshCellRuntime(row, col);
        }
        /// <summary>
        /// Updates the runtime visual state for the door at the given 0-based cell.
        /// </summary>
        public void SetDoorRuntimeState(int row, int col, DoorState state)
        {
            if (row < 0 || col < 0 || row >= _doorRuntimeState.GetLength(0) || col >= _doorRuntimeState.GetLength(1)) return;
            _doorRuntimeState[row, col] = state;
            MazeCellContent? content = GetCellContent(row + 1, col + 1);
            if (content is null) return;
            // Walker-on-cell check: an Open door's transition would hide the
            // walker if we replaced Content. The door can only become Open via
            // a Tick that runs while the player is *not* on the door cell
            // (Opening requires StartedUnlocking, which doesn't move the
            // player), so in practice this guard fires only for tests that
            // call SetDoorRuntimeState directly.
            if (state == DoorState.Open && _walkerRow - 1 == row && _walkerCol - 1 == col) return;
            switch (state)
            {
                case DoorState.Opening: content.SetIconOpacity(0.5); break;
                case DoorState.Open: content.Update(CellType.Empty, content.SolutionPathDirection); break;
                default: content.SetIconOpacity(1.0); break;
            }
        }
        protected override void OnBeforeRowsInserted(int startDisplayRow, int count)
        {
            ResizeLogicalArrayRows(RowCount + count, startDisplayRow - 1, count, true);
            if (_startRow >= startDisplayRow) _startRow += count;
            if (_finishRow >= startDisplayRow) _finishRow += count;
        }
        protected override void OnBeforeColumnsInserted(int startDisplayColumn, int count)
        {
            ResizeLogicalArrayCols(ColumnCount + count, startDisplayColumn - 1, count, true);
            if (_startCol >= startDisplayColumn) _startCol += count;
            if (_finishCol >= startDisplayColumn) _finishCol += count;
        }
        protected override void OnAfterRowsRemoved(int startDisplayRow, int count)
        {
            ResizeLogicalArrayRows(RowCount, startDisplayRow - 1, count, false);
            int removedEnd = startDisplayRow + count - 1;
            if (_startRow >= startDisplayRow && _startRow <= removedEnd) _startRow = _startCol = -1;
            else if (_startRow > removedEnd) _startRow -= count;
            if (_finishRow >= startDisplayRow && _finishRow <= removedEnd) _finishRow = _finishCol = -1;
            else if (_finishRow > removedEnd) _finishRow -= count;
        }
        protected override void OnAfterColumnsRemoved(int startDisplayColumn, int count)
        {
            ResizeLogicalArrayCols(ColumnCount, startDisplayColumn - 1, count, false);
            int removedEnd = startDisplayColumn + count - 1;
            if (_startCol >= startDisplayColumn && _startCol <= removedEnd) _startRow = _startCol = -1;
            else if (_startCol > removedEnd) _startCol -= count;
            if (_finishCol >= startDisplayColumn && _finishCol <= removedEnd) _finishRow = _finishCol = -1;
            else if (_finishCol > removedEnd) _finishCol -= count;
        }
        private void ResizeLogicalArrayRows(int newRowCount, int insertIdx, int count, bool insert)
        {
            var newTypes = new CellType[newRowCount, ColumnCount];
            var newDirs = new MazeCellContent.PathDirection[newRowCount, ColumnCount];

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
            // Keep overrides aligned with the same row shift the cells just took.
            if (insert) { _overrides.InsertRows(insertIdx, count); }
            else { _overrides.DeleteRows(insertIdx, count); }
            // Game-mode runtime arrays only matter for an active game session, which
            // never enters this editor-only resize path. Reset to fresh defaults so
            // the next Initialize() / game start sees a clean slate at the new size.
            _keyCollected = new bool[newRowCount, ColumnCount];
            _doorRuntimeState = new DoorState[newRowCount, ColumnCount];
        }
        private void ResizeLogicalArrayCols(int newColCount, int insertIdx, int count, bool insert)
        {
            var newTypes = new CellType[RowCount, newColCount];
            var newDirs = new MazeCellContent.PathDirection[RowCount, newColCount];

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
            // Keep overrides aligned with the same column shift the cells just took.
            if (insert) { _overrides.InsertCols(insertIdx, count); }
            else { _overrides.DeleteCols(insertIdx, count); }
            // See ResizeLogicalArrayRows: game-mode arrays reset to defaults here.
            _keyCollected = new bool[RowCount, newColCount];
            _doorRuntimeState = new DoorState[RowCount, newColCount];
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
                case CellType.Key:
                case CellType.Door:
                case CellType.Enemy:
                case CellType.Health:
                case CellType.Treasure:
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
                if (_startRow > 0 && currentSelection.ContainsPosition(_startRow, _startCol)) { _startRow = _startCol = -1; }
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
                // A rewritten cell loses any override it carried (the new character may
                // not even accept the old entity, e.g. an enemy override on a wall).
                _overrides.Remove(row - 1, column - 1);
                if (cellType == CellType.Start) { _startRow = row; _startCol = column; }
                else if (cellType == CellType.Finish) { _finishRow = row; _finishCol = column; }
                else if (_startRow == row && _startCol == column) _startRow = _startCol = -1;
                else if (_finishRow == row && _finishCol == column) _finishRow = _finishCol = -1;
            }
            CellFrame? cellFrame = GetCell(row, column) as CellFrame;
            if (cellFrame is not null)
                Controls.InteractiveGrid.Grid.SetCellContent(cellFrame, new MazeCellContent(cellType));
            return cellFrame;
        }
        /// <summary>
        /// The maze's game settings (the wall/enemy/health defaults the override panel
        /// inherits for a non-overridden cell), or null when unset.
        /// </summary>
        public MazeGameSettings? GameSettings => mazeItem?.GameSettings;
        /// <summary>
        /// Gets the per-cell override on a cell (its non-default characteristics), or
        /// null when the cell carries none.
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        /// <returns>The cell's override, or null</returns>
        public CellEntityInfo? GetCellOverride(int row, int column) => _overrides.Get(row - 1, column - 1);
        /// <summary>
        /// Sets the per-cell override on a cell. The caller is responsible for the
        /// entity type matching the cell's current type.
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        /// <param name="entity">The override to apply</param>
        public void SetCellOverride(int row, int column, CellEntityInfo entity) => _overrides.Set(row - 1, column - 1, entity);
        /// <summary>
        /// Clears the per-cell override on a cell, if any.
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        public void ClearCellOverride(int row, int column) => _overrides.Remove(row - 1, column - 1);
        /// <summary>
        /// Whether a cell carries a per-cell override.
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        /// <returns>True when the cell has an override</returns>
        public bool HasCellOverride(int row, int column) => _overrides.Has(row - 1, column - 1);
        /// <summary>
        /// Re-renders a cell from the current model (its type and override) so a change
        /// to its override shows immediately. A no-op when the cell is off-screen — it
        /// re-renders from the model when it next scrolls into view.
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        public void RefreshCellContent(int row, int column)
        {
            if (row < 1 || row > RowCount || column < 1 || column > ColumnCount)
            {
                return;
            }
            if (GetCell(row, column) is CellFrame frame)
            {
                UpdateCellContent(frame, row - 1, column - 1);
            }
        }
        /// <summary>
        /// Scrolls the grid so the given cell is within the visible viewport — used to
        /// keep a selected cell visible after the override panel shrinks the grid. A
        /// no-op until the grid is laid out (scrolling a zero-sized viewport mis-computes
        /// the jump and can leave the busy cursor set).
        /// </summary>
        /// <param name="row">Row index (one-based)</param>
        /// <param name="column">Column index (one-based)</param>
        public void EnsureCellVisible(int row, int column)
        {
            if (Width > 0 && Height > 0)
            {
                ScrollCellIntoView(row, column);
            }
        }
        /// <summary>
        /// Whether a cell of the given type can carry a per-cell override (S/F and
        /// empty cells cannot).
        /// </summary>
        /// <param name="type">Cell type</param>
        /// <returns>True for overridable cell types</returns>
        private static bool IsOverridableType(CellType type) =>
            type is CellType.Wall or CellType.Key or CellType.Door or CellType.Enemy or CellType.Health or CellType.Treasure;
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
                        case CellType.Start: maze.SetStartCell((uint)row, (uint)column); break;
                        case CellType.Finish: maze.SetFinishCell((uint)row, (uint)column); break;
                        case CellType.Wall: maze.SetWallCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                        case CellType.Key: maze.SetKeyCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                        case CellType.Door: maze.SetDoorCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                        case CellType.Enemy: maze.SetEnemyCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                        case CellType.Health: maze.SetHealthCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                        case CellType.Treasure: maze.SetTreasureCells((uint)row, (uint)column, (uint)row, (uint)column); break;
                    }
                }
            }

            // Stamp the per-cell overrides on top of the now-populated characters.
            // Every override sits on a cell whose character matches its entity type
            // (rewriting a cell drops its override), so the maze accepts each one.
            foreach (KeyValuePair<(int Row, int Col), CellEntityInfo> entry in _overrides.Entries)
            {
                maze.SetCellEntity((uint)entry.Key.Row, (uint)entry.Key.Col, entry.Value);
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

            Api.Maze.Point thisPoint;
            for (int i = 0; i < points.Count; i++)
            {
                thisPoint = points[i];
                Api.Maze.Point? nextPoint = i + 1 < points.Count ? points[i + 1] : null;
                MazeCellContent.PathDirection thisCellDirection = GetCellPathDirection(prevCellDirection, thisPoint, nextPoint);
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
        private static MazeCellContent.PathDirection GetCellPathDirection(
            MazeCellContent.PathDirection prevCellDirection,
            Api.Maze.Point cellPoint,
            Api.Maze.Point? nextCellPoint
        )
        {
            MazeCellContent.PathDirection direction;
            if (nextCellPoint is not null)
            {
                Api.Maze.Point nextPoint = nextCellPoint.Value;

                direction = GetCellOffsetDirection(prevCellDirection, cellPoint, nextPoint);
            }
            else
            {
                direction = MazeGrid.GetContinueDirection(prevCellDirection);
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
        private static MazeCellContent.PathDirection GetCellOffsetDirection(MazeCellContent.PathDirection prevDirection, Api.Maze.Point from, Api.Maze.Point to)
        {
            MazeCellContent.PathDirection direction = MazeCellContent.PathDirection.None;
            bool sameRow = from.Row == to.Row;
            bool sameColumn = from.Column == to.Column;

            if (!(sameRow && sameColumn))
            {
                if (sameColumn)
                {
                    direction = to.Row > from.Row ? MazeGrid.GetDownDirection(prevDirection) : MazeGrid.GetUpDirection(prevDirection);
                }
                if (sameRow)
                {
                    direction = to.Column > from.Column ? MazeGrid.GetRightDirection(prevDirection) : MazeGrid.GetLeftDirection(prevDirection);
                }
            }

            return direction;
        }
        /// <summary>
        /// Gets the up direction to be used following a previous direction
        /// </summary>
        /// <param name="prevDirection">Previous cell direction</param>
        /// <returns>Path direction</returns>
        private static MazeCellContent.PathDirection GetUpDirection(MazeCellContent.PathDirection prevDirection)
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
        private static MazeCellContent.PathDirection GetDownDirection(MazeCellContent.PathDirection prevDirection)
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
        private static MazeCellContent.PathDirection GetLeftDirection(MazeCellContent.PathDirection prevDirection)
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
        private static MazeCellContent.PathDirection GetRightDirection(MazeCellContent.PathDirection prevDirection)
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
        private static MazeCellContent.PathDirection GetContinueDirection(MazeCellContent.PathDirection currentDirection)
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

                    int r = (int)points[i].Row;
                    int c = (int)points[i].Column;
                    bool isLast = i == points.Count - 1;

                    if (isLast)
                        SetPlayerCelebrate(r, c);
                    else
                        SetPlayerAt(r, c, GetMovementDirection(points[i], points[i + 1]));

                    // Mark the previous cell with its footstep overlay
                    if (i > 0)
                        SetFootstepAt((int)points[i - 1].Row, (int)points[i - 1].Column, PathDirectionToCardinal(directions[i - 1]));

                    await Task.Delay(getStepMs(), ct);
                }
                // Walk completed — celebrate GIF stays visible until Clear Solution is pressed
            }
            catch (OperationCanceledException)
            {
                ClearPlayer();
                throw;
            }
        }
        /// <summary>
        /// Moves the walker visual to the given cell, restoring the previous walker cell to its normal state
        /// </summary>
        private void SetWalkerCell(int row, int col, string walkerImage)
        {
            int prevRow = _walkerRow, prevCol = _walkerCol;
            // Advance the walker state first so the previous cell rebuilds as
            // "player no longer here" (an enemy left behind on it reappears).
            _walkerRow = row;
            _walkerCol = col;
            _walkerImage = walkerImage;

            if (prevRow > 0 && (prevRow != row || prevCol != col))
                RefreshCellRuntime(prevRow - 1, prevCol - 1);

            // Render the new cell: the walker is layered over the cell's base content
            // (and any live enemy already standing on it).
            RefreshCellRuntime(row - 1, col - 1);
        }
        /// <summary>
        /// Clears the walker visual and restores the cell to its normal state
        /// </summary>
        private void ClearWalkerCell()
        {
            if (_walkerRow > 0)
            {
                int row = _walkerRow, col = _walkerCol;
                // Clear the player position first so the cell rebuilds as "player no longer
                // here"; RefreshCellRuntime then restores its base content (and override).
                _walkerRow = -1;
                _walkerCol = -1;
                RefreshCellRuntime(row - 1, col - 1);
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
                        cellContent?.SetSolutionPath(MazeCellContent.PathDirection.None);
                    }
                }
                haveSolutionCells = false;
            }
            return true;
        }
        /// <summary>
        /// Places the player sprite at the given 0-based cell facing the given direction.
        /// </summary>
        /// <param name="row">Row index (0-based)</param>
        /// <param name="col">Column index (0-based)</param>
        /// <param name="direction">Facing direction</param>
        public void SetPlayerAt(int row, int col, MazeGameDirection direction)
        {
            string image = direction switch
            {
                MazeGameDirection.Up => "walker_up.gif",
                MazeGameDirection.Down => "walker_down.gif",
                MazeGameDirection.Left => "walker_left.gif",
                MazeGameDirection.Right => "walker_right.gif",
                MazeGameDirection.None => "walker_down.gif",   // forward-facing before first move
                _ => throw new ArgumentOutOfRangeException(nameof(direction))
            };
            SetWalkerCell(row + 1, col + 1, image);
            // Scroll one cell ahead so the player can see what's coming before reaching the edge
            (int aheadRow, int aheadCol) = direction switch
            {
                MazeGameDirection.Up => (row - 1, col + 1),
                MazeGameDirection.Down => (row + 3, col + 1),
                MazeGameDirection.Left => (row + 1, col - 1),
                MazeGameDirection.Right => (row + 1, col + 3),
                _ => (row + 1, col + 1)
            };
            ScrollCellIntoView(Math.Clamp(aheadRow, 1, RowCount), Math.Clamp(aheadCol, 1, ColumnCount));
        }
        /// <summary>
        /// Shows the celebration sprite at the given 0-based cell.
        /// </summary>
        /// <param name="row">Row index (0-based)</param>
        /// <param name="col">Column index (0-based)</param>
        public void SetPlayerCelebrate(int row, int col)
        {
            SetWalkerCell(row + 1, col + 1, "walker_celebrate.gif");
            ScrollCellIntoView(row + 1, col + 1);
        }
        /// <summary>
        /// Stamps a footstep overlay on a previously-visited 0-based cell.
        /// Pass <see cref="MazeGameDirection.None"/> to clear the overlay.
        /// </summary>
        /// <param name="row">Row index (0-based)</param>
        /// <param name="col">Column index (0-based)</param>
        /// <param name="direction">Direction the player was moving when they left this cell</param>
        public void SetFootstepAt(int row, int col, MazeGameDirection direction)
        {
            var dir = direction switch
            {
                MazeGameDirection.Up => MazeCellContent.PathDirection.Up,
                MazeGameDirection.Down => MazeCellContent.PathDirection.Down,
                MazeGameDirection.Left => MazeCellContent.PathDirection.Left,
                MazeGameDirection.Right => MazeCellContent.PathDirection.Right,
                MazeGameDirection.None => MazeCellContent.PathDirection.None,
                _ => throw new ArgumentOutOfRangeException(nameof(direction))
            };
            SetSolutionCell(row + 1, col + 1, dir);
        }
        /// <summary>
        /// Stamps a dot marker on a previously-visited 0-based cell (game mode).
        /// </summary>
        /// <param name="row">Row index (0-based)</param>
        /// <param name="col">Column index (0-based)</param>
        public void SetVisitedDotAt(int row, int col)
            => SetSolutionCell(row + 1, col + 1, MazeCellContent.PathDirection.VisitedDot);

        /// <summary>
        /// Removes the player sprite and all footstep overlays.
        /// </summary>
        public void ClearPlayer() => ClearLastSolution();
        /// <summary>
        /// Returns the movement direction from one maze point to the next.
        /// </summary>
        private static MazeGameDirection GetMovementDirection(Api.Maze.Point from, Api.Maze.Point to)
        {
            if (to.Row < from.Row) return MazeGameDirection.Up;
            if (to.Row > from.Row) return MazeGameDirection.Down;
            if (to.Column < from.Column) return MazeGameDirection.Left;
            return MazeGameDirection.Right;
        }
        /// <summary>
        /// Collapses a corner-aware <see cref="MazeCellContent.PathDirection"/> to its cardinal
        /// <see cref="MazeGameDirection"/> equivalent. Corner variants were intended for diagonal
        /// footstep images that were never implemented; all render identically to their cardinal form.
        /// </summary>
        private static MazeGameDirection PathDirectionToCardinal(MazeCellContent.PathDirection dir) =>
            dir switch
            {
                MazeCellContent.PathDirection.Up or
                MazeCellContent.PathDirection.UpFromLeft or
                MazeCellContent.PathDirection.UpFromRight => MazeGameDirection.Up,
                MazeCellContent.PathDirection.Down or
                MazeCellContent.PathDirection.DownFromLeft or
                MazeCellContent.PathDirection.DownFromRight => MazeGameDirection.Down,
                MazeCellContent.PathDirection.Left or
                MazeCellContent.PathDirection.LeftFromUp or
                MazeCellContent.PathDirection.LeftFromDown => MazeGameDirection.Left,
                MazeCellContent.PathDirection.Right or
                MazeCellContent.PathDirection.RightFromUp or
                MazeCellContent.PathDirection.RightFromDown => MazeGameDirection.Right,
                _ => MazeGameDirection.None
            };
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
                cellContent?.SetSolutionPath(direction);
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
        readonly Controls.Keyboard.KeyState keyState = Controls.Keyboard.KeyState.None;
        readonly Controls.Keyboard.Key key = Controls.Keyboard.Key.None;

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
        /// Indicates whether the selection contains a key cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsKey { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains a door cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsDoor { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains an enemy cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsEnemy { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains a health cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsHealth { get; set; } = false;
        /// <summary>
        /// Indicates whether the selection contains a treasure cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool ContainsTreasure { get; set; } = false;
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
        /// Indicates whether the selection contains all empty cells. K, D, E,
        /// H and T cells count as non-empty so that the Clear button enables on a
        /// selection that contains them — mirrors the React editor's
        /// <c>selectionStatus.isEmpty</c> rule.
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsEmpty { get => !ContainsWall && !ContainsStart && !ContainsFinish && !ContainsKey && !ContainsDoor && !ContainsEnemy && !ContainsHealth && !ContainsTreasure; }
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
            DownFromRight = 12,
            /// <summary>
            /// A visited-cell dot marker (used in game mode)
            /// </summary>
            /// <returns>Dot</returns>
            VisitedDot = 13
        }

        private static readonly Color SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR = Colors.White;
        private static readonly Color SOLUTION_PATH_CELL_HIGHLIGHT_COLOR = Colors.LightGreen;
        private static readonly Color GAME_VISITED_CELL_HIGHLIGHT_COLOR = Colors.White;
        // Corner dot marking an editor cell that carries a per-cell override, mirroring
        // the web editor's override badge.
        private static readonly Microsoft.Maui.Controls.Brush OVERRIDE_BADGE_BRUSH =
            new Microsoft.Maui.Controls.SolidColorBrush(Color.FromArgb("#512BD4"));

        CellType cellType = CellType.Empty;
        PathDirection solutionPathDirection = PathDirection.None;
        // The cell's per-cell override (drives the variant sprite + badge), or null.
        CellEntityInfo? cellOverride = null;
        // The maze's game settings, supplying the base sprite a non-overridden wall /
        // enemy / health cell inherits (e.g. a lava maze's walls), or null for the
        // hardcoded bases. Per-cell overrides still win.
        MazeGameSettings? settings = null;
        // Whether to overlay the authoring badge on an override cell. The variant sprite
        // always renders from the override; the badge is an editor affordance only and is
        // suppressed during play (matching the web editor, which hides it in-game).
        bool showOverrideBadge = true;

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
        /// Indicates whether the cell is a key cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsKey { get => CellType == CellType.Key; }
        /// <summary>
        /// Indicates whether the cell is a door cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsDoor { get => CellType == CellType.Door; }
        /// <summary>
        /// Indicates whether the cell is an enemy cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsEnemy { get => CellType == CellType.Enemy; }
        /// <summary>
        /// Indicates whether the cell is a health cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsHealth { get => CellType == CellType.Health; }
        /// <summary>
        /// Indicates whether the cell is a treasure cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsTreasure { get => CellType == CellType.Treasure; }
        /// <summary>
        /// Indicates whether the cell is a start or finish cell
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsStartOrFinish { get => IsStart || IsFinish; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="cellType">Cell type</param>
        /// <param name="cellOverride">The cell's per-cell override (drives the variant sprite + badge), or null</param>
        /// <param name="settings">The maze's game settings (supplies the non-overridden base sprite), or null</param>
        /// <param name="showBadge">Whether to overlay the authoring badge on an override cell (editor only)</param>
        public MazeCellContent(CellType cellType, CellEntityInfo? cellOverride = null, MazeGameSettings? settings = null, bool showBadge = true)
        {
            this.cellType = cellType;
            this.cellOverride = cellOverride;
            this.settings = settings;
            this.showOverrideBadge = showBadge;
            switch (cellType)
            {
                case CellType.Start:
                case CellType.Finish:
                case CellType.Wall:
                case CellType.Key:
                case CellType.Door:
                case CellType.Enemy:
                case CellType.Health:
                case CellType.Treasure:
                    Content = BuildIconContent();
                    break;
                case CellType.Empty:
                default:
                    Content = new Label();
                    break;
            }
        }
        /// <summary>
        /// Builds the cell icon: a single image, or — when the cell carries an
        /// override — that image with a small corner badge overlaid.
        /// </summary>
        /// <returns>The cell content view</returns>
        private View BuildIconContent()
        {
            Image image = new()
            {
                Source = GetImageName(true),
                Aspect = Aspect.AspectFit,
                HorizontalOptions = LayoutOptions.Fill,
                VerticalOptions = LayoutOptions.Fill
            };
            if (cellOverride is null || !showOverrideBadge)
            {
                return image;
            }
            return new Microsoft.Maui.Controls.Grid { image, BuildOverrideBadge() };
        }
        /// <summary>
        /// A small corner dot marking a cell that carries a per-cell override.
        /// </summary>
        /// <returns>The badge view</returns>
        private static Microsoft.Maui.Controls.Shapes.Ellipse BuildOverrideBadge() => new Microsoft.Maui.Controls.Shapes.Ellipse
        {
            Fill = OVERRIDE_BADGE_BRUSH,
            WidthRequest = 7,
            HeightRequest = 7,
            HorizontalOptions = LayoutOptions.End,
            VerticalOptions = LayoutOptions.Start,
            Margin = new Thickness(0, 1, 1, 0),
            InputTransparent = true
        };
        /// <summary>
        /// Gets the name of the image to display for the cell, preferring a variant
        /// sprite when the cell's override selects one (ghost / potion / water / lava /
        /// iron fence) and otherwise the base sprite for the cell type.
        /// </summary>
        /// <param name="preferFlag">If the cell is a start or finish cell, returned a flag image (otherwise return a sign image)</param>
        /// <returns>Image name</returns>
        private string GetImageName(bool preferFlag)
        {
            string? variant = CellSprite.VariantImageName(cellOverride);
            if (variant is not null)
            {
                return variant;
            }
            // A non-overridden wall / enemy / health cell inherits the maze's game-settings
            // default base (e.g. lava walls); falls through to the hardcoded base otherwise.
            string? mazeDefault = CellSprite.BaseImageName(cellType, cellOverride, settings);
            if (mazeDefault is not null)
            {
                return mazeDefault;
            }
            switch (cellType)
            {
                case CellType.Start:
                    return preferFlag ? "start_flag.png" : "start_sign.png";
                case CellType.Finish:
                    return preferFlag ? "finish_flag.png" : "finish_sign.png";
                case CellType.Wall:
                    return "wall.png";
                case CellType.Key:
                    return "key.png";
                case CellType.Door:
                    return "door.png";
                case CellType.Enemy:
                    return "enemy.png";
                case CellType.Health:
                    return "health.png";
                case CellType.Treasure:
                    // Silver is the default treasure sprite; the richer styles are
                    // variants resolved above via CellSprite.VariantImageName.
                    return "silver_in_trunk.png";
            }
            return "";
        }
        /// <summary>
        /// Updates the cell content in-place, reusing the existing Image or Label where possible
        /// to avoid a new async image-load cycle on pool-recycled frames.
        /// </summary>
        /// <param name="newCellType">New cell type</param>
        /// <param name="newDirection">New solution path direction</param>
        /// <param name="newOverride">The cell's per-cell override (drives the variant sprite + badge), or null</param>
        /// <param name="newSettings">The maze's game settings (supplies the non-overridden base sprite), or null</param>
        /// <param name="showBadge">Whether to overlay the authoring badge on an override cell (editor only)</param>
        public void Update(CellType newCellType, PathDirection newDirection, CellEntityInfo? newOverride = null, MazeGameSettings? newSettings = null, bool showBadge = true)
        {
            cellType = newCellType;
            solutionPathDirection = newDirection;
            cellOverride = newOverride;
            settings = newSettings;
            showOverrideBadge = showBadge;

            bool needsImage = cellType != CellType.Empty || solutionPathDirection != PathDirection.None;

            if (!needsImage)
            {
                if (Content is not Label)
                    Content = new Label();
                Content.BackgroundColor = Colors.Transparent;
                return;
            }

            if (cellType != CellType.Empty && cellOverride is not null)
            {
                // Override cell: rebuild as the variant sprite plus the corner badge.
                Content = BuildIconContent();
            }
            else
            {
                // Plain icon (or an empty cell's solution footstep) — reuse the Image
                // to avoid a reload cycle on pool-recycled frames.
                string source = cellType == CellType.Empty ? GetSolutionPathImage() : GetImageName(true);
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
            }
            Content.BackgroundColor = GetSolutionPathHighlightColor();
        }
        /// <summary>
        /// Sets the opacity of this cell's icon image. Used by game mode to
        /// dim a door that's opening (0.5) or hide a collected key /
        /// fully-open door (0.0). A no-op when the cell isn't currently
        /// rendering an <see cref="Image"/>.
        /// </summary>
        /// <param name="opacity">Target opacity in <c>[0.0, 1.0]</c>.</param>
        public void SetIconOpacity(double opacity)
        {
            if (Content is Image img)
            {
                img.Opacity = opacity;
            }
        }
        /// <summary>
        /// Renders the live game entities over the cell's static base content: the base
        /// cell sprite (e.g. a potion the enemy is standing on) shows through underneath,
        /// then the live enemy sprite, then the player walker when the player shares the
        /// cell (enemy dimmed behind it), and a dark-green count chip in the top-right
        /// corner when two or more enemies occupy the cell. Mirrors the React 2D game,
        /// which overlays entities on the cell rather than replacing it. The current
        /// <c>cellType</c>/override already reflect the resolved game-mode base (spawn
        /// markers and collected pickups have been reduced to an empty passage).
        /// </summary>
        /// <param name="enemyCount">Number of enemies on the cell.</param>
        /// <param name="walkerImage">Player walker GIF source when the player is here; otherwise null.</param>
        /// <param name="enemyType">The enemy's visual rig (ghost vs the default goblin), or null.</param>
        public void SetEntityOverlay(int enemyCount, string? walkerImage, EnemyType? enemyType = null)
        {
            var layers = new Microsoft.Maui.Controls.Grid { BackgroundColor = Colors.Transparent };
            // Base cell sprite underneath the entities (an empty passage adds no layer).
            if (cellType != CellType.Empty)
            {
                layers.Add(BuildIconContent());
            }
            if (enemyCount > 0)
            {
                layers.Add(new Image
                {
                    Source = CellSprite.LiveEnemyImageName(enemyType, settings),
                    Aspect = Aspect.AspectFit,
                    HorizontalOptions = LayoutOptions.Fill,
                    VerticalOptions = LayoutOptions.Fill,
                    // Dim the enemy behind the player so the player reads as foreground.
                    Opacity = walkerImage is not null ? 0.5 : 1.0,
                });
            }
            if (walkerImage is not null)
            {
                layers.Add(new Image
                {
                    Source = walkerImage,
                    Aspect = Aspect.AspectFit,
                    HorizontalOptions = LayoutOptions.Fill,
                    VerticalOptions = LayoutOptions.Fill,
                    IsAnimationPlaying = true,
                });
            }
            if (enemyCount >= 2)
            {
                layers.Add(new Border
                {
                    BackgroundColor = Color.FromArgb("#2a4d18"),
                    Stroke = Colors.Transparent,
                    StrokeShape = new Microsoft.Maui.Controls.Shapes.RoundRectangle { CornerRadius = new CornerRadius(6) },
                    Padding = new Thickness(3, 0),
                    HorizontalOptions = LayoutOptions.End,
                    VerticalOptions = LayoutOptions.Start,
                    Content = new Label
                    {
                        Text = enemyCount.ToString(),
                        TextColor = Colors.White,
                        FontSize = 9,
                        FontAttributes = FontAttributes.Bold,
                    },
                });
            }
            Content = layers;
            Content.BackgroundColor = Colors.Transparent;
        }
        /// <summary>
        /// Sets the solution path direction in the cell
        /// </summary>
        /// <param name="pathDirection">Path direction</param>
        public void SetSolutionPath(PathDirection pathDirection)
        {
            // Empty cells get a footstep image; Start/Finish/Key/Door/Enemy/Health
            // keep their existing icon and just gain a green BackgroundColor that
            // shows through the icon's transparent border. (These icon PNGs
            // ship with transparent corners specifically so the highlight
            // is visible here.) Enemy and health cells are passable, so the
            // solution path can cross them and they highlight like any passage.
            if (IsEmpty || IsStartOrFinish || IsKey || IsDoor || IsEnemy || IsHealth || IsTreasure)
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
            if (!ContainsSolutionPath) return Colors.Transparent;
            if (IsStartOrFinish) return SOLUTION_PATH_START_FINISH_HIGHLIGHT_COLOR;
            if (solutionPathDirection == PathDirection.VisitedDot) return GAME_VISITED_CELL_HIGHLIGHT_COLOR;
            return SOLUTION_PATH_CELL_HIGHLIGHT_COLOR;
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
                case PathDirection.VisitedDot:
                    return "visited_dot.png";
                case PathDirection.None:
                default:
                    return "";
            }
        }
    }

}
