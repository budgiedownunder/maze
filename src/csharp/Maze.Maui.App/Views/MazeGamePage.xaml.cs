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
            _viewModel.Cleanup();
        }

        private void SetBusyIndicators(bool busy)
        {
            Pointer.SetCursor(this, busy ? Icon.Wait : Icon.Arrow);
            _viewModel.IsBusy = busy;
        }
    }
}
