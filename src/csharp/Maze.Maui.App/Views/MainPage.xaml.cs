namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.App.Controls;
    using Maze.Maui.App.Services;

    /// <summary>
    /// This class represents the main page within the application, which provides
    /// functionality to design and solve mazes.
    /// 
    /// <strong>Screenshots (Windows Desktop)</strong>:
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
    ///         <td><img src="/images/screenshots/windows-design.png" width="250"/></td>
    ///         <td><img src="/images/screenshots/windows-solved.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    /// </summary>
    public partial class MainPage : ContentPage
    {
        const String APP_TITLE = "MAZE";
        MainPageViewModel _viewModel;

        public MainPage()
        {
            InitializeComponent();
            IDeviceTypeService deviceTypeService = new DeviceTypeService();

            _viewModel = new MainPageViewModel(deviceTypeService);
            BindingContext = _viewModel;

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

            MazeGrid.Initialize(!deviceTypeService.IsTouchOnlyDevice());
            MazeGrid.CellTapped += OnMazeGridCellTapped;
            MazeGrid.CellDoubleTapped += OnMazeGridCellDoubleTapped;
            MazeGrid.KeyDown += OnMazeGridKeyDown;
            MazeGrid.SelectionChanged += OnMazeGridSelectionChanged;

            MazeGrid.ActivateCell(1, 1, false);

            UpdateControls();
        }

        private bool IsTouchOnlyDevice { get => _viewModel.IsTouchOnlyDevice; }

        private bool IsSolveSupported { get => !IsTouchOnlyDevice;  }

        private bool IsSolutionDisplayed { get; set;  } = false;

        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
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
            bool haveSelection = MazeGrid.ActiveCell != null;
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
    }
}
