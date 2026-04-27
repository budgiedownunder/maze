using System.Runtime.InteropServices;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Web.WebView2.Core;
using WinUIWindow = Microsoft.UI.Xaml.Window;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Windows-specific <see cref="IWebAuthenticatorBroker"/> backed by an in-app
    /// <c>WebView2</c> popup window. The OAuth flow runs entirely inside the
    /// app — no system browser or OS protocol activation is involved — and
    /// the <c>maze-app://oauth-callback</c> redirect is intercepted via the
    /// WebView2's <see cref="CoreWebView2.NavigationStarting"/> event before
    /// the navigation reaches the OS.
    /// </summary>
    /// <remarks>
    /// Flow:
    /// <list type="number">
    ///   <item><description>Open a centred <c>WebView2</c> popup on the UI
    ///     thread, owned by the main MAUI window.</description></item>
    ///   <item><description>Navigate to <c>startUrl</c>; the IdP performs its
    ///     consent dance, and the server's OAuth callback handler renders an
    ///     HTML bridge page that meta-refreshes to
    ///     <c>maze-app://oauth-callback#token=...&amp;...</c>.</description></item>
    ///   <item><description><see cref="CoreWebView2.NavigationStarting"/>
    ///     fires for that custom-scheme navigation. We cancel it so the OS
    ///     protocol handler is never consulted, parse the URL fragment and
    ///     query into a <see cref="OAuthCallbackResult.Properties"/> dict,
    ///     resolve the in-flight task, and close the popup.</description></item>
    ///   <item><description>If the user closes the popup before the redirect
    ///     happens we surface that as a <see cref="TaskCanceledException"/>,
    ///     mirroring MAUI's <c>WebAuthenticator</c> on other platforms.</description></item>
    /// </list>
    /// </remarks>
    internal class WindowsWebAuthenticatorBroker : IWebAuthenticatorBroker
    {
        private const int OAUTH_POPUP_WIDTH = 540;
        private const int OAUTH_POPUP_HEIGHT = 720;

        public Task<OAuthCallbackResult> AuthenticateAsync(Uri startUrl, Uri callbackUrl)
        {
            ArgumentNullException.ThrowIfNull(startUrl);
            ArgumentNullException.ThrowIfNull(callbackUrl);

            var tcs = new TaskCompletionSource<OAuthCallbackResult>(
                TaskCreationOptions.RunContinuationsAsynchronously);

            // WebView2 is a XAML control; creation and configuration must
            // happen on the UI thread. AuthenticateAsync may be invoked from
            // a ViewModel command on a background thread, so marshal first.
            MainThread.BeginInvokeOnMainThread(() =>
            {
                try
                {
                    OpenAuthPopup(startUrl, callbackUrl, tcs);
                }
                catch (Exception ex)
                {
                    tcs.TrySetException(ex);
                }
            });

            return tcs.Task;
        }

        private static void OpenAuthPopup(
            Uri startUrl,
            Uri callbackUrl,
            TaskCompletionSource<OAuthCallbackResult> tcs)
        {
            var window = new WinUIWindow { Title = "Sign in" };
            var webView = new WebView2();
            window.Content = webView;

            var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
            var ownerHwnd = TryGetMauiMainWindowHandle();

            // Establish an owner relationship with the MAUI main window so the
            // popup stays on top of it, minimises with it, and hands focus
            // back when closed. Best-effort — failures here just leave the
            // popup as a peer top-level window, which still works.
            if (ownerHwnd != IntPtr.Zero)
                _ = SetWindowLongPtr(hwnd, GWLP_HWNDPARENT, ownerHwnd);

            var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hwnd);
            var appWindow = Microsoft.UI.Windowing.AppWindow.GetFromWindowId(windowId);
            appWindow.Resize(new Windows.Graphics.SizeInt32
            {
                Width = OAUTH_POPUP_WIDTH,
                Height = OAUTH_POPUP_HEIGHT,
            });
            CenterOnOwner(appWindow, ownerHwnd);

            // User-cancellation: if the popup is closed before NavigationStarting
            // catches the callback URL, surface a TaskCanceledException so the
            // ViewModel's catch block can show a friendly "sign-in cancelled"
            // toast rather than the generic OAuth-failure path.
            window.Closed += (_, _) => tcs.TrySetCanceled();

            window.Activate();

            // Fire-and-forget; failures inside this kick off the same TCS.
            _ = InitWebViewAsync(webView, startUrl, callbackUrl, window, tcs);
        }

        private static async Task InitWebViewAsync(
            WebView2 webView,
            Uri startUrl,
            Uri callbackUrl,
            WinUIWindow window,
            TaskCompletionSource<OAuthCallbackResult> tcs)
        {
            try
            {
                await webView.EnsureCoreWebView2Async();

                webView.CoreWebView2.NavigationStarting += (_, args) =>
                {
                    if (!Uri.TryCreate(args.Uri, UriKind.Absolute, out var navUri))
                        return;
                    if (!IsCallback(navUri, callbackUrl))
                        return;

                    // Cancel BEFORE the OS protocol handler is consulted.
                    // WebView2 fires NavigationStarting for any top-frame
                    // navigation including custom schemes, so this is the
                    // single chokepoint for the maze-app:// redirect.
                    args.Cancel = true;

                    var props = ParseCallbackParams(navUri);
                    tcs.TrySetResult(new OAuthCallbackResult { Properties = props });
                    window.Close();
                };

                webView.Source = startUrl;
            }
            catch (Exception ex)
            {
                tcs.TrySetException(ex);
                try { window.Close(); } catch { /* already closed */ }
            }
        }

        private static bool IsCallback(Uri navUri, Uri callbackUrl) =>
            string.Equals(navUri.Scheme, callbackUrl.Scheme, StringComparison.OrdinalIgnoreCase) &&
            string.Equals(navUri.Host, callbackUrl.Host, StringComparison.OrdinalIgnoreCase);

        /// <summary>
        /// Parse params from BOTH the URL fragment and the query string. The
        /// current server contract puts everything in the fragment
        /// (<c>maze-app://oauth-callback#token=...</c>); parsing both keeps
        /// the broker forwards-compatible if that contract ever changes.
        /// </summary>
        private static Dictionary<string, string> ParseCallbackParams(Uri uri)
        {
            var dict = new Dictionary<string, string>();
            ParseInto(dict, uri.Fragment.TrimStart('#'));
            ParseInto(dict, uri.Query.TrimStart('?'));
            return dict;
        }

        private static void ParseInto(Dictionary<string, string> dict, string source)
        {
            if (string.IsNullOrEmpty(source)) return;
            foreach (var part in source.Split('&', StringSplitOptions.RemoveEmptyEntries))
            {
                var eq = part.IndexOf('=');
                if (eq <= 0) continue;
                var key = Uri.UnescapeDataString(part[..eq]);
                var value = Uri.UnescapeDataString(part[(eq + 1)..]);
                dict[key] = value;
            }
        }

        private static IntPtr TryGetMauiMainWindowHandle()
        {
            try
            {
                var mauiWindow = Microsoft.Maui.Controls.Application.Current?.Windows.FirstOrDefault();
                if (mauiWindow?.Handler?.PlatformView is WinUIWindow nativeWindow)
                    return WinRT.Interop.WindowNative.GetWindowHandle(nativeWindow);
            }
            catch
            {
                // Best-effort — fall through to IntPtr.Zero.
            }
            return IntPtr.Zero;
        }

        private static void CenterOnOwner(
            Microsoft.UI.Windowing.AppWindow appWindow,
            IntPtr ownerHwnd)
        {
            try
            {
                if (ownerHwnd == IntPtr.Zero) return;
                if (!GetWindowRect(ownerHwnd, out var ownerRect)) return;

                var ownerWidth = ownerRect.Right - ownerRect.Left;
                var ownerHeight = ownerRect.Bottom - ownerRect.Top;
                var x = ownerRect.Left + (ownerWidth - appWindow.Size.Width) / 2;
                var y = ownerRect.Top + (ownerHeight - appWindow.Size.Height) / 2;
                appWindow.Move(new Windows.Graphics.PointInt32 { X = x, Y = y });
            }
            catch
            {
                // Best-effort; on failure the OS picks a default position.
            }
        }

        // ---- Win32 interop -------------------------------------------------

        private const int GWLP_HWNDPARENT = -8;

        [DllImport("user32.dll", EntryPoint = "SetWindowLongPtrW", SetLastError = true)]
        private static extern IntPtr SetWindowLongPtr(IntPtr hWnd, int nIndex, IntPtr dwNewLong);

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

        [StructLayout(LayoutKind.Sequential)]
        private struct RECT
        {
            public int Left;
            public int Top;
            public int Right;
            public int Bottom;
        }
    }
}
