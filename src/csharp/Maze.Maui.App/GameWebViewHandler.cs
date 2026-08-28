using Microsoft.Maui.Handlers;

namespace Maze.Maui.App
{
    partial class GameWebViewHandler : WebViewHandler
    {
        internal static bool IgnoreSslErrors { get; set; }

        /// <summary>
        /// Raised when the hosted <c>/game/</c> page posts a GameResult JSON
        /// payload via the platform message bridge — WebView2 <c>postMessage</c>
        /// on Windows, <c>JavascriptInterface</c> on Android,
        /// <c>WKScriptMessageHandler</c> on iOS / MacCatalyst. The argument is
        /// the raw JSON string. May fire on a non-UI thread (Android); the
        /// subscriber is responsible for marshalling.
        /// </summary>
        internal static event Action<string>? GameResultReceived;

        /// <summary>
        /// Raised when a run dies rather than finishes. Two sources feed it: the
        /// hosted <c>/game/</c> page reporting a Rust panic or script error over
        /// the bridge, and the platform handlers reporting that the WebView's own
        /// renderer process died. Carries a parsed <see cref="Models.GameFailure"/>
        /// rather than raw JSON precisely because of that second source — a
        /// renderer death originates natively, with no JSON to hand.
        /// Fires on the same threads as <see cref="GameResultReceived"/>, so the
        /// subscriber is likewise responsible for marshalling.
        /// </summary>
        internal static event Action<Models.GameFailure>? GameFailureReceived;

        /// <summary>
        /// Raised when the game reports that it has finished tearing down and
        /// released its memory. Lets the page wait for the release before
        /// destroying the document, rather than guessing at a delay.
        /// </summary>
        internal static event Action? GameStoppedReceived;

        /// <summary>
        /// Raised when the player asks for this run's leaderboard from the hosted
        /// page's end-of-run overlay. Carries no payload: the page cannot navigate
        /// to a native page, so all it does is ask, and the hosting page already
        /// knows which board the run belongs to. Fires on the same threads as
        /// <see cref="GameResultReceived"/>, so the subscriber marshals.
        /// </summary>
        internal static event Action? GameLeaderboardRequested;

        /// <summary>
        /// Invoked by the per-platform handler code when a bridge message
        /// arrives. The page multiplexes results and failures over one channel,
        /// so the payload's envelope decides which event it becomes — see
        /// <see cref="Models.HostMessage.KindOf"/>.
        /// </summary>
        /// <param name="json">Raw JSON payload from the platform bridge</param>

        internal static void RaiseHostMessage(string json)
        {
            var kind = Models.HostMessage.KindOf(json);
            if (kind == Models.HostMessageKind.Stopped)
            {
                GameStoppedReceived?.Invoke();
                return;
            }
            if (kind == Models.HostMessageKind.Leaderboard)
            {
                GameLeaderboardRequested?.Invoke();
                return;
            }
            if (kind == Models.HostMessageKind.Failure)
            {
                // An unreadable failure payload still has to surface: dropping it
                // would turn a reported crash back into a silent one.
                GameFailureReceived?.Invoke(
                    Models.GameFailure.FromJson(json) ?? Models.GameFailure.Unreadable(json));
                return;
            }
            GameResultReceived?.Invoke(json);
        }

        /// <summary>
        /// Invoked by the per-platform handler code when the WebView itself
        /// fails — its renderer process was killed, so the page is gone and
        /// cannot report anything over the bridge.
        /// </summary>
        /// <param name="failure">The synthesised failure describing what died</param>
        internal static void RaiseHostFailure(Models.GameFailure failure) =>
            GameFailureReceived?.Invoke(failure);
    }
}
