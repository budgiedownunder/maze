using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a user profile returned by the server
    /// </summary>
    public class UserProfile
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = "";

        [JsonPropertyName("is_admin")]
        public bool IsAdmin { get; set; }

        [JsonPropertyName("username")]
        public string Username { get; set; } = "";

        [JsonPropertyName("full_name")]
        public string FullName { get; set; } = "";

        [JsonPropertyName("email")]
        public string Email { get; set; } = "";
    }

    /// <summary>
    /// Represents a service for managing user authentication
    /// </summary>
    public interface IAuthService
    {
        /// <summary>Returns true if a bearer token is currently stored.</summary>
        Task<bool> IsAuthenticatedAsync();

        /// <summary>Returns the stored bearer token, or null if not authenticated.</summary>
        Task<string?> GetBearerTokenAsync();

        /// <summary>Signs in with email and password. Stores the returned bearer token. Returns the user profile.</summary>
        Task<UserProfile> SignInAsync(string email, string password);

        /// <summary>
        /// Signs in via the named OAuth provider (e.g. "google", "github") using the platform's
        /// browser. The server runs the full OAuth flow and redirects the platform browser to
        /// <c>maze-app://oauth-callback?token=...&amp;expires_at=...</c>; the WebAuthenticator
        /// captures it. The token is then stored and the user profile fetched.
        /// </summary>
        /// <param name="providerName">Canonical provider name as exposed by <see cref="IAppFeaturesService"/>.</param>
        Task<UserProfile> SignInWithOAuthAsync(string providerName);

        /// <summary>Signs out, removing the stored bearer token.</summary>
        Task SignOutAsync();

        /// <summary>Registers a new account. Does not auto sign-in.</summary>
        Task<UserProfile> SignUpAsync(string email, string password);

        /// <summary>Returns the profile for the currently authenticated user.</summary>
        Task<UserProfile> GetMyProfileAsync();

        /// <summary>Deletes the currently authenticated user's account and clears the stored token.</summary>
        Task DeleteMyAccountAsync();

        /// <summary>Changes the current user's password. Throws HttpRequestException on failure.</summary>
        Task ChangePasswordAsync(string currentPassword, string newPassword);

        /// <summary>Updates the current user's profile (username, full name, email). Returns the updated profile.</summary>
        Task<UserProfile> UpdateProfileAsync(string username, string fullName, string email);

        /// <summary>
        /// Attempts to renew the current bearer login token, extending its lifetime without
        /// re-authenticating. Updates the stored token expiry on success.
        /// </summary>
        /// <returns>True if the server confirmed renewal; false if the token is missing, expired, or the request failed</returns>
        Task<bool> RenewAsync();

        /// <summary>Returns the expiry timestamp of the stored bearer token, or null if not available.</summary>
        /// <returns>Token expiry, or null</returns>
        Task<DateTimeOffset?> GetTokenExpiryAsync();
    }
}
