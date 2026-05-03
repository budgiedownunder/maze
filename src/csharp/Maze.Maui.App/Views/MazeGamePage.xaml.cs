using Maze.Api;
using Maze.Maui.App.ViewModels;
using Maze.Maui.Controls.Pointer;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Interactive 2D maze game page. The player navigates the maze using arrow keys (Windows)
    /// or D-pad buttons (Android/iOS). Holding a key or D-pad button moves continuously at a
    /// controlled rate; each press also moves one step immediately.
    /// </summary>
    public partial class MazeGamePage : ContentPage
    {
        private const int MoveIntervalMs = 120;

        private readonly MazeGameViewModel _viewModel;
        private bool _gameStarted = false;
        private IDispatcherTimer? _dpadTimer;
        private MazeGameDirection _dpadDirection = MazeGameDirection.None;
        private long _lastMoveTickMs = 0;
        private MazeGameDirection _lastMoveDirection = MazeGameDirection.None;

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
            if (_gameStarted) return;
            _gameStarted = true;
            DpadGrid.IsVisible = false;
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
                    DpadGrid.IsVisible = DeviceInfo.Platform != DevicePlatform.WinUI;
                }
            });
        }

        /// <inheritdoc/>
        protected override void OnAppearing()
        {
            base.OnAppearing();
            Shell.Current.Navigating += OnShellNavigating;
        }

        /// <inheritdoc/>
        protected override void OnNavigatedFrom(NavigatedFromEventArgs args)
        {
            base.OnNavigatedFrom(args);
            if (_viewModel.IsShowingResultPopup) return;
            StopDpad();
            GameGrid.KeyDown -= OnGameGridKeyDown;
            GameGrid.CellTapped -= OnGameGridCellTapped;
            GameGrid.CellDoubleTapped -= OnGameGridCellTapped;
        }

        /// <inheritdoc/>
        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            Shell.Current.Navigating -= OnShellNavigating;
            if (_viewModel.IsShowingResultPopup) return;
            _gameStarted = false;
            DpadGrid.IsVisible = false;
            _viewModel.Cleanup();
        }

        private async void OnShellNavigating(object? sender, ShellNavigatingEventArgs e)
        {
            if (e.Source == ShellNavigationSource.Pop)
            {
                var deferral = e.GetDeferral();
                SetBusyIndicators(true);
                await Task.Delay(50);
                deferral.Complete();
            }
        }

        private void Move(MazeGameDirection direction)
        {
            if (direction == MazeGameDirection.None) return;
            long now = Environment.TickCount64;
            if (direction != _lastMoveDirection)
                _lastMoveTickMs = 0;
            if (now - _lastMoveTickMs < MoveIntervalMs) return;
            _lastMoveTickMs = now;
            _lastMoveDirection = direction;
            _viewModel.Move(direction);
        }

        private void OnGameGridCellTapped(object? _, MazeGridCellTappedEventArgs __) { }

        private void OnGameGridKeyDown(object? sender, MazeGridKeyDownEventArgs e)
        {
            MazeGameDirection dir = e.Key switch
            {
                Controls.Keyboard.Key.Up => MazeGameDirection.Up,
                Controls.Keyboard.Key.Down => MazeGameDirection.Down,
                Controls.Keyboard.Key.Left => MazeGameDirection.Left,
                Controls.Keyboard.Key.Right => MazeGameDirection.Right,
                _ => MazeGameDirection.None
            };
            Move(dir);
        }

        private void OnDpadUpPressed(object? sender, EventArgs e) => StartDpad(MazeGameDirection.Up);
        private void OnDpadDownPressed(object? sender, EventArgs e) => StartDpad(MazeGameDirection.Down);
        private void OnDpadLeftPressed(object? sender, EventArgs e) => StartDpad(MazeGameDirection.Left);
        private void OnDpadRightPressed(object? sender, EventArgs e) => StartDpad(MazeGameDirection.Right);
        private void OnDpadReleased(object? sender, EventArgs e) => StopDpad();

        private void StartDpad(MazeGameDirection direction)
        {
            _dpadDirection = direction;
            Move(direction);
            _dpadTimer ??= CreateDpadTimer();
            _dpadTimer.Start();
        }

        private void StopDpad()
        {
            _dpadTimer?.Stop();
            _dpadDirection = MazeGameDirection.None;
        }

        private IDispatcherTimer CreateDpadTimer()
        {
            var timer = Dispatcher.CreateTimer();
            timer.Interval = TimeSpan.FromMilliseconds(MoveIntervalMs);
            timer.Tick += (_, _) => Move(_dpadDirection);
            return timer;
        }

        private void SetBusyIndicators(bool busy)
        {
            Pointer.SetCursor(this, busy ? Icon.Wait : Icon.Arrow);
            _viewModel.IsBusy = busy;
        }
    }
}
