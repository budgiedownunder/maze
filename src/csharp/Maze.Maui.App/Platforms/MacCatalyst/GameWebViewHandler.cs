using Foundation;
using WebKit;

namespace Maze.Maui.App
{
    partial class GameWebViewHandler
    {
        static GameWebViewHandler()
        {
            // Bridge: the /game/ page posts GameResult JSON via
            // window.webkit.messageHandlers.MazeMauiHost.postMessage(...).
            // Surface it on the shared GameResultReceived event.
            // (Identical to the iOS handler — both host a WKWebView.)
            Mapper.AppendToMapping("GameWebViewMessageBridge", (handler, view) =>
            {
                if (handler.PlatformView is not WKWebView webView) return;

                var controller = webView.Configuration.UserContentController;
                // Remove-then-add keeps this idempotent — AddScriptMessageHandler
                // throws if a handler with the same name is already registered.
                controller.RemoveScriptMessageHandler("MazeMauiHost");
                controller.AddScriptMessageHandler(new MazeMauiHostBridge(), "MazeMauiHost");
            });
        }

        /// <summary>
        /// JS-facing bridge exposed as
        /// <c>window.webkit.messageHandlers.MazeMauiHost</c>. Forwards the raw
        /// JSON payload to <see cref="RaiseHostMessage"/>.
        /// </summary>
        private sealed class MazeMauiHostBridge : NSObject, IWKScriptMessageHandler
        {
            public void DidReceiveScriptMessage(
                WKUserContentController userContentController, WKScriptMessage message)
            {
                if (message.Body is NSString json)
                    RaiseHostMessage(json.ToString());
            }
        }
    }
}
