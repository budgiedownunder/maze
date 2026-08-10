namespace Maze.Maui.App
{
    partial class GameWebViewHandler
    {
        static GameWebViewHandler()
        {
            Mapper.AppendToMapping("GameWebViewSetup", (handler, view) =>
            {
                if (handler is not GameWebViewHandler gameHandler) return;

                // Always installed — the client carries the renderer-death
                // handling that keeps Android from killing the whole app
                // process, and applies the TLS bypass only when configured.
                gameHandler.PlatformView.SetWebViewClient(
                    new Platforms.Android.GameWebViewClient(gameHandler, IgnoreSslErrors));

                // Bridge: the /game/ page posts GameResult JSON via
                // window.MazeMauiHost.onGameResult(...). Re-adding with the same
                // name replaces the prior registration, so this is idempotent.
                gameHandler.PlatformView.AddJavascriptInterface(
                    new MazeMauiHostBridge(), "MazeMauiHost");
            });
        }

        /// <summary>
        /// JS-facing bridge object exposed as <c>window.MazeMauiHost</c>. The
        /// <c>onGameResult</c> callback runs on the WebView's JS-bridge thread —
        /// <see cref="RaiseHostMessage"/> subscribers must marshal to the UI thread.
        /// </summary>
        private sealed class MazeMauiHostBridge : Java.Lang.Object
        {
            [Android.Webkit.JavascriptInterface]
            [Java.Interop.Export("onGameResult")]
            [System.Diagnostics.CodeAnalysis.SuppressMessage(
                "Performance", "CA1822:Mark members as static",
                Justification = "Must be an instance method — invoked via JNI on the JavascriptInterface object.")]
            public void OnGameResult(string json) => RaiseHostMessage(json);
        }
    }
}
