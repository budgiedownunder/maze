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
        /// Invoked by the per-platform handler code when a bridge message
        /// arrives, forwarding it to <see cref="GameResultReceived"/>.
        /// </summary>
        /// <param name="json">Raw GameResult JSON payload</param>
        internal static void RaiseGameResult(string json) => GameResultReceived?.Invoke(json);
    }
}
