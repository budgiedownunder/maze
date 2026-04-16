
namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.Services;
    using Maze.Maui.App.Models;
    using Maze.Maui.App.Services;
    using CommunityToolkit.Maui.Extensions;
    using CommunityToolkit.Maui.Core;
    using Maze.Maui.Controls.Pointer;
    using Maze.Maui.App.Extensions;
    using Maze.Maui.Controls.InteractiveGrid;

    /// <summary>
    /// This class represents the maze page within the application. It provides
    /// functionality to design and solve mazes.
    /// 
    /// This is how the page appears on Windows Desktop:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Design</strong></th>
    ///         <th><strong>Solved</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/windows-design.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/windows-solved.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    ///  
    /// and this is how it appears on Android/iOS devices:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Android</strong></th>
    ///         <th><strong>iOS</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/android-solved.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/ios-solved.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    /// </summary>
    public partial class MazePage : ContentPage
    {
        // Private definitions
        const String APP_TITLE = "MAZE";
        // Private properties
        MazeViewModel _viewModel;
        MazesViewModel _mazesViewModel;
        IDialogService _dialogService;
        IDeviceTypeService _deviceTypeService;
        uint? _lastMinSolutionLength;
        CancellationTokenSource? _fallbackInitCts;
        CancellationTokenSource? _walkCts;
        bool _isWalking = false;
        bool _walkSolutionDisplayed = false;
        CellRange? _savedWalkSelection;
        static readonly int[] WALK_SPEED_MS = [750, 500, 200, 20];
        static readonly string[] WALK_SPEED_LABELS = ["Slow", "Normal", "Fast", "Turbo"];
        const string WALK_SPEED_PREF_KEY = "walk_speed_index";
        const int WALK_SPEED_DEFAULT_INDEX = 1; // Normal
        int _walkSpeedIndex = WALK_SPEED_DEFAULT_INDEX;

        /// <summary>
        /// Indicates whether the page is initlialized
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsInitialized { get; set; }
        /// <summary>
        /// Indicates whether pan gestures are supported
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsPanSupported { get; set; }
        /// <summary>
        /// Indicates whether the device being used is a touch-only device
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsTouchOnlyDevice { get => _viewModel.IsTouchOnlyDevice; }
        /// <summary>
        /// Indicates whether maze solve is supported
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsSolveSupported { get => OperatingSystem.IsWindows() || OperatingSystem.IsAndroid() || OperatingSystem.IsIOS(); }
        /// <summary>
        /// Indicates whether maze generation is supported
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsGenerationSupported { get => OperatingSystem.IsWindows() || OperatingSystem.IsAndroid() || OperatingSystem.IsIOS(); }
        /// <summary>
        /// Indicates whether the maze solution is currently displayed
        /// </summary>
        /// <returns>Boolean value</returns>
        private bool IsSolutionDisplayed { get; set; } = false;
        /// <summary>
        /// Represents the maze item associated with the page
        /// </summary>
        /// <returns>Boolean value</returns>
        public MazeItem? MazeItem { get; set; }
        /// <summary>
        /// Indicates whether the maze has unsaved changes
        /// </summary>
        public bool IsDirty => _viewModel.CanSave;
        /// <summary>
        /// Saves the maze. Returns true on success, false if the save failed or was cancelled.
        /// </summary>
        public Task<bool> TrySaveAsync() => Save();
        /// <summary>
        /// Constructor 
        /// </summary>
        /// <param name="deviceTypeService">Injected device type service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="viewModel">Injected maze view model</param>
        /// <param name="mazesViewModel">Injected mazes view model</param>
        public MazePage(IDeviceTypeService deviceTypeService, IDialogService dialogService, MazeViewModel viewModel, MazesViewModel mazesViewModel)
        {
            InitializeComponent();
            _deviceTypeService = deviceTypeService;
            _dialogService = dialogService;
            _viewModel = viewModel;
            _mazesViewModel = mazesViewModel;

            BindingContext = viewModel;

            IsPanSupported = !_deviceTypeService.IsTouchOnlyDevice();

            _viewModel.InsertRowsRequested += (s, e) => InsertRows();
            _viewModel.DeleteRowsRequested += (s, e) => DeleteRows();
            _viewModel.InsertColumnsRequested += (s, e) => InsertColumns();
            _viewModel.DeleteColumnsRequested += (s, e) => DeleteColumns();
            _viewModel.SelectRangeRequested += (s, e) => { SetSelectRangeMode(true); };
            _viewModel.DoneRequested += (s, e) => { SetSelectRangeMode(false); };
            _viewModel.SetWallRequested += (s, e) => { ChangeSelectionToWall(); };
            _viewModel.SetStartRequested += (s, e) => { ChangeSelectionToStart(); };
            _viewModel.SetFinishRequested += (s, e) => { ChangeSelectionToFinish(); };
            _viewModel.ClearRequested += (s, e) => { ClearSelection(); };
            _viewModel.SolveRequested += (s, e) => { Solve(); };
            _viewModel.WalkSolutionRequested += async (s, e) => { await WalkSolution(); };
            _viewModel.ClearSolutionRequested += async (s, e) => { await ClearSolution(); };
            _viewModel.SaveRequested += async (s, e) => { await Save(); };
            _viewModel.RefreshRequested += async (s, e) => { await Refresh(); };
            _viewModel.GenerateRequested += async (s, e) => { await Generate(); };
            _walkSpeedIndex = Preferences.Default.Get(WALK_SPEED_PREF_KEY, WALK_SPEED_DEFAULT_INDEX);
            if (_walkSpeedIndex < 0 || _walkSpeedIndex >= WALK_SPEED_MS.Length)
                _walkSpeedIndex = WALK_SPEED_DEFAULT_INDEX;
        }
        /// <summary>
        /// Intializes the page 
        /// </summary>
        private void Initialize()
        {
            if (IsInitialized)
                return;

            MazeItem = _viewModel.MazeItem;

            MazeGrid.Initialize(IsPanSupported, MazeItem);
            MazeGrid.CellTapped += OnMazeGridCellTapped;
            MazeGrid.CellDoubleTapped += OnMazeGridCellDoubleTapped;
            MazeGrid.KeyDown += OnMazeGridKeyDown;
            MazeGrid.SelectionChanged += OnMazeGridSelectionChanged;

            MazeGrid.ActivateCell(1, 1, false);

            _viewModel.IsStored = MazeItem.ID != "";
            _viewModel.CanRefresh = false;
            _viewModel.CanSave = MazeItem.ID == "";

            WalkSpeedPicker.ItemsSource = WALK_SPEED_LABELS;
            WalkSpeedPicker.SelectedIndex = _walkSpeedIndex;

            UpdateControls();

            IsInitialized = true;
        }
        /// <summary>
        /// Resets the page display (e.g following refresh)
        /// </summary>
        private void ResetDisplay()
        {
            IsSolutionDisplayed = false;
            MazeGrid.Initialize(IsPanSupported, MazeItem);
            MazeGrid.ActivateCell(1, 1, false);
            UpdateControls();
        }
        /// <summary>
        /// Handles the maze grid cell tapped event
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Event arguments</param>
        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            if (_isWalking) return;
            Initialize();
            MazeGrid.OnCellTapped(e.Cell, false);
        }
        /// <summary>
        /// Handles the maze grid cell double-tapped event
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Event arguments</param>
        private void OnMazeGridCellDoubleTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            bool inExtendedSelectionMode = MazeGrid.IsExtendedSelectionMode;
            if (IsTouchOnlyDevice && inExtendedSelectionMode)
                SetSelectRangeMode(false);

            MazeGrid.OnCellDoubleTapped(e.Cell, false);

            if (IsTouchOnlyDevice && !inExtendedSelectionMode)
                SetSelectRangeMode(true);
        }
        /// <summary>
        /// Handles the maze grid cell key down event
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Event arguments</param>
        private void OnMazeGridKeyDown(object sender, MazeGridKeyDownEventArgs e)
        {
            if (_isWalking) return;
            switch (e.Key)
            {
                case Controls.Keyboard.Key.W:
                    if (_viewModel.CanSetWall)
                        _viewModel.SetWallCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.S:
                    if (_viewModel.CanSetStart)
                        _viewModel.SetStartCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.F:
                    if (_viewModel.CanSetFinish)
                        _viewModel.SetFinishCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.Delete:
                    if (_viewModel.CanClear)
                        _viewModel.ClearCommand.Execute(null);
                    break;
                default:
                    MazeGrid.OnProcessKeyDown(e.KeyState, e.Key, false);
                    break;
            }
        }
        /// <summary>
        /// Changes the selected cells to walls
        /// </summary>
        private void ChangeSelectionToWall()
        {
            ChangeSelectedCellsContent(Maze.CellType.Wall);
        }
        /// <summary>
        /// Changes the selected cell to a start cell
        /// </summary>
        private void ChangeSelectionToStart()
        {
            ChangeSelectedCellsContent(Maze.CellType.Start);
        }
        /// <summary>
        /// Changes the selected cell to a finish cell
        /// </summary>
        private void ChangeSelectionToFinish()
        {
            ChangeSelectedCellsContent(Maze.CellType.Finish);
        }
        /// <summary>
        /// Clears the selected cell(s) content
        /// </summary>
        private void ClearSelection()
        {
            ChangeSelectedCellsContent(Maze.CellType.Empty);
        }
        /// <summary>
        /// Changes the selected cell(s) content to the given cell type
        /// </summary>
        /// <param name="newCellType">New cell type</param>
        private void ChangeSelectedCellsContent(Maze.CellType newCellType)
        {
            MazeGrid.SetSelectionContent(newCellType);
            ExitSelectionModeAndUpdateControls();
        }
        /// <summary>
        /// Deletes the selected rows
        /// </summary>
        private void DeleteRows()
        {
            MazeGrid.DeleteSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }
        /// <summary>
        /// Deletes the selected columns
        /// </summary>
        private void DeleteColumns()
        {
            MazeGrid.DeleteSelectedColumns();
            ExitSelectionModeAndUpdateControls();
        }
        /// <summary>
        /// Inserts rows at the current row selection
        /// </summary>
        private void InsertRows()
        {
            MazeGrid.InsertSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }
        /// <summary>
        /// Inserts columns at the current column selection
        /// </summary>
        private void InsertColumns()
        {
            MazeGrid.InsertSelectedColumns(); ;
            ExitSelectionModeAndUpdateControls();
        }
        /// <summary>
        /// Attempts to solve the maze. If successful, the solution is displayed. If not, an error message is displayed.
        /// </summary>
        private void Solve()
        {
            SetBusyIndicators(true, updateViewModel: false);
            try
            {
                Maze maze = MazeGrid.ToMaze();
                Solution solution = maze.Solve();

                IsSolutionDisplayed = MazeGrid.DisplaySolution(solution);
                UpdateControls();
            }
            catch (Exception ex)
            {
                _dialogService.ShowAlert(APP_TITLE, $"Unable to solve maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            finally
            {
                SetBusyIndicators(false, updateViewModel: false);
            }
        }
        /// <summary>
        /// Triggers the save maze process
        /// </summary>
        private async Task<bool> Save()
        {
            SetBusyIndicators(true, updateViewModel: false);
            try
            {
                return await _viewModel.SaveMaze(MazeGrid.ToMaze());
            }
            finally
            {
                SetBusyIndicators(false, updateViewModel: false);
            }
        }
        /// <summary>
        /// Triggers the refresh maze process. If successful, the maze is updated to reflect the updated definition.
        /// </summary>
        private async Task<bool> Refresh()
        {
            bool refreshed = false;
            try
            {
                refreshed = await _viewModel.RefreshMaze();
                if (refreshed)
                {
                    _viewModel.IsBusy = true;
                    await Task.Delay(50);
                    ResetDisplay();
                }
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert(APP_TITLE, $"Failed to refresh maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            finally
            {
                _viewModel.IsBusy = false;
            }
            return refreshed;
        }
        /// <summary>
        /// Triggers the generate maze process. Prompts the user for generation options and, if confirmed,
        /// generates a new maze and updates the display.
        /// </summary>
        private async Task Generate()
        {
            try
            {
                Maze current = MazeGrid.ToMaze();
                uint rows = current.IsEmpty ? 7 : current.RowCount;
                uint cols = current.IsEmpty ? 7 : current.ColCount;
                uint startRow = 0, startCol = 0;
                uint finishRow = rows > 0 ? rows - 1 : 0;
                uint finishCol = cols > 0 ? cols - 1 : 0;

                if (!current.IsEmpty)
                {
                    if (current.HasStartCell)
                    {
                        var start = current.GetStartCell();
                        startRow = start.Row;
                        startCol = start.Column;
                    }
                    if (current.HasFinishCell)
                    {
                        var finish = current.GetFinishCell();
                        finishRow = finish.Row;
                        finishCol = finish.Column;
                    }
                    // If either is missing, leave the fallback defaults (top-left / bottom-right)
                }

                uint defaultMinSolutionLength = (rows + cols) / 2;
                uint minSolutionLength = _lastMinSolutionLength is uint last && last <= rows * cols
                    ? last
                    : defaultMinSolutionLength;
                string? generationError = null;

                while (true)
                {
                    var popup = new GenerateMazePopup(rows, cols, startRow, startCol, finishRow, finishCol, minSolutionLength, generationError);
                    IPopupResult<Maze.GenerationOptions?> result = await this.ShowPopupAsync<Maze.GenerationOptions?>(popup);

                    if (result.WasDismissedByTappingOutsideOfPopup || result.Result is not Maze.GenerationOptions popupOptions)
                        break;

                    var options = new Maze.GenerationOptions
                    {
                        RowCount = popupOptions.RowCount,
                        ColCount = popupOptions.ColCount,
                        Seed = (ulong)DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),
                        StartRow = popupOptions.StartRow,
                        StartCol = popupOptions.StartCol,
                        FinishRow = popupOptions.FinishRow,
                        FinishCol = popupOptions.FinishCol,
                        MinSpineLength = popupOptions.MinSpineLength,
                    };

                    bool generationSucceeded = false;
                    SetBusyIndicators(true);
                    try
                    {
                        await Task.Delay(150);
                        Maze generated = Maze.Generate(options);
                        _lastMinSolutionLength = options.MinSpineLength;
                        await MainThread.InvokeOnMainThreadAsync(() =>
                        {
                            MazeItem!.Definition = generated;
                            ResetDisplay();
                        });
                        generationSucceeded = true;
                        break;
                    }
                    catch (Exception ex)
                    {
                        generationError = ex.Message;
                        rows = popupOptions.RowCount;
                        cols = popupOptions.ColCount;
                        startRow = popupOptions.StartRow ?? startRow;
                        startCol = popupOptions.StartCol ?? startCol;
                        finishRow = popupOptions.FinishRow ?? finishRow;
                        finishCol = popupOptions.FinishCol ?? finishCol;
                        minSolutionLength = popupOptions.MinSpineLength ?? minSolutionLength;
                    }
                    finally
                    {
                        SetBusyIndicators(false);
                        if (generationSucceeded)
                            _viewModel.NotifyMazeChanged();
                    }
                }
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert(APP_TITLE, $"Failed to generate maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
        }
        /// <summary>
        /// Animates a walker character stepping through the solution path one cell at a time.
        /// Blocks all editing for the duration of the walk. Clear Solution cancels the walk.
        /// </summary>
        private async Task WalkSolution()
        {
            _walkCts = new CancellationTokenSource();
            _isWalking = true;
            MazeGrid.IsInteractionLocked = true;
            _savedWalkSelection = MazeGrid.CurrentSelection;
            MazeGrid.ClearSelection();
            UpdateControls();
            try
            {
                Maze maze = MazeGrid.ToMaze();
                Solution solution = maze.Solve();
                await MazeGrid.WalkSolutionAsync(solution, () => WALK_SPEED_MS[_walkSpeedIndex], _walkCts.Token);
                IsSolutionDisplayed = true;
                _walkSolutionDisplayed = true;
            }
            catch (OperationCanceledException) { /* cancelled cleanly by ClearSolution */ }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert(APP_TITLE, $"Unable to solve maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            finally
            {
                _walkCts?.Dispose();
                _walkCts = null;
                _isWalking = false;
                MazeGrid.IsInteractionLocked = false;
                // Restore selection immediately when cancelled; on success, defer to ClearSolution
                if (!IsSolutionDisplayed)
                {
                    _walkSolutionDisplayed = false;
                    if (_savedWalkSelection is CellRange sel && sel.Top > 0)
                        MazeGrid.ActivateCell(new CellPoint(sel.Top, sel.Left), false);
                    _savedWalkSelection = null;
                }
                UpdateControls();
            }
        }
        /// <summary>
        /// Clears the maze solution that is displayed (if any) and resets toolbar/buttons states appropriately
        /// </summary>
        private async Task ClearSolution()
        {
            _walkCts?.Cancel();
            SetBusyIndicators(true, updateViewModel: false);
            await Task.Delay(50);
            try
            {
                IsSolutionDisplayed = !MazeGrid.ClearLastSolution();
                _walkSolutionDisplayed = false;
                // Restore the selection that was saved when the walk started
                if (_savedWalkSelection is CellRange sel && sel.Top > 0)
                    MazeGrid.ActivateCell(new CellPoint(sel.Top, sel.Left), false);
                _savedWalkSelection = null;
                UpdateControls();
            }
            finally
            {
                SetBusyIndicators(false, updateViewModel: false);
            }
        }
        /// <summary>
        /// Enables or disables extended selection (select range) mode and updates the page button states appropriately
        /// </summary>
        /// <param name="enable">Enable?</param>
        private void SetSelectRangeMode(bool enable)
        {
            EnableExtendedSelectionMode(enable);
            ShowSelectRangeButtons(!enable);
        }
        /// <summary>
        /// Exits extended selection (select range) mode
        /// </summary>
        private void ExitSelectionModeAndUpdateControls()
        {
            EnableExtendedSelectionMode(false);
            UpdateControls();
        }
        /// <summary>
        /// Enables or disables extended selection (select range) mode
        /// </summary>
        /// <param name="enable">Enable?</param>
        private void EnableExtendedSelectionMode(bool enable)
        {
            if (MazeGrid.IsExtendedSelectionMode == enable)
                return;

            if (enable)
                MazeGrid.EnableExtendedSelection();
            else
                MazeGrid.CancelExtendedSelection();
        }
        /// <summary>
        /// Adjusts the cell edit flags based on the cells that are selected
        /// </summary>
        private void ShowCellEditButtons()
        {
            CellStatus status = MazeGrid.GetCurrentSelectionStatus();

            _viewModel.CanSetWall = !status.IsAllWalls && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanSetStart = status.IsSingleCell && !status.IsStart && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanSetFinish = status.IsSingleCell && !status.IsFinish && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanClear = !status.IsEmpty && !IsSolutionDisplayed && !_isWalking;
        }
        /// <summary>
        /// Adjusts the row and column edit flags based on the cells that are selected
        /// </summary>
        private void ShowEditRowColumnButtons()
        {
            bool allRowsSelected = MazeGrid.AllRowsSelected;
            bool allColumnsSelected = MazeGrid.AllColumnsSelected;

            _viewModel.CanInsertRows = allColumnsSelected && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanDeleteRows = allColumnsSelected && !allRowsSelected && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanInsertColumns = allRowsSelected && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanDeleteColumns = allRowsSelected && !allColumnsSelected && !IsSolutionDisplayed && !_isWalking;
        }
        /// <summary>
        /// Adjusts the select range visibility flags based on whether visibility is required and a solution is displayed
        /// </summary>
        /// <param name="show">Show?</param>
        private void ShowSelectRangeButtons(bool show)
        {
            bool touchOnly = IsTouchOnlyDevice;
            bool showSelectRangeBtn = show && touchOnly && !IsSolutionDisplayed && !_isWalking;
            bool showDoneBtn = !show && touchOnly && !IsSolutionDisplayed && !_isWalking;

            _viewModel.CanSelectRange = showSelectRangeBtn;
            _viewModel.CanShowDone = showDoneBtn;
        }
        /// <summary>
        /// Adjusts the solve-related flags based on whether solve is supported and a solution is displayed
        /// </summary>
        private void ShowSolveButtons()
        {
            _viewModel.CanSolve = IsSolveSupported && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanWalkSolution = IsSolveSupported && !IsSolutionDisplayed && !_isWalking;
            _viewModel.CanClearSolution = IsSolveSupported && (IsSolutionDisplayed || _isWalking);
            _viewModel.IsWalking = _isWalking || _walkSolutionDisplayed;
        }
        /// <summary>
        /// Handles walk speed picker selection changes; saves the new preference
        /// </summary>
        private void OnWalkSpeedPickerChanged(object sender, EventArgs e)
        {
            int index = WalkSpeedPicker.SelectedIndex;
            if (index < 0 || index >= WALK_SPEED_MS.Length) return;
            _walkSpeedIndex = index;
            Preferences.Default.Set(WALK_SPEED_PREF_KEY, _walkSpeedIndex);
        }
        /// <summary>
        /// Adjusts the generate button visibility based on whether generation is supported and a solution is displayed
        /// </summary>
        private void ShowGenerateButton()
        {
            _viewModel.CanGenerate = IsGenerationSupported && !IsSolutionDisplayed && !_isWalking;
        }
        /// <summary>
        /// Handles the maze cell selection changed event
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Event arguments</param>
        private void OnMazeGridSelectionChanged(object sender, MazeGridSelectionChangedEventArgs e)
        {
            UpdateControls();
        }
        /// <summary>
        /// Sets the busy indicators: the cursor and optionally the view model's IsBusy flag.
        /// </summary>
        /// <param name="busy">True to indicate busy; false to indicate ready.</param>
        /// <param name="updateViewModel">If true (default), also updates view model</param>
        private void SetBusyIndicators(bool busy, bool updateViewModel = true)
        {
            Pointer.SetCursor(this, busy ? Icon.Wait : Icon.Arrow);
            if (updateViewModel)
                _viewModel.IsBusy = busy;
        }
        /// <summary>
        /// Updates the control visibility/enabled states
        /// </summary>
        private void UpdateControls()
        {
            bool showSelectRangeButtons = IsTouchOnlyDevice || MazeGrid.IsExtendedSelectionMode;
            bool haveSelection = MazeGrid.HasActiveCell;
            bool showTopRowLayout = showSelectRangeButtons || haveSelection || _isWalking || IsSolutionDisplayed;
            ShowMainGridRow(0, showTopRowLayout);
            if (showTopRowLayout)
            {
                ShowCellEditButtons();
                ShowEditRowColumnButtons();
                ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
                ShowSolveButtons();
                ShowGenerateButton();
            }
        }
        /// <summary>
        /// Shows/hides a given grid row
        /// </summary>
        /// <param name="row">Row index (zero-based)</param>
        /// <param name="show">Show?</param>
        private void ShowMainGridRow(int row, bool show)
        {
            MainGrid.RowDefinitions[row].Height = show ? GridLength.Auto : new GridLength(0);
            if (row == 0)
                TopRowLayout.IsVisible = show;
        }
        /// <summary>
        /// Handles the page navigated to event (via shell)
        /// </summary>
        /// <param name="args">Event argumennts</param>
        protected override void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            if (!IsInitialized)
            {
                SetBusyIndicators(true);
                Dispatcher.Dispatch(async () =>
                {
                    await Task.Delay(50);
                    try
                    {
                        Initialize();
                    }
                    finally
                    {
                        SetBusyIndicators(false);
                    }
                });
            }
        }
        /// <summary>
        /// Handles the page appearing event
        /// </summary>
        protected override void OnAppearing()
        {
            base.OnAppearing();
            Shell.Current.Navigating += OnShellNavigating;
            if (!IsInitialized)
            {
                // Safety net: call Initialize() if OnNavigatedTo doesn't fire within 1s
                _fallbackInitCts?.Cancel();
                _fallbackInitCts = new CancellationTokenSource();
                var cts = _fallbackInitCts;
                Dispatcher.Dispatch(async () =>
                {
                    try
                    {
                        await Task.Delay(1000, cts.Token);
                    }
                    catch (OperationCanceledException)
                    {
                        return;
                    }
                    if (!IsInitialized && Shell.Current.CurrentPage == this)
                    {
                        SetBusyIndicators(true);
                        await Task.Delay(50);
                        try
                        {
                            Initialize();
                        }
                        finally
                        {
                            SetBusyIndicators(false);
                        }
                    }
                });
            }
        }
        /// <summary>
        /// Handles the page disappearing event
        /// </summary>
        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            Shell.Current.Navigating -= OnShellNavigating;
            if (IsInitialized)
                _fallbackInitCts?.Cancel();
        }
        /// <summary>
        /// Handles the page navigating event (via shell). Checks whether the page is about
        /// to be navigated away from and, if so, prompts the user to save any changes that 
        /// have been made as required
        /// </summary>
        /// <param name="sender">Sender</param>
        /// <param name="e">Event arguments</param>
        private async void OnShellNavigating(object? sender, ShellNavigatingEventArgs e)
        {
            if (_viewModel.IsBusy) return;

            if (_viewModel.CanSave && e.Source == ShellNavigationSource.PopToRoot)
            {
                var deferral = e.GetDeferral();
                bool? choice = await _dialogService.ShowConfirmation(
                    "Unsaved Changes",
                    "Do you want to save your changes?",
                    "Save",
                    "Discard",
                    "Cancel"
                );
                if (choice == null)
                {
                    e.Cancel();
                }
                else if (choice == true)
                {
                    bool saved = await Save();
                    if (!saved)
                        e.Cancel();
                }
                deferral.Complete();
            }
        }
    }
}
