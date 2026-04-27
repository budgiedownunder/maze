namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Result of an OAuth browser flow: the query parameters echoed back by the
    /// server in the URL fragment of the custom-scheme redirect (typically
    /// <c>token</c> and <c>expires_at</c>).
    /// </summary>
    public class OAuthCallbackResult
    {
        public IDictionary<string, string> Properties { get; init; } = new Dictionary<string, string>();
    }

    /// <summary>
    /// Thin abstraction over the platform browser-flow API so tests can substitute
    /// a fake. Different platforms use different underlying APIs:
    /// <list type="bullet">
    ///   <item><description>Android, iOS, macOS — MAUI's <c>Microsoft.Maui.Authentication.WebAuthenticator</c></description></item>
    ///   <item><description>Windows — an in-app <c>WebView2</c> popup that
    ///     intercepts the <c>maze-app://oauth-callback</c> redirect via
    ///     <c>NavigationStarting</c>. MAUI's built-in
    ///     <c>WebAuthenticator</c> throws <see cref="PlatformNotSupportedException"/>
    ///     on Windows (see <see href="https://github.com/microsoft/WindowsAppSDK/issues/441"/>),
    ///     and the in-app popup keeps the flow visually owned by the app
    ///     without involving the system browser or OS protocol activation.</description></item>
    /// </list>
    /// The platform-appropriate implementation is registered in DI by
    /// <c>MauiProgram.CreateMauiApp</c> using <c>#if</c> compile-time selection.
    /// </summary>
    public interface IWebAuthenticatorBroker
    {
        /// <summary>
        /// Opens <paramref name="startUrl"/> in the platform browser flow and waits
        /// for it to be redirected to <paramref name="callbackUrl"/> via the
        /// registered custom URL scheme. On Windows the "browser" is an in-app
        /// WebView2 popup; on other platforms it is the OS-managed browser
        /// session selected by MAUI's <c>WebAuthenticator</c>. Returns the
        /// parsed callback parameters from the URL fragment and/or query
        /// string of the final redirect.
        /// </summary>
        Task<OAuthCallbackResult> AuthenticateAsync(Uri startUrl, Uri callbackUrl);
    }

#if !WINDOWS
    /// <summary>
    /// Default implementation that delegates to MAUI's <c>WebAuthenticator.Default</c>.
    /// Used on every platform except Windows; the Windows broker lives in
    /// <c>Platforms/Windows/</c> and hosts an in-app <c>WebView2</c> popup.
    /// </summary>
    internal class DefaultWebAuthenticatorBroker : IWebAuthenticatorBroker
    {
        public async Task<OAuthCallbackResult> AuthenticateAsync(Uri startUrl, Uri callbackUrl)
        {
            var result = await Microsoft.Maui.Authentication.WebAuthenticator.Default.AuthenticateAsync(
                new Microsoft.Maui.Authentication.WebAuthenticatorOptions
                {
                    Url = startUrl,
                    CallbackUrl = callbackUrl,
                    // Prefer ephemeral browser session on iOS so cached IdP cookies do not
                    // pre-select an unexpected account.
                    PrefersEphemeralWebBrowserSession = true,
                });
            return new OAuthCallbackResult
            {
                Properties = new Dictionary<string, string>(result.Properties),
            };
        }
    }
#endif
}
