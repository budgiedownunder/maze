using Microsoft.Web.WebView2.Core;

namespace Maze.Maui.App
{
    partial class GameWebViewHandler
    {
        // Bump the key to force another clear when new WASM builds are deployed.
        private const string CacheClearedKey = "GameWebView2CacheCleared_v2";

        static GameWebViewHandler()
        {
            // Hook CoreWebView2 readiness rather than forcing it with
            // EnsureCoreWebView2Async() from inside the Mapper callback. Forcing
            // early init that way races with MAUI's own WebViewHandler
            // initialization — benign on older WindowsAppSDK builds, but the
            // version shipped with .NET SDK 10.0.300 leaves the WebView blank
            // (no navigation, no page). Letting MAUI drive init and attaching
            // once CoreWebView2 exists avoids the race; the disk-cache clear and
            // the message bridge both run from OnCoreReady.
            Mapper.AppendToMapping("GameWebViewSetup", (handler, view) =>
            {
                var webView = handler.PlatformView;
                webView.CoreWebView2Initialized -= OnCoreWebView2Initialized;
                webView.CoreWebView2Initialized += OnCoreWebView2Initialized;
                // If the mapping happens to run after init, wire up immediately.
                if (webView.CoreWebView2 is not null)
                    OnCoreReady(webView.CoreWebView2);
            });
        }

        private static void OnCoreWebView2Initialized(
            Microsoft.UI.Xaml.Controls.WebView2 sender,
            Microsoft.UI.Xaml.Controls.CoreWebView2InitializedEventArgs e)
        {
            if (e.Exception is null && sender.CoreWebView2 is not null)
                OnCoreReady(sender.CoreWebView2);
        }

        private static void OnCoreReady(CoreWebView2 core)
        {
            // One-time disk-cache clear after a WASM redeploy, guarded by a
            // persisted preference so it runs at most once per key.
            if (!Preferences.Default.Get(CacheClearedKey, false))
                _ = ClearCacheOnceAsync(core);

            // Bridge: the /game/ page posts GameResult JSON via
            // window.chrome.webview.postMessage(...). -= then += keeps this
            // idempotent — OnWebMessageReceived is a static method, so the
            // delegate identity is stable across re-subscribes.
            core.WebMessageReceived -= OnWebMessageReceived;
            core.WebMessageReceived += OnWebMessageReceived;

            // A dead WebView2 process leaves a blank control and reports nothing
            // to the page, so the failure has to be surfaced from here. Desktop
            // has memory to spare and this is not expected to fire — it is the
            // same hazard the mobile platforms close, kept consistent.
            core.ProcessFailed -= OnProcessFailed;
            core.ProcessFailed += OnProcessFailed;
        }

        /// <summary>
        /// Reports a failed WebView2 process as a game failure. WebView2 names
        /// the cause outright, so an out-of-memory kill is distinguished from an
        /// ordinary crash rather than guessed at.
        /// </summary>
        private static void OnProcessFailed(CoreWebView2 sender, CoreWebView2ProcessFailedEventArgs e)
        {
            bool outOfMemory = e.Reason == CoreWebView2ProcessFailedReason.OutOfMemory;
            RaiseHostFailure(new Models.GameFailure
            {
                Reason = outOfMemory
                    ? "The game ran out of memory and had to close."
                    : Models.GameFailure.GenericReason,
                Detail = $"webview2-process-failed kind={e.ProcessFailedKind} reason={e.Reason}",
                Phase = null,
            });
        }

        private static async Task ClearCacheOnceAsync(CoreWebView2 core)
        {
            await core.Profile.ClearBrowsingDataAsync(CoreWebView2BrowsingDataKinds.DiskCache);
            Preferences.Default.Set(CacheClearedKey, true);
        }

        private static void OnWebMessageReceived(
            CoreWebView2 sender, CoreWebView2WebMessageReceivedEventArgs e)
        {
            try
            {
                var json = e.TryGetWebMessageAsString();
                if (!string.IsNullOrEmpty(json))
                    RaiseHostMessage(json);
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
