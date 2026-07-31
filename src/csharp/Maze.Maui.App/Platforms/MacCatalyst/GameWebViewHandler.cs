using Foundation;
using Maze.Maui.App.Models;
using Microsoft.Maui.Handlers;
using Microsoft.Maui.Platform;
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

            // Watch for the web content process dying — see the iOS handler.
            // Kept in step with iOS so the behaviour is there if MacCatalyst is
            // re-enabled in the csproj (it is currently commented out, so this
            // file is not compiled today).
            Mapper.AppendToMapping("GameWebViewProcessWatch", (handler, view) =>
            {
                if (handler.PlatformView is not WKWebView webView) return;
                if (webView.NavigationDelegate is ProcessWatchNavigationDelegate) return;
                webView.NavigationDelegate = new ProcessWatchNavigationDelegate(handler);
            });
        }

        /// <summary>
        /// MAUI's navigation delegate plus the <c>webViewWebContentProcessDidTerminate:</c>
        /// callback it does not implement. Mirrors the iOS handler.
        /// </summary>
        private sealed class ProcessWatchNavigationDelegate : MauiWebViewNavigationDelegate
        {
            public ProcessWatchNavigationDelegate(IWebViewHandler handler) : base(handler) { }

            /// <summary>
            /// The web content process backing this WKWebView was terminated.
            /// </summary>
            /// <param name="webView">The web view whose content process died</param>
            [Export("webViewWebContentProcessDidTerminate:")]
            [System.Diagnostics.CodeAnalysis.SuppressMessage(
                "Style", "IDE0060:Remove unused parameter",
                Justification = "Required by the webViewWebContentProcessDidTerminate: selector — ObjC dispatches the web view as the argument whether or not it is read.")]
            [System.Diagnostics.CodeAnalysis.SuppressMessage(
                "Performance", "CA1822:Mark members as static",
                Justification = "Must be an instance method — ObjC dispatches the selector on the navigation-delegate instance.")]
            public void WebViewWebContentProcessDidTerminate(WKWebView webView) =>
                RaiseHostFailure(new GameFailure
                {
                    Reason = "The game ran out of memory and had to close.",
                    Detail = "web-content-process-terminated",
                    Phase = null,
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
