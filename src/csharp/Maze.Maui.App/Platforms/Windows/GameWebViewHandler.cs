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
        }

        private static async Task ClearCacheOnceAsync(
            Microsoft.UI.Xaml.Controls.WebView2 webView, string key)
        {
            await webView.EnsureCoreWebView2Async();
            await webView.CoreWebView2.Profile.ClearBrowsingDataAsync(
                CoreWebView2BrowsingDataKinds.DiskCache);
            Preferences.Default.Set(key, true);
        }
    }
}
