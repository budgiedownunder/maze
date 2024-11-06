namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.App.Controls;
    using Maze.Maui.App.Services;

    public partial class MainPage : ContentPage
    {
        const String APP_TITLE = "MAZE";
        int count = 0;

        public MainPage()
        {
            InitializeComponent();
            IDeviceTypeService deviceTypeService = new DeviceTypeService();

            BindingContext = new MainPageViewModel(deviceTypeService);

            MazeGrid.Initialize(!deviceTypeService.IsTouchOnlyDevice());
            MazeGrid.CellTapped += OnMazeGridCellTapped;
            MazeGrid.CellDoubleTapped += OnMazeGridCellDoubleTapped;
            MazeGrid.KeyDown += OnMazeGridKeyDown;
            MazeGrid.SelectionChanged += OnMazeGridSelectionChanged;

            UpdateControls();
        }

        private bool IsTouchOnlyDevice
        {
            get
            {
                bool touchOnly = false;
                if (BindingContext is MainPageViewModel viewModel)
                    touchOnly = viewModel.IsTouchOnlyDevice;
                return touchOnly;
            }
        }

        private void OnSelectRangeBtnClicked(object sender, EventArgs e)
        {
            if (BindingContext is MainPageViewModel viewModel)
                SetSelectRangeMode(true);
        }

        private void OnDoneBtnClicked(object sender, EventArgs e)
        {
            if (BindingContext is MainPageViewModel viewModel)
                SetSelectRangeMode(false);
        }

        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            MazeGrid.OnCellTapped(e.Cell, false);
        }

        private void OnMazeGridCellDoubleTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            if (BindingContext is MainPageViewModel viewModel)
            {
                bool inExtendedSelectionMode = MazeGrid.IsExtendedSelectionMode;
                if (IsTouchOnlyDevice && inExtendedSelectionMode)
                    SetSelectRangeMode(false);
                MazeGrid.OnCellDoubleTapped(e.Cell, false);
                if (IsTouchOnlyDevice && !inExtendedSelectionMode)
                    SetSelectRangeMode(true);
            }
        }

        private void OnMazeGridKeyDown(object sender, MazeGridKeyDownEventArgs e)
        {
            switch(e.Key)
            {
                case Controls.Keyboard.Key.W:
                    ChangeSelectionToWall();
                    break;
                case Controls.Keyboard.Key.S:
                    ChangeSelectionToStart();
                    break;
                case Controls.Keyboard.Key.F:
                    ChangeSelectionToFinish();
                    break;
                case Controls.Keyboard.Key.Delete:
                    DeleteSelection();
                    break;
                default:
                    MazeGrid.OnProcessKeyDown(e.KeyState, e.Key, false);
                    break;
            }
        }

        private void OnSetWallBtnClicked(object sender, EventArgs e)
        {
            ChangeSelectionToWall();
        }

        private void ChangeSelectionToWall()
        {
            ChangeSelectedCellsContent(Maze.CellType.Wall);
        }

        private void OnSetStartBtnClicked(object sender, EventArgs e)
        {
            ChangeSelectionToStart();
        }

        private void ChangeSelectionToStart()
        {
            ChangeSelectedCellsContent(Maze.CellType.Start);
        }

        private void OnSetFinishBtnClicked(object sender, EventArgs e)
        {
            ChangeSelectionToFinish();
        }

        private void ChangeSelectionToFinish()
        {
            ChangeSelectedCellsContent(Maze.CellType.Finish);
        }

        private void OnClearBtnClicked(object sender, EventArgs e)
        {
            DeleteSelection();
        }

        private void DeleteSelection()
        {
            ChangeSelectedCellsContent(Maze.CellType.Empty);
        }


        private void ChangeSelectedCellsContent(Maze.CellType newCellType)
        {
            MazeGrid.SetSelectionContent(newCellType);
            EnableExtendedSelectionMode(false);
            UpdateControls();
        }


        private void SetSelectRangeMode(bool enable)
        {
            EnableExtendedSelectionMode(enable);
            ShowSelectRangeButtons(!enable);
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


        private void ShowSelectRangeButtons(bool show)
        {
            bool touchOnly = IsTouchOnlyDevice;
            bool showSelectRangeBtn = show && touchOnly;
            bool showDoneBtn = !show && touchOnly;

            ShowButton(SelectRangeBtn, showSelectRangeBtn, "Select Range");
            ShowButton(DoneBtn, showDoneBtn, "Done");
        }

        private void ShowCellEditButtons(bool haveSelection)
        {
            CellStatus status = MazeGrid.GetCurrentSelectionStatus();

            ShowButton(SetWalllBtn, !status.IsAllWalls, "Wall");
            ShowButton(SetStartBtn, status.IsSingleCell && !status.IsStart, "Start");
            ShowButton(SetFinishBtn, status.IsSingleCell && !status.IsFinish, "Finish");
        }

        private void ShowButton(Button button, bool show, string text)
        {
            button.Text = show ? text : null;
            button.IsVisible = show;
            button.InputTransparent = !show;
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
                ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
                ShowCellEditButtons(haveSelection);
            }
        }

        private void ShowMainGridRow(int row, bool show)
        {
            MainGrid.RowDefinitions[row].Height = show ? GridLength.Auto : new GridLength(0);
        }

        private void OnCounterClicked(object sender, EventArgs e)
        {
            count += 1;
            using (Maze maze = new Maze(10, 20))
            {
                if (count == 1)
                    CounterBtn.Text = $"Clicked {count} time (maze size = {maze.RowCount} rows x {maze.ColCount} columns";
                else
                    CounterBtn.Text = $"Clicked {count} times (maze size = {maze.RowCount} rows x {maze.ColCount} columns";

                SemanticScreenReader.Announce(CounterBtn.Text);
            }
        }

        private void OnResetBtn_Clicked(object sender, EventArgs e)
        {
            DisplayAlert(APP_TITLE, "Reset", "OK");
        }
    }
}
