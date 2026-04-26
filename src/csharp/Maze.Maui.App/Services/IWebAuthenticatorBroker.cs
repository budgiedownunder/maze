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
    ///   <item><description>Windows — <c>WinUIEx.WebAuthenticator</c> (MAUI's built-in throws
    ///     <see cref="PlatformNotSupportedException"/> on Windows;
    ///     see <see href="https://github.com/microsoft/WindowsAppSDK/issues/441"/>)</description></item>
    /// </list>
    /// The platform-appropriate implementation is registered in DI by
    /// <c>MauiProgram.CreateMauiApp</c> using <c>#if</c> compile-time selection.
    /// </summary>
    public interface IWebAuthenticatorBroker
    {
        /// <summary>
        /// Opens <paramref name="startUrl"/> in the platform browser and waits for the
        /// browser to be redirected to <paramref name="callbackUrl"/> via the registered
        /// custom URL scheme. Returns the parsed query parameters.
        /// </summary>
        Task<OAuthCallbackResult> AuthenticateAsync(Uri startUrl, Uri callbackUrl);
    }

#if !WINDOWS
    /// <summary>
    /// Default implementation that delegates to MAUI's <c>WebAuthenticator.Default</c>.
    /// Used on every platform except Windows; the Windows broker lives in
    /// <c>Platforms/Windows/</c> and uses <c>WinUIEx.WebAuthenticator</c>.
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
