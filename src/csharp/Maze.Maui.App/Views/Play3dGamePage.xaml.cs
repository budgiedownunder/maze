using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.Views
{
    [QueryProperty(nameof(MazeItem), "MazeItem")]
    public partial class Play3dGamePage : ContentPage
    {
        private readonly ConfigurationService _configurationService;
        private readonly IAuthService _authService;

        public MazeItem? MazeItem { get; set; }

        public Play3dGamePage(ConfigurationService configurationService, IAuthService authService)
        {
            InitializeComponent();
            _configurationService = configurationService;
            _authService = authService;
        }

        protected override async void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            var apiRootUri = _configurationService.ApiRootUri;
            var apiIndex = apiRootUri.LastIndexOf("/api/", StringComparison.Ordinal);
            var gameUrl = apiIndex >= 0
                ? apiRootUri[..apiIndex] + "/game/"
                : apiRootUri + "game/";

            var token = await _authService.GetBearerTokenAsync();
            if (MazeItem is not null)
            {
                var id = Uri.EscapeDataString(MazeItem.ID);
                gameUrl += $"?id={id}";
                if (token is not null) gameUrl += $"&t={token}";
            }
            else if (token is not null)
            {
                gameUrl += $"?t={token}";
            }

            MazeGameWebView.Source = new UrlWebViewSource { Url = gameUrl };
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            MazeItem = null;
            MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };
        }
    }
}
