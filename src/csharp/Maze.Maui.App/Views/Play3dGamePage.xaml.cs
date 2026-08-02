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
        private readonly IDialogService _dialogService;
        private readonly INavigationService _navigationService;
        private readonly ILogger<Play3dGamePage> _logger;

        /// <summary>
        /// Whether the game URL has already been handed to the WebView. The page
        /// is transient (one instance per navigation), so this guards a second
        /// <c>OnNavigatedTo</c> — which would otherwise silently restart a run the
        /// player did not ask to restart, and would re-launch a game that has just
        /// died.
        /// </summary>
        private bool _launchAttempted;

        /// <summary>
        /// Whether a failure has already been handled for this run. A dying game
        /// can report more than once (the page's own handler, then the platform
        /// noticing the renderer died), and the player should see one alert.
        /// </summary>
        private bool _failureHandled;

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

        public Play3dGamePage(
            ConfigurationService configurationService,
            IAuthService authService,
            IDialogService dialogService,
            INavigationService navigationService,
            ILogger<Play3dGamePage> logger)
        {
            InitializeComponent();
            _configurationService = configurationService;
            _authService = authService;
            _dialogService = dialogService;
            _navigationService = navigationService;
            _logger = logger;
        }

        protected override void OnAppearing()
        {
            base.OnAppearing();
            // -= then += keeps this idempotent. These are *static* events, so a
            // duplicate subscription outlives the page: OnDisappearing removes
            // one delegate, the other keeps a strong reference to this page, and
            // the page keeps its WebView — and its web-content process — alive.
            // Platforms do not agree on whether OnAppearing can fire twice
            // without an intervening OnDisappearing (returning from background is
            // the usual culprit), so this does not rely on them.
            GameWebViewHandler.GameResultReceived -= OnGameResultReceived;
            GameWebViewHandler.GameResultReceived += OnGameResultReceived;
            GameWebViewHandler.GameFailureReceived -= OnGameFailureReceived;
            GameWebViewHandler.GameFailureReceived += OnGameFailureReceived;
        }

        protected override async void OnNavigatedTo(NavigatedToEventArgs args)
        {
            base.OnNavigatedTo(args);
            // Only ever launch once per navigation — see _launchAttempted.
            if (_launchAttempted) return;
            _launchAttempted = true;
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

#if DEBUG
            // Developer diagnostics readout (memory, entity counts, frame rate).
            // Debug-only rather than a setting: appsettings.json ships as a
            // bundled asset, so switching it there would need a rebuild and
            // redeploy anyway — the same cost as this, with more machinery.
            gameUrl += gameUrl.Contains('?') ? "&mem=1" : "?mem=1";
#endif

            MazeGameWebView.Source = new UrlWebViewSource { Url = gameUrl };
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            GameWebViewHandler.GameResultReceived -= OnGameResultReceived;
            GameWebViewHandler.GameFailureReceived -= OnGameFailureReceived;
            MazeItem = null;
            DefinitionId = null;
            LaunchSettings = null;
            // Two steps, and both are needed. Navigating to about:blank destroys
            // the *document*, which is what releases the game's WebAssembly heap.
            // It does not release the WebView itself — on iOS the WKWebView and
            // its web-content process survive, so a second launch adds a second
            // live process rather than reusing the budget of the first. Measured
            // on device: the heap resets between runs while the app still dies on
            // the second launch, before its world is even built.
            //
            // DisconnectHandler tears the platform view down on a known schedule
            // rather than leaving it to a garbage collection that may not happen
            // before the next game starts.
            MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };
            MazeGameWebView.Handler?.DisconnectHandler();
        }

        /// <summary>
        /// Handles a run that died rather than finished — a Rust panic, a script
        /// error, or the WebView's own renderer being killed (the out-of-memory
        /// case on mobile). The run is <b>cancelled, never retried</b>: the player
        /// is told why and returned to where they came from.
        ///
        /// Order matters. The WebView is pointed at <c>about:blank</c> first,
        /// before anything is awaited, because a dead renderer is otherwise liable
        /// to be reloaded — by the platform or by the page being re-entered — and
        /// the player would watch the same run die repeatedly with no explanation.
        /// The bridge may fire on a non-UI thread (Android), so this hops to the
        /// main thread.
        /// </summary>
        /// <param name="failure">The reported failure</param>
        private void OnGameFailureReceived(GameFailure failure)
        {
            MainThread.BeginInvokeOnMainThread(async () =>
            {
                if (_failureHandled) return;
                _failureHandled = true;

                MazeGameWebView.Source = new UrlWebViewSource { Url = "about:blank" };

                _logger.LogError(
                    "Play3dGamePage: game failed — reason={Reason} detail={Detail} phase={Phase} subject={Subject}",
                    failure.Reason,
                    failure.Detail ?? "(none)",
                    failure.Phase ?? "(unknown)",
                    Play3dFailureReport.Subject(MazeItem?.ID, DefinitionId));

                await _dialogService.ShowAlert("Game stopped", Play3dFailureReport.AlertMessage(failure), "OK");
                await _navigationService.GoBackAsync();
            });
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
