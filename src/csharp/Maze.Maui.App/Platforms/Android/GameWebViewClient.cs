using Android.Net.Http;
using Android.Webkit;
using Maze.Maui.App.Models;
using Microsoft.Maui.Handlers;
using Microsoft.Maui.Platform;

namespace Maze.Maui.App.Platforms.Android
{
    /// <summary>
    /// The WebView client used by the hosted 3D game. Always installed, because
    /// it carries the renderer-death handling; the TLS bypass it also provides is
    /// conditional on <c>DisableStrictTLSCertificateValidation</c> in
    /// appsettings.json (needed for development servers using self-signed
    /// certificates).
    /// </summary>
    /// <param name="handler">The MAUI WebView handler this client belongs to</param>
    /// <param name="ignoreSslErrors">Whether to accept untrusted TLS certificates</param>
    class GameWebViewClient(WebViewHandler handler, bool ignoreSslErrors) : MauiWebViewClient(handler)
    {
        public override void OnReceivedSslError(
            global::Android.Webkit.WebView? view, SslErrorHandler? handler, SslError? error)
        {
            if (ignoreSslErrors)
            {
                handler?.Proceed();
                return;
            }
            base.OnReceivedSslError(view, handler, error);
        }

        /// <summary>
        /// The renderer process backing this WebView died. **Returning
        /// <c>true</c> is the point of this override**: returning <c>false</c>
        /// (the default when no client handles it) lets the Android framework
        /// kill the entire app process, which the user experiences as the app
        /// vanishing and relaunching. Returning <c>true</c> keeps the app alive
        /// with a dead WebView, which the game page then tears down cleanly.
        ///
        /// <c>RenderProcessGoneDetail.DidCrash()</c> separates a genuine crash
        /// from the system reclaiming the renderer — the latter being how an
        /// out-of-memory kill presents.
        /// </summary>
        /// <param name="view">The web view whose renderer died</param>
        /// <param name="detail">Why the renderer went away</param>
        /// <returns>Always <c>true</c> — handled, so the app survives</returns>
        public override bool OnRenderProcessGone(
            global::Android.Webkit.WebView? view, RenderProcessGoneDetail? detail)
        {
            // The app supports Android 21, but both this callback and
            // DidCrash() are API 26+. Android simply never invokes the callback
            // below 26 (there the framework kills the app process and there is
            // nothing to report), so the guard is really for the analyzer — and
            // it keeps the call honest if that ever changes.
            bool crashed = OperatingSystem.IsAndroidVersionAtLeast(26) && (detail?.DidCrash() ?? false);
            GameWebViewHandler.RaiseHostFailure(new GameFailure
            {
                Reason = crashed
                    ? GameFailure.GenericReason
                    : "The game ran out of memory and had to close.",
                Detail = $"render-process-gone didCrash={crashed}",
                Phase = null,
            });
            return true;
        }
    }
}
