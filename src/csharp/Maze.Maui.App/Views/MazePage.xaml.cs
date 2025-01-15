
namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.Services;
    using Maze.Maui.App.Models;
    using Maze.Maui.App.Services;

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
            _viewModel.ClearSolutionRequested += (s, e) => { ClearSolution(); };
            _viewModel.SaveRequested += async (s, e) => { await Save(); };
            _viewModel.RefreshRequested += async (s, e) => { await Refresh(); };
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

            UpdateControls();

            IsInitialized = true;
        }
        /// <summary>
        /// Resets the page display (e.g following refresh)
        /// </summary>
        private void ResetDisplay()
        {
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
            try
            {
                Maze maze = MazeGrid.ToMaze();
                Solution solution = maze.Solve();

                IsSolutionDisplayed = MazeGrid.DisplaySolution(solution);
                UpdateControls();
            }
            catch (Exception ex)
            {
                _dialogService.ShowAlert(APP_TITLE, $"Unable to solve maze\n\nReason: {ex.Message}", "OK");
            }
        }
        /// <summary>
        /// Triggers the save maze process
        /// </summary>
        private Task<bool> Save()
        {
            return _viewModel.SaveMaze(MazeGrid.ToMaze());
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
                    ResetDisplay();
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert(APP_TITLE, $"Failed to refresh maze\n\nReason: {ex.Message}", "OK");
            }
            return refreshed;
        }
        /// <summary>
        /// Clears the maze solution that is displayed (if any) and resets toolbar/buttons states appropriately
        /// </summary>
        private void ClearSolution()
        {
            IsSolutionDisplayed = !MazeGrid.ClearLastSolution();
            UpdateControls();
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

            _viewModel.CanSetWall = !status.IsAllWalls && !IsSolutionDisplayed;
            _viewModel.CanSetStart = status.IsSingleCell && !status.IsStart && !IsSolutionDisplayed;
            _viewModel.CanSetFinish = status.IsSingleCell && !status.IsFinish && !IsSolutionDisplayed;
            _viewModel.CanClear = !status.IsEmpty && !IsSolutionDisplayed;
        }
        /// <summary>
        /// Adjusts the row and column edit flags based on the cells that are selected
        /// </summary>
        private void ShowEditRowColumnButtons()
        {
            bool allRowsSelected = MazeGrid.AllRowsSelected;
            bool allColumnsSelected = MazeGrid.AllColumnsSelected;

            _viewModel.CanInsertRows = allColumnsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteRows = allColumnsSelected && !allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanInsertColumns = allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteColumns = allRowsSelected && !allColumnsSelected && !IsSolutionDisplayed;
        }
        /// <summary>
        /// Adjusts the select range visibility flags based on whether visibility is required and a solution is displayed
        /// </summary>
        /// <param name="show">Show?</param>
        private void ShowSelectRangeButtons(bool show)
        {
            bool touchOnly = IsTouchOnlyDevice;
            bool showSelectRangeBtn = show && touchOnly && !IsSolutionDisplayed;
            bool showDoneBtn = !show && touchOnly && !IsSolutionDisplayed;

            _viewModel.CanSelectRange = showSelectRangeBtn;
            _viewModel.CanShowDone = showDoneBtn;
        }
        /// <summary>
        /// Adjusts the solve-related flags based on whether solve is supported and a solution is displayed
        /// </summary>
        private void ShowSolveButtons()
        {
            _viewModel.CanSolve = IsSolveSupported && !IsSolutionDisplayed;
            _viewModel.CanClearSolution = IsSolveSupported && IsSolutionDisplayed;
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
        /// Updates the control visibility/enabled states
        /// </summary>
        private void UpdateControls()
        {
            bool showSelectRangeButtons = IsTouchOnlyDevice || MazeGrid.IsExtendedSelectionMode;
            bool haveSelection = MazeGrid.ActiveCell is not null;
            bool showTopRowLayout = showSelectRangeButtons || haveSelection;
            ShowMainGridRow(0, showTopRowLayout);
            if (showTopRowLayout)
            {
                ShowCellEditButtons();
                ShowEditRowColumnButtons();
                ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
                ShowSolveButtons();
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
            if (!IsInitialized)
                Initialize();
            base.OnNavigatedTo(args);
        }
        /// <summary>
        /// Handles the page appearing event
        /// </summary>
        protected override void OnAppearing()
        {
            base.OnAppearing();
            Shell.Current.Navigating += OnShellNavigating;
        }
        /// <summary>
        /// Handles the page disappearing event
        /// </summary>
        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            Shell.Current.Navigating -= OnShellNavigating;
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
                bool saveChanges = await _dialogService.ShowConfirmation(
                    "Unsaved Changes",
                    "Do you want to save your changes?",
                    "Yes",
                    "No"
                );
                if (saveChanges)
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
