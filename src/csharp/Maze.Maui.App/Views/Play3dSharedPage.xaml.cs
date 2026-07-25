using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Shared with me — 3D games and collections other users have shared with the
    /// caller, as a Games / Collections tabbed browser over the reusable
    /// <see cref="Play3dScopeBrowserView"/>.
    /// </summary>
    public partial class Play3dSharedPage : ContentPage
    {
        private readonly Play3dScopeBrowserViewModel _viewModel;
        private bool _loaded;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service</param>
        public Play3dSharedPage(IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService)
        {
            InitializeComponent();
            _viewModel = new Play3dScopeBrowserViewModel(gameLibrary, navigationService, dialogService, GameListScope.Shared, "Shared with me");
            BindingContext = _viewModel;
        }

        protected override async void OnAppearing()
        {
            base.OnAppearing();
            if (_loaded)
                return;
            _loaded = true;
            await _viewModel.LoadAsync();
        }
    }
}
