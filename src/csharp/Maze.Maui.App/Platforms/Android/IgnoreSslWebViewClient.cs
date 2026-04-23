using Android.Net.Http;
using Android.Webkit;
using Microsoft.Maui.Handlers;
using Microsoft.Maui.Platform;

namespace Maze.Maui.App.Platforms.Android
{
    // Bypasses TLS certificate validation in the Android WebView.
    // Required for development servers using self-signed certificates.
    // Controlled by DisableStrictTLSCertificateValidation in appsettings.json.
    class IgnoreSslWebViewClient : MauiWebViewClient
    {
        public IgnoreSslWebViewClient(WebViewHandler handler) : base(handler) { }

        public override void OnReceivedSslError(global::Android.Webkit.WebView? view, SslErrorHandler? handler, SslError? error)
        {
            handler?.Proceed();
        }
    }
}
