using System.Globalization;

namespace Maze.Maui.App.Converters
{
    /// <summary>
    /// Maps an OAuth provider's canonical name (e.g. "google", "github") to the
    /// asset filename of its brand-compliant icon under
    /// <c>Resources/Images/</c>. Returns <c>null</c> for unknown providers,
    /// which causes MAUI's <c>Button.ImageSource</c> binding to render a plain
    /// text-only button — graceful fallback when a server adds a new provider
    /// before the matching icon ships in the app.
    ///
    /// Adding a new provider is a two-step change here: drop a new SVG into
    /// <c>Resources/Images/</c> and add one switch arm.
    /// </summary>
    public class OAuthProviderIconConverter : IValueConverter
    {
        public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        {
            if (value is not string name) return null;
            // MAUI's MauiImage build pipeline rasterises every SVG under
            // Resources/Images/ into a PNG (per density bucket) at build time.
            // Runtime references must use the .png filename even though the
            // source asset is .svg — same convention as icon_lock.png /
            // icon_account.png elsewhere in the project.
            //
            // The GitHub Octocat is single-colour and would be invisible on a
            // dark-mode background using its light-mode dark fill, so we pick
            // a light-fill variant (`oauth_github_dark.png`) when the OS
            // theme is dark. Google's "G" stays single-asset because Google's
            // brand guidelines forbid recolouring the mark.
            //
            // The theme is captured here at bind time. To keep the OAuth
            // buttons in sync when the OS theme changes at runtime,
            // LoginPage and SignUpPage subscribe to
            // Application.RequestedThemeChanged in OnAppearing and call
            // LoginViewModel.RefreshOAuthProviderItems() /
            // SignUpViewModel.RefreshOAuthProviderItems() — clearing and
            // re-adding the provider list forces BindableLayout to
            // re-instantiate each item's Button, which re-runs this
            // converter under the new theme.
            var isDark = Application.Current?.RequestedTheme == AppTheme.Dark;
            return name.ToLowerInvariant() switch
            {
                "google" => "oauth_google.png",
                "github" => isDark ? "oauth_github_dark.png" : "oauth_github.png",
                _ => null,
            };
        }

        public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
            => throw new NotImplementedException();
    }
}
