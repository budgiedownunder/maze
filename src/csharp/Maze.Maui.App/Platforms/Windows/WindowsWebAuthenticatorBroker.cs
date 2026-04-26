namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Windows-specific <see cref="IWebAuthenticatorBroker"/> backed by
    /// <c>WinUIEx.WebAuthenticator</c>. MAUI's built-in
    /// <c>Microsoft.Maui.Authentication.WebAuthenticator</c> throws
    /// <see cref="PlatformNotSupportedException"/> on Windows
    /// (see <see href="https://github.com/microsoft/WindowsAppSDK/issues/441"/>),
    /// so we delegate to WinUIEx, which works for both packaged and unpackaged apps.
    /// WinUIEx registers a temporary protocol handler for the callback URL's
    /// scheme in <c>HKCU\Software\Classes</c>, opens the start URL in the system
    /// browser, and resolves the in-flight task when the OS reactivates the app
    /// via that scheme.
    /// </summary>
    internal class WindowsWebAuthenticatorBroker : IWebAuthenticatorBroker
    {
        public async Task<OAuthCallbackResult> AuthenticateAsync(Uri startUrl, Uri callbackUrl)
        {
            var result = await WinUIEx.WebAuthenticator.AuthenticateAsync(startUrl, callbackUrl);
            return new OAuthCallbackResult
            {
                Properties = new Dictionary<string, string>(result.Properties),
            };
        }
    }
}
