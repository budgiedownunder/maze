using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Microsoft.Extensions.Logging;

namespace Maze.Maui.App.Views
{
    [QueryProperty(nameof(MazeItem), "MazeItem")]
    [QueryProperty(nameof(DefinitionId), "def")]
    [QueryProperty(nameof(LaunchSettings), "LaunchSettings")]
    public partial class Play3dGamePage : ContentPage
    {
        private readonly ConfigurationService _configurationService;
        private readonly IAuthService _authService;
        private readonly ILogger<Play3dGamePage> _logger;

        public MazeItem? MazeItem { get; set; }

        /// <summary>
        /// Stored game-definition id passed by the Play 3D browser. When set (and
        /// no <see cref="MazeItem"/> is supplied), it is forwarded to the game as
        /// <c>/game/?def=…</c> so the host page fetches the definition's config and
        /// records scores under its <c>def:&lt;id&gt;</c> board.
        /// </summary>
        public string? DefinitionId { get; set; }

        /// <summary>
        /// Per-launch custom settings chosen by the user via the
        /// <see cref="MazeGameSettingsPopup"/>. Only relevant for the
        /// <see cref="MazeItem"/>-driven path (specific stored maze); when
        /// set, the settings are appended to the <c>/game/?id=…</c> URL as
        /// query parameters that <c>/game/index.html</c> reads back when
        /// building the <c>StartConfig</c>.
        /// </summary>
        public MazeGameSettings? LaunchSettings { get; set; }

        public Play3dGamePage(ConfigurationService configurationService, IAuthService authService, ILogger<Play3dGamePage> logger)
        {
            InitializeComponent();
            _configurationService = configurationService;
            _authService = authService;
            _logger = logger;
        }

        protected override void OnAppearing()
        {
            base.OnAppearing();
            GameWebViewHandler.GameResultReceived += OnGameResultReceived;
        }

        protected override async void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            var apiRootUri = _configurationService.ApiRootUri;
            var token = await _authService.GetBearerTokenAsync();

            // Per-launch settings (MazeItem path only) ride as URL params so
            // /game/index.html reads them with priority over localStorage — the
            // MAUI flow uses Preferences, not the SPA's localStorage.
            string gameUrl = MazeItem is not null
                ? Play3dGameHostUrl.BuildForMaze(apiRootUri, MazeItem.ID, token, LaunchSettings?.ToQueryString())
                : !string.IsNullOrEmpty(DefinitionId)
                    ? Play3dGameHostUrl.BuildForDefinition(apiRootUri, DefinitionId, token)
                    : Play3dGameHostUrl.BuildForToken(apiRootUri, token);

            MazeGameWebView.Source = new UrlWebViewSource { Url = gameUrl };
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            GameWebViewHandler.GameResultReceived -= OnGameResultReceived;
            MazeItem = null;
            DefinitionId = null;
            LaunchSettings = null;
            MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };
        }

        /// <summary>
        /// Handles a GameResult posted by the hosted /game/ page via the
        /// platform WebView bridge. Currently diagnostic only — parses the
        /// payload and logs it (debug builds route through the Debug provider).
        /// Leaderboard recording is future work. The bridge may fire on a
        /// non-UI thread (Android), so the handler hops to the main thread.
        /// </summary>
        /// <param name="json">Raw GameResult JSON payload</param>
        private void OnGameResultReceived(string json)
        {
            MainThread.BeginInvokeOnMainThread(() =>
            {
                var result = GameResult.FromJson(json);
                if (result is null)
                {
                    _logger.LogWarning("Play3dGamePage: unparseable GameResult payload: {Json}", json);
                    return;
                }

                if (_logger.IsEnabled(LogLevel.Information))
                {
                    _logger.LogInformation(
                        "Play3dGamePage: GameResult outcome={Outcome} elapsedMs={ElapsedMs} score={Score} difficulty={Difficulty} rows={Rows} cols={Cols} seed={Seed}",
                        result.Outcome, result.ElapsedMs, result.Score, result.Difficulty ?? "(none)", result.Rows, result.Cols, result.Seed);
                }
            });
        }
    }
}
