namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.App.Controls;
    using System.Runtime.CompilerServices;
    using System.Diagnostics;
    using Maze.Maui.App.Services;

    public partial class MainPage : ContentPage
    {
        const String APP_TITLE = "MAZE";
        int count = 0;


        public MainPage()
        {
            InitializeComponent();

            BindingContext = new MainPageViewModel(new DeviceTypeService());

            MazeGrid.CellTapped += OnMazeGridCellTapped;
            MazeGrid.CellDoubleTapped += OnMazeGridCellDoubleTapped;
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

        private void OnCancelBtnClicked(object sender, EventArgs e)
        {
            if (BindingContext is MainPageViewModel viewModel)
                SetSelectRangeMode(false);
        }

        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            MazeGrid.OnCellTapped(e.Cell, e.Row, e.Column, false);
        }

        private void OnMazeGridCellDoubleTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            if (BindingContext is MainPageViewModel viewModel)
            {
                if (MazeGrid.IsExtendedSelectionMode)
                    SetSelectRangeMode(false);

                MazeGrid.OnCellDoubleTapped(e.Cell, e.Row, e.Column, false);
            }
        }

        private void SetSelectRangeMode(bool enable)
        {
            if (enable)
                MazeGrid.EnableExtendedSelection();
            else
                MazeGrid.CancelExtendedSelection();

            ShowSelectRangeButtons(!enable);
        }

        private void ShowSelectRangeButtons(bool show)
        {
            if (BindingContext is MainPageViewModel viewModel)
            {
                bool touchOnly = IsTouchOnlyDevice;
                viewModel.ShowSelectRangeBtn = show && touchOnly;
                viewModel.ShowCancelBtn = !show && touchOnly;
                if (!show)
                    CancelBtn.Text = "Cancel Select Range";
            }
        }

        private void OnMazeGridSelectionChanged(object sender, MazeGridSelectionChangedEventArgs e)
        {
            UpdateControls();
        }

        private void UpdateControls()
        {
            if (BindingContext is MainPageViewModel viewModel)
            {
                bool showSelectRangeButtons = IsTouchOnlyDevice || MazeGrid.IsExtendedSelectionMode;
                bool showTopRowLayout = showSelectRangeButtons;
                viewModel.ShowTopRowLayout = showTopRowLayout;
                if (showTopRowLayout)
                    ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
            }
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
