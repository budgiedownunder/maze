using Maze.Maui.App.Services;

namespace Maze.Maui.App.Views
{
    public partial class Play3dGamePage : ContentPage
    {
        private readonly ConfigurationService _configurationService;

        public Play3dGamePage(ConfigurationService configurationService)
        {
            InitializeComponent();
            _configurationService = configurationService;
        }

        protected override void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            var apiRootUri = _configurationService.ApiRootUri;
            var apiIndex = apiRootUri.LastIndexOf("/api/", StringComparison.Ordinal);
            var gameUrl = apiIndex >= 0
                ? apiRootUri[..apiIndex] + "/game/"
                : apiRootUri + "game/";
            MazeGameWebView.Source = new UrlWebViewSource { Url = gameUrl };
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };
        }
    }
}
