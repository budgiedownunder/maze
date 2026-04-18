using Maze.Api;
using Maze.Maui.App.ViewModels;
using Maze.Maui.Controls.Pointer;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Interactive 2D maze game page. The player navigates the maze using arrow keys (Windows)
    /// or D-pad buttons (Android/iOS).
    /// </summary>
    public partial class MazeGamePage : ContentPage
    {
        private readonly MazeGameViewModel _viewModel;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">Injected game view model</param>
        public MazeGamePage(MazeGameViewModel viewModel)
        {
            InitializeComponent();
            _viewModel = viewModel;
            BindingContext = viewModel;
        }

        /// <inheritdoc/>
        protected override void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            GameGrid.KeyDown += OnGameGridKeyDown;
            GameGrid.CellTapped += OnGameGridCellTapped;
            GameGrid.CellDoubleTapped += OnGameGridCellTapped;
            SetBusyIndicators(true);
            Dispatcher.Dispatch(async () =>
            {
                await Task.Delay(50);
                try
                {
                    _viewModel.StartGame(GameGrid);
                }
                finally
                {
                    SetBusyIndicators(false);
                }
            });
        }

        /// <inheritdoc/>
        protected override void OnNavigatedFrom(NavigatedFromEventArgs args)
        {
            base.OnNavigatedFrom(args);
            GameGrid.KeyDown -= OnGameGridKeyDown;
            GameGrid.CellTapped -= OnGameGridCellTapped;
            GameGrid.CellDoubleTapped -= OnGameGridCellTapped;
            _viewModel.Cleanup();
        }

        private void OnGameGridCellTapped(object? _, MazeGridCellTappedEventArgs __) { }

        private void OnGameGridKeyDown(object? sender, MazeGridKeyDownEventArgs e)
        {
            MazeGameDirection dir = e.Key switch
            {
                Controls.Keyboard.Key.Up    => MazeGameDirection.Up,
                Controls.Keyboard.Key.Down  => MazeGameDirection.Down,
                Controls.Keyboard.Key.Left  => MazeGameDirection.Left,
                Controls.Keyboard.Key.Right => MazeGameDirection.Right,
                _ => MazeGameDirection.None
            };
            _viewModel.Move(dir);
        }

        private void SetBusyIndicators(bool busy)
        {
            Pointer.SetCursor(this, busy ? Icon.Wait : Icon.Arrow);
            _viewModel.IsBusy = busy;
        }
    }
}
