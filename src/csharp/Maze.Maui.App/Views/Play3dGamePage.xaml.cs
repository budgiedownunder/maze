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

        /// <summary>
        /// Whether the leaderboard has already been opened for this run. The
        /// end-of-run overlay stays on screen while the navigation runs, so a
        /// second tap must not queue a second one.
        /// </summary>
        private bool _leaderboardRequested;

        /// <summary>
        /// Longest to wait for the game to confirm it has torn down before
        /// giving up and destroying the document anyway. Only a fallback: the
        /// game reports completion, so this exists so an unresponsive page
        /// cannot wedge the back navigation, not as the expected path.
        /// </summary>
        private const int StopTimeoutMs = 2000;

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
            GameWebViewHandler.GameLeaderboardRequested -= OnGameLeaderboardRequested;
            GameWebViewHandler.GameLeaderboardRequested += OnGameLeaderboardRequested;
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
            gameUrl = Play3dGameHostUrl.AppendParameter(gameUrl, "mem=1");
#endif
            // On a phone or tablet, ask the game for the settings measured to
            // matter there. What the mode implies is the game's decision, not
            // the app's; all this does is say where it is running — which MAUI
            // knows for certain, where a browser would have to infer it.
            if (Play3dGameHostUrl.IsMobilePlatform())
            {
                gameUrl = Play3dGameHostUrl.AppendParameter(gameUrl, "mobile_mode=1");
            }

            MazeGameWebView.Source = new UrlWebViewSource { Url = gameUrl };
        }

        /// <summary>
        /// Gives the WebView the platform's keyboard focus once the game page has
        /// loaded. The page focuses its own canvas — which is where Bevy listens —
        /// but a native control that does not itself hold keyboard focus is sent
        /// no keystrokes, so without this the player has to click the game before
        /// it answers the keyboard.
        ///
        /// Dispatched rather than called inline so the native handlers are
        /// attached by the time <c>Focus</c> runs, as the app's popups do. Skipped
        /// for a failed load and for the teardown navigation to
        /// <c>about:blank</c> — neither is a game to type into.
        /// </summary>
        /// <param name="sender">The WebView</param>
        /// <param name="e">The navigation outcome and destination</param>
        private void OnWebViewNavigated(object? sender, WebNavigatedEventArgs e)
        {
            if (e.Result != WebNavigationResult.Success) return;
            if (string.IsNullOrEmpty(e.Url) || e.Url.StartsWith("about:", StringComparison.OrdinalIgnoreCase)) return;
            Dispatcher.Dispatch(() => MazeGameWebView.Focus());
        }

        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            GameWebViewHandler.GameResultReceived -= OnGameResultReceived;
            GameWebViewHandler.GameFailureReceived -= OnGameFailureReceived;
            GameWebViewHandler.GameLeaderboardRequested -= OnGameLeaderboardRequested;
            MazeItem = null;
            DefinitionId = null;
            LaunchSettings = null;
            // Released asynchronously — see ReleaseGameAsync for why the order
            // matters. Deliberately not awaited: OnDisappearing must not block
            // navigation, and the release is best-effort by nature.
            _ = ReleaseGameAsync();
        }

        /// <summary>
        /// Releases the running game, in the order that actually frees things.
        ///
        /// First ask the game itself to shut down, which drops the Bevy app and
        /// returns its memory. The request is *polled* — a system turns it into
        /// an app exit on a later frame — so navigating away immediately would
        /// defeat it; wait for the game to confirm instead of guessing at a delay.
        ///
        /// Only then destroy the document (<c>about:blank</c>) and the platform
        /// WebView (<c>DisconnectHandler</c>). Doing those first leaves the
        /// game's release to the browser engine reclaiming the document, which on
        /// iOS is not prompt enough to stop a second launch competing with it.
        /// </summary>
        private async Task ReleaseGameAsync()
        {
            var stopped = new TaskCompletionSource();
            // A local function so the subscription captures only the completion
            // source, never the page — this is a static event, and a stray
            // reference here would pin the very WebView being released.
            void OnStopped() => stopped.TrySetResult();

            GameWebViewHandler.GameStoppedReceived += OnStopped;
            try
            {
                await MazeGameWebView.EvaluateJavaScriptAsync(
                    "window.__mazeStop && window.__mazeStop()");
                if (await Task.WhenAny(stopped.Task, Task.Delay(StopTimeoutMs)) != stopped.Task)
                {
                    _logger.LogWarning(
                        "Play3dGamePage: the game did not confirm teardown within {Timeout}ms; tearing down anyway",
                        StopTimeoutMs);
                }
            }
            catch (Exception ex)
            {
                // The page may already be gone, or the bridge unavailable — the
                // teardown below still has to happen, so this is logged and
                // stepped over rather than surfaced.
                _logger.LogDebug(ex, "Play3dGamePage: could not ask the game to stop before teardown");
            }
            finally
            {
                GameWebViewHandler.GameStoppedReceived -= OnStopped;
            }

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
        /// Opens this run's leaderboard, asked for from the hosted page's end-of-run
        /// overlay.
        ///
        /// The game page is <b>replaced</b> rather than left under the board: the run
        /// is over, leaving tears the WebView down, and <c>_launchAttempted</c> means
        /// coming back to this page would show a blank one — so this pops to wherever
        /// the game was launched from and pushes the Leaderboards page there instead.
        ///
        /// The board is named from this page's own launch arguments — the same values
        /// that built the game URL — read before the pop, which clears them. The
        /// bridge may fire on a non-UI thread (Android), so this hops to the main
        /// thread.
        /// </summary>
        private void OnGameLeaderboardRequested()
        {
            MainThread.BeginInvokeOnMainThread(async () =>
            {
                if (_leaderboardRequested) return;

                string key = MazeItem is not null ? "maze" : "def";
                string? subject = MazeItem is not null ? MazeItem.ID : DefinitionId;
                if (string.IsNullOrEmpty(subject))
                {
                    // No stable subject → no board to open. The overlay only offers
                    // the button when there is one, so this is belt and braces.
                    _logger.LogWarning("Play3dGamePage: leaderboard requested for a run with no board subject");
                    return;
                }
                _leaderboardRequested = true;

                await _navigationService.GoBackAsync();
                await _navigationService.GoToAsync(
                    "LeaderboardsPage",
                    new Dictionary<string, object> { { key, subject } });
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
