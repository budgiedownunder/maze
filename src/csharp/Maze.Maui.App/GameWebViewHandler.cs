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
        /// Raised when the hosted <c>/game/</c> page reports that a run died
        /// rather than finished — a Rust panic, an uncaught script error, or a
        /// renderer that ran out of memory. The argument is the raw JSON string;
        /// parse it with <see cref="Models.GameFailure.FromJson"/>. Fires on the
        /// same threads as <see cref="GameResultReceived"/>, so the subscriber is
        /// likewise responsible for marshalling.
        /// </summary>
        internal static event Action<string>? GameFailureReceived;

        /// <summary>
        /// Invoked by the per-platform handler code when a bridge message
        /// arrives. The page multiplexes results and failures over one channel,
        /// so the payload's envelope decides which event it becomes — see
        /// <see cref="Models.HostMessage.KindOf"/>.
        /// </summary>
        /// <param name="json">Raw JSON payload from the platform bridge</param>
        internal static void RaiseHostMessage(string json)
        {
            if (Models.HostMessage.KindOf(json) == Models.HostMessageKind.Failure)
            {
                GameFailureReceived?.Invoke(json);
                return;
            }
            GameResultReceived?.Invoke(json);
        }
    }
}
