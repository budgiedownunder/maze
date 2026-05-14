using Microsoft.Web.WebView2.Core;

namespace Maze.Maui.App
{
    partial class GameWebViewHandler
    {
        // Bump the key to force another clear when new WASM builds are deployed.
        private const string CacheClearedKey = "GameWebView2CacheCleared_v2";

        static GameWebViewHandler()
        {
            Mapper.AppendToMapping("GameWebViewCacheClear", (handler, view) =>
            {
                if (Preferences.Default.Get(CacheClearedKey, false)) return;
                _ = ClearCacheOnceAsync(handler.PlatformView, CacheClearedKey);
            });

            // Bridge: the /game/ page posts GameResult JSON via
            // window.chrome.webview.postMessage(...). Surface it on the shared
            // GameResultReceived event for Play3dGamePage to consume.
            Mapper.AppendToMapping("GameWebViewMessageBridge", (handler, view) =>
            {
                _ = WireMessageBridgeAsync(handler.PlatformView);
            });
        }

        private static async Task ClearCacheOnceAsync(
            Microsoft.UI.Xaml.Controls.WebView2 webView, string key)
        {
            await webView.EnsureCoreWebView2Async();
            await webView.CoreWebView2.Profile.ClearBrowsingDataAsync(
                CoreWebView2BrowsingDataKinds.DiskCache);
            Preferences.Default.Set(key, true);
        }

        private static async Task WireMessageBridgeAsync(Microsoft.UI.Xaml.Controls.WebView2 webView)
        {
            await webView.EnsureCoreWebView2Async();
            // -= then += keeps this idempotent if the mapping fires more than once
            // for the same WebView (OnWebMessageReceived is a static method, so
            // the delegate identity is stable).
            webView.CoreWebView2.WebMessageReceived -= OnWebMessageReceived;
            webView.CoreWebView2.WebMessageReceived += OnWebMessageReceived;
        }

        private static void OnWebMessageReceived(
            CoreWebView2 sender, CoreWebView2WebMessageReceivedEventArgs e)
        {
            try
            {
                var json = e.TryGetWebMessageAsString();
                if (!string.IsNullOrEmpty(json))
                    RaiseGameResult(json);
            }
            catch (Exception)
            {
                // TryGetWebMessageAsString throws if the message isn't a string.
                // The /game/ page only ever posts JSON strings, so a non-string
                // message is not ours — ignore it rather than crash the bridge.
            }
        }
    }
}
