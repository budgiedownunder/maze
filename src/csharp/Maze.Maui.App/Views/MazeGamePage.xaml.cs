using Maze.Api;
using Maze.Maui.App.ViewModels;
using Maze.Maui.Controls.Pointer;
using Maze.Maui.Services;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Interactive 2D maze game page. The player navigates the maze using arrow keys (Windows)
    /// or D-pad buttons (Android/iOS). Holding a key or D-pad button moves continuously at a
    /// controlled rate; each press also moves one step immediately. Keys are auto-collected
    /// when the player walks onto them.
    /// </summary>
    public partial class MazeGamePage : ContentPage
    {
        private const int MoveIntervalMs = 120;
        private const int TickIntervalMs = 16; // ~60Hz; drives door-opening animation

        private readonly MazeGameViewModel _viewModel;
        private readonly IDeviceTypeService _deviceTypeService;
        private bool _gameStarted = false;
        private IDispatcherTimer? _dpadTimer;
        private IDispatcherTimer? _tickTimer;
        private long _lastTickMs = 0;
        private MazeGameDirection _dpadDirection = MazeGameDirection.None;
        private long _lastMoveTickMs = 0;
        private MazeGameDirection _lastMoveDirection = MazeGameDirection.None;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">Injected game view model</param>
        /// <param name="deviceTypeService">Injected device type service (drives the desktop-only keyboard legend)</param>
        public MazeGamePage(MazeGameViewModel viewModel, IDeviceTypeService deviceTypeService)
        {
            InitializeComponent();
            _viewModel = viewModel;
            _deviceTypeService = deviceTypeService;
            BindingContext = viewModel;
        }

        /// <inheritdoc/>
        protected override void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            GameGrid.KeyDown += OnGameGridKeyDown;
            GameGrid.CellTapped += OnGameGridCellTapped;
            GameGrid.CellDoubleTapped += OnGameGridCellTapped;
            _viewModel.TickStartRequested += OnTickStartRequested;
            _viewModel.PauseRequested += OnPauseRequested;
            _viewModel.DamageFlashRequested += OnDamageFlashRequested;
            if (_gameStarted) return;
            _gameStarted = true;
            DpadGrid.IsVisible = false;
            BagStack.IsVisible = false;
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
                    // The keyboard legend is only useful where a physical keyboard
                    // drives the game, so it shows on non-touch (desktop) devices.
                    ShortcutsHint.IsVisible = !_deviceTypeService.IsTouchOnlyDevice();
                    BagStack.IsVisible = true;
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
            if (_viewModel.IsShowingResultPopup || _viewModel.IsShowingPausePopup) return;
            StopDpad();
            StopTickTimer();
            GameGrid.KeyDown -= OnGameGridKeyDown;
            GameGrid.CellTapped -= OnGameGridCellTapped;
            GameGrid.CellDoubleTapped -= OnGameGridCellTapped;
            _viewModel.TickStartRequested -= OnTickStartRequested;
            _viewModel.PauseRequested -= OnPauseRequested;
            _viewModel.DamageFlashRequested -= OnDamageFlashRequested;
        }

        /// <inheritdoc/>
        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            Shell.Current.Navigating -= OnShellNavigating;
            if (_viewModel.IsShowingResultPopup || _viewModel.IsShowingPausePopup) return;
            _gameStarted = false;
            DpadGrid.IsVisible = false;
            BagStack.IsVisible = false;
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
            // Space / Esc toggle pause (mirrors the centre D-pad "||" button).
            if (e.Key is Controls.Keyboard.Key.Space or Controls.Keyboard.Key.Escape)
            {
                if (_viewModel.PauseCommand.CanExecute(null))
                    _viewModel.PauseCommand.Execute(null);
                return;
            }
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

        /// <summary>
        /// Starts the ~60Hz tick timer that drives door-opening animation.
        /// Hooked to <see cref="MazeGameViewModel.TickStartRequested"/>; the
        /// view-model's <see cref="MazeGameViewModel.Tick(double)"/> returns
        /// <c>false</c> once no door is still opening, which stops the timer.
        /// </summary>
        private void OnTickStartRequested()
        {
            _tickTimer ??= CreateTickTimer();
            if (!_tickTimer.IsRunning)
            {
                _lastTickMs = Environment.TickCount64;
                _tickTimer.Start();
            }
        }

        private void StopTickTimer()
        {
            _tickTimer?.Stop();
        }

        /// <summary>
        /// Stops the tick loop when the game is paused. Hooked to
        /// <see cref="MazeGameViewModel.PauseRequested"/>; resume re-arms the
        /// loop via <see cref="MazeGameViewModel.TickStartRequested"/>, which
        /// also reseeds the dt baseline.
        /// </summary>
        private void OnPauseRequested() => StopTickTimer();

        /// <summary>
        /// Flashes the red damage overlay when the player takes a hit. Snaps to a
        /// partial-alpha red, then fades back to transparent. Restarts cleanly on
        /// back-to-back hits by resetting opacity before each fade.
        /// </summary>
        private void OnDamageFlashRequested()
        {
            DamageFlashOverlay.CancelAnimations();
            DamageFlashOverlay.Opacity = 0.4;
            _ = DamageFlashOverlay.FadeToAsync(0, 300);
        }

        private IDispatcherTimer CreateTickTimer()
        {
            var timer = Dispatcher.CreateTimer();
            timer.Interval = TimeSpan.FromMilliseconds(TickIntervalMs);
            timer.Tick += (_, _) =>
            {
                long now = Environment.TickCount64;
                double dt = Math.Clamp(now - _lastTickMs, 1, 100);
                _lastTickMs = now;
                if (!_viewModel.Tick(dt))
                    timer.Stop();
            };
            return timer;
        }

        private void SetBusyIndicators(bool busy)
        {
            Pointer.SetCursor(this, busy ? Icon.Wait : Icon.Arrow);
            _viewModel.IsBusy = busy;
        }
    }
}
