
namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.Controls;
    using Maze.Maui.Services;
    using Maze.Maui.App.Models;

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
        const String APP_TITLE = "MAZE";
        MazePageViewModel _viewModel;
        MazesViewModel _mazesViewModel;

        private bool IsInitialized { get; set; }
        private bool IsPanSupported { get; set; }
        public MazeItem? MazeItem { get; set; }

        /// <summary>
        /// Constructor 
        /// </summary>
        public MazePage(MazePageViewModel viewModel, MazesViewModel mazesViewModel)
        {
            InitializeComponent();
            IDeviceTypeService deviceTypeService = new DeviceTypeService();
            IsPanSupported = !deviceTypeService.IsTouchOnlyDevice();
            BindingContext = viewModel;
            _viewModel = viewModel;
            _mazesViewModel = mazesViewModel;
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
            _mazesViewModel = mazesViewModel;
        }

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

        private void ResetDisplay()
        {
            MazeGrid.Initialize(IsPanSupported, MazeItem);
            MazeGrid.ActivateCell(1, 1, false);
            UpdateControls();
        }

        private bool IsTouchOnlyDevice { get => _viewModel.IsTouchOnlyDevice; }

        private bool IsSolveSupported { get => OperatingSystem.IsWindows() || OperatingSystem.IsAndroid() || OperatingSystem.IsIOS(); }

        private bool IsSolutionDisplayed { get; set; } = false;

        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            Initialize();
            MazeGrid.OnCellTapped(e.Cell, false);
        }

        private void OnMazeGridCellDoubleTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            bool inExtendedSelectionMode = MazeGrid.IsExtendedSelectionMode;
            if (IsTouchOnlyDevice && inExtendedSelectionMode)
                SetSelectRangeMode(false);

            MazeGrid.OnCellDoubleTapped(e.Cell, false);

            if (IsTouchOnlyDevice && !inExtendedSelectionMode)
                SetSelectRangeMode(true);
        }

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

        private void ChangeSelectionToWall()
        {
            ChangeSelectedCellsContent(Maze.CellType.Wall);
        }

        private void ChangeSelectionToStart()
        {
            ChangeSelectedCellsContent(Maze.CellType.Start);
        }

        private void ChangeSelectionToFinish()
        {
            ChangeSelectedCellsContent(Maze.CellType.Finish);
        }

        private void ClearSelection()
        {
            ChangeSelectedCellsContent(Maze.CellType.Empty);
        }

        private void ChangeSelectedCellsContent(Maze.CellType newCellType)
        {
            MazeGrid.SetSelectionContent(newCellType);
            ExitSelectionModeAndUpdateControls();
        }

        private void DeleteRows()
        {
            MazeGrid.DeleteSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }

        private void DeleteColumns()
        {
            MazeGrid.DeleteSelectedColumns();
            ExitSelectionModeAndUpdateControls();
        }

        private void InsertRows()
        {
            MazeGrid.InsertSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }

        private void InsertColumns()
        {
            MazeGrid.InsertSelectedColumns(); ;
            ExitSelectionModeAndUpdateControls();
        }

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
                DisplayAlert(APP_TITLE, $"Unable to solve maze\n\nReason: {ex.Message}", "OK");
            }
        }

        private Task<bool> Save()
        {
            return _viewModel.SaveMaze(MazeGrid.ToMaze());
        }

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
                await DisplayAlert(APP_TITLE, $"Failed to refresh maze\n\nReason: {ex.Message}", "OK");
            }
            return refreshed;
        }

        private void ClearSolution()
        {
            IsSolutionDisplayed = !MazeGrid.ClearLastSolution();
            UpdateControls();
        }

        private void SetSelectRangeMode(bool enable)
        {
            EnableExtendedSelectionMode(enable);
            ShowSelectRangeButtons(!enable);
        }

        private void ExitSelectionModeAndUpdateControls()
        {
            EnableExtendedSelectionMode(false);
            UpdateControls();
        }

        private void EnableExtendedSelectionMode(bool enable)
        {
            if (MazeGrid.IsExtendedSelectionMode == enable)
                return;

            if (enable)
                MazeGrid.EnableExtendedSelection();
            else
                MazeGrid.CancelExtendedSelection();
        }

        private void ShowCellEditButtons(bool haveSelection)
        {
            CellStatus status = MazeGrid.GetCurrentSelectionStatus();

            _viewModel.CanSetWall = !status.IsAllWalls && !IsSolutionDisplayed;
            _viewModel.CanSetStart = status.IsSingleCell && !status.IsStart && !IsSolutionDisplayed;
            _viewModel.CanSetFinish = status.IsSingleCell && !status.IsFinish && !IsSolutionDisplayed;
            _viewModel.CanClear = !status.IsEmpty && !IsSolutionDisplayed;
        }

        private void ShowEditRowColumnButtons()
        {
            bool allRowsSelected = MazeGrid.AllRowsSelected;
            bool allColumnsSelected = MazeGrid.AllColumnsSelected;

            _viewModel.CanInsertRows = allColumnsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteRows = allColumnsSelected && !allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanInsertColumns = allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteColumns = allRowsSelected && !allColumnsSelected && !IsSolutionDisplayed;
        }

        private void ShowSelectRangeButtons(bool show)
        {
            bool touchOnly = IsTouchOnlyDevice;
            bool showSelectRangeBtn = show && touchOnly && !IsSolutionDisplayed;
            bool showDoneBtn = !show && touchOnly && !IsSolutionDisplayed;

            _viewModel.CanSelectRange = showSelectRangeBtn;
            _viewModel.CanShowDone = showDoneBtn;
        }


        private void ShowSolveButtons()
        {
            _viewModel.CanSolve = IsSolveSupported && !IsSolutionDisplayed;
            _viewModel.CanClearSolution = IsSolveSupported && IsSolutionDisplayed;
        }

        private void OnMazeGridSelectionChanged(object sender, MazeGridSelectionChangedEventArgs e)
        {
            UpdateControls();
        }

        private void UpdateControls()
        {
            bool showSelectRangeButtons = IsTouchOnlyDevice || MazeGrid.IsExtendedSelectionMode;
            bool haveSelection = MazeGrid.ActiveCell is not null;
            bool showTopRowLayout = showSelectRangeButtons || haveSelection;
            ShowMainGridRow(0, showTopRowLayout);
            if (showTopRowLayout)
            {
                ShowCellEditButtons(haveSelection);
                ShowEditRowColumnButtons();
                ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
                ShowSolveButtons();
            }
        }

        private void ShowMainGridRow(int row, bool show)
        {
            MainGrid.RowDefinitions[row].Height = show ? GridLength.Auto : new GridLength(0);
            if (row == 0)
                TopRowLayout.IsVisible = show;
        }

        protected override void OnNavigatedTo(NavigatedToEventArgs args)
        {
            if (!IsInitialized)
                Initialize();
            base.OnNavigatedTo(args);
        }

        protected override void OnAppearing()
        {
            base.OnAppearing();

            // Subscribe to Shell's Navigating event
            Shell.Current.Navigating += OnShellNavigating;
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();

            // Unsubscribe to prevent memory leaks
            Shell.Current.Navigating -= OnShellNavigating;
        }

        private async void OnShellNavigating(object? sender, ShellNavigatingEventArgs e)
        {
            if (_viewModel.IsBusy) return;

            if (_viewModel.CanSave && e.Source == ShellNavigationSource.PopToRoot)
            {
                var deferral = e.GetDeferral();
                bool saveChanges = await ShowConfirmation(
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
        // TO DO - move these to a dialog service
        private async Task ShowAlert(string title, string message, string cancel)
        {
            await Shell.Current.DisplayAlert(title, message, cancel);
        }

        private async Task<bool> ShowConfirmation(string title, string message, string accept, string cancel)
        {
            return await Shell.Current.DisplayAlert(title, message, accept, cancel);
        }

    }
}
