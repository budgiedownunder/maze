using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents the server-controlled feature flags for the application.
    /// </summary>
    public class AppFeatures
    {
        /// <summary>Whether new users can self-register.</summary>
        [JsonPropertyName("allow_signup")]
        public bool AllowSignUp { get; set; } = true;
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
