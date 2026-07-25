using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The Featured sub-page of the Play 3D browser — the admin-ordered catalogue of
    /// curated games and collections. Loads its first page on first appear; further
    /// pages come from Load-more and pull-to-refresh reloads the list.
    /// </summary>
    public partial class Play3dFeaturedPage : ContentPage
    {
        private readonly Play3dFeaturedViewModel _viewModel;
        private bool _loaded;

        /// <summary>Constructor</summary>
        /// <param name="viewModel">Injected Featured view model</param>
        public Play3dFeaturedPage(Play3dFeaturedViewModel viewModel)
        {
            InitializeComponent();
            _viewModel = viewModel;
            BindingContext = viewModel;
        }

        protected override async void OnAppearing()
        {
            base.OnAppearing();
            if (_loaded)
                return;
            _loaded = true;
            await _viewModel.LoadFirstPageAsync();
        }
    }
}
