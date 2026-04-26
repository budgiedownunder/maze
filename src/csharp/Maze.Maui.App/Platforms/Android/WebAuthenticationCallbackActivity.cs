using Android.App;
using Android.Content;
using Android.Content.PM;

namespace Maze.Maui.App
{
    /// <summary>
    /// Receives the OAuth provider's redirect on Android. The intent-filter declares
    /// that this activity handles <c>maze-app://oauth-callback</c>; the MAUI
    /// <c>WebAuthenticator</c> base class extracts the query parameters and resolves
    /// the in-flight <c>AuthenticateAsync</c> task.
    /// </summary>
    [Activity(NoHistory = true, LaunchMode = LaunchMode.SingleTop, Exported = true)]
    [IntentFilter(
        new[] { Intent.ActionView },
        Categories = new[] { Intent.CategoryDefault, Intent.CategoryBrowsable },
        DataScheme = "maze-app",
        DataHost = "oauth-callback")]
    public class WebAuthenticationCallbackActivity : Microsoft.Maui.Authentication.WebAuthenticatorCallbackActivity
    {
    }
}
