using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.Views
{
    [QueryProperty(nameof(MazeItem), "MazeItem")]
    [QueryProperty(nameof(DifficultyValue), "difficulty")]
    public partial class Play3dGamePage : ContentPage
    {
        private readonly ConfigurationService _configurationService;
        private readonly IAuthService _authService;

        public MazeItem? MazeItem { get; set; }

        /// <summary>
        /// Difficulty token (e.g. "easy" / "tricky" / "hard") passed by the
        /// Play 3D entry points. When set (and no <see cref="MazeItem"/> is
        /// supplied), it is forwarded to the game as <c>/game/?difficulty=…</c>
        /// so the server resolves the maze-size / timer / seed preset.
        /// </summary>
        public string? DifficultyValue { get; set; }

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
                // Specific stored maze — id path, difficulty not consulted.
                var id = Uri.EscapeDataString(MazeItem.ID);
                gameUrl += $"?id={id}";
                if (token is not null) gameUrl += $"&t={token}";
            }
            else if (!string.IsNullOrEmpty(DifficultyValue))
            {
                gameUrl += $"?difficulty={Uri.EscapeDataString(DifficultyValue)}";
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
            DifficultyValue = null;
            MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };
        }
    }
}
