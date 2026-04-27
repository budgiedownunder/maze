using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Public-facing description of a single OAuth provider configured on the server.
    /// </summary>
    public class OAuthProviderPublic
    {
        /// <summary>Canonical provider name, e.g. "google" or "github".</summary>
        [JsonPropertyName("name")]
        public string Name { get; set; } = "";

        /// <summary>User-facing label rendered on the provider button.</summary>
        [JsonPropertyName("display_name")]
        public string DisplayName { get; set; } = "";
    }

    /// <summary>
    /// Represents the server-controlled feature flags for the application.
    /// </summary>
    public class AppFeatures
    {
        /// <summary>Whether new users can self-register.</summary>
        [JsonPropertyName("allow_signup")]
        public bool AllowSignUp { get; set; } = true;

        /// <summary>OAuth providers currently enabled on the server. Empty when OAuth is disabled.</summary>
        [JsonPropertyName("oauth_providers")]
        public List<OAuthProviderPublic> OAuthProviders { get; set; } = new();
    }

    /// <summary>
    /// Represents a service for retrieving server-controlled feature flags.
    /// </summary>
    public interface IAppFeaturesService
    {
        /// <summary>The most recently fetched features. Defaults to fail-open values.</summary>
        AppFeatures Features { get; }

        /// <summary>Fetches features from the server. On failure, retains the current (default) value.</summary>
        Task RefreshAsync();
    }
}
