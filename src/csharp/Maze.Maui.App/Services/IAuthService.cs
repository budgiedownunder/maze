using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// One row from the server's <c>user_emails</c> table — an address plus
    /// its primary/verified flags. Returned in the <see cref="UserProfile.Emails"/>
    /// list and as elements of the <see cref="EmailsResponse"/>.
    /// </summary>
    public class UserEmail
    {
        [JsonPropertyName("email")]
        public string Email { get; set; } = "";

        [JsonPropertyName("is_primary")]
        public bool IsPrimary { get; set; }

        [JsonPropertyName("verified")]
        public bool Verified { get; set; }

        [JsonPropertyName("verified_at")]
        public DateTimeOffset? VerifiedAt { get; set; }
    }

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

        /// <summary>All email rows attached to this user, including primary
        /// status, verification status, and verification timestamp.</summary>
        [JsonPropertyName("emails")]
        public List<UserEmail> Emails { get; set; } = new();

        /// <summary>Whether the user has a password set. <c>false</c> for
        /// OAuth-only users who haven't yet added a password as a second
        /// login method — front-ends use this to choose between the "Change"
        /// and "Set" variants of the password popup. The hash itself is
        /// never exposed.</summary>
        [JsonPropertyName("has_password")]
        public bool HasPassword { get; set; }
    }

    /// <summary>
    /// Result of a successful OAuth sign-in. <see cref="IsFirstSignIn"/>
    /// is the welcome-banner trigger across both auth flows: it is true
    /// when the user has had no prior successful sign-in via any method
    /// (server returned <c>first_sign_in=true</c> in the callback URL
    /// fragment). <see cref="IsNewUser"/> is a separate audit signal —
    /// true when the server's <c>account::resolve</c> created a brand-new
    /// User row during this OAuth flow. They agree in most cases but
    /// disagree e.g. when a user signed up via credentials, never
    /// verified, then signs in via OAuth: <c>IsNewUser=false</c> (row
    /// already existed) but <c>IsFirstSignIn=true</c> (no prior session).
    /// The ViewModel layer keys the welcome banner off
    /// <c>IsFirstSignIn</c>.
    /// </summary>
    public class OAuthSignInResult
    {
        public UserProfile Profile { get; init; } = new();
        public bool IsNewUser { get; init; }
        public bool IsFirstSignIn { get; init; }
    }

    /// <summary>
    /// Result of a successful credential sign-in. <see cref="IsFirstSignIn"/>
    /// is the welcome-banner trigger — true when the user has had no
    /// prior successful sign-in via any method (server returned
    /// <c>is_first_sign_in: true</c> in the login response). The
    /// ViewModel layer keys the welcome banner off this flag, the same
    /// way it does for <see cref="OAuthSignInResult"/>.
    /// </summary>
    public class CredentialsSignInResult
    {
        public UserProfile Profile { get; init; } = new();
        public bool IsFirstSignIn { get; init; }
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

        /// <summary>Signs in with email and password. Stores the returned bearer token.
        /// Returns the user profile alongside <c>IsFirstSignIn</c>, the welcome-banner
        /// trigger (true when this is the user's first ever successful sign-in via
        /// any method).</summary>
        Task<CredentialsSignInResult> SignInAsync(string email, string password);

        /// <summary>
        /// Signs in via the named OAuth provider (e.g. "google", "github") using the platform's
        /// browser flow. The server runs the full OAuth flow and redirects the browser to
        /// <c>maze-app://oauth-callback#token=...&amp;expires_at=...&amp;new_user=true</c> (params
        /// in the fragment; the <c>new_user</c> flag is present only on first-time sign-ups);
        /// the platform broker captures it. The token is stored, the user profile is fetched,
        /// and the <c>IsNewUser</c> flag is forwarded to the caller.
        /// </summary>
        /// <param name="providerName">Canonical provider name as exposed by <see cref="IAppFeaturesService"/>.</param>
        Task<OAuthSignInResult> SignInWithOAuthAsync(string providerName);

        /// <summary>Signs out, removing the stored bearer token.</summary>
        Task SignOutAsync();

        /// <summary>Registers a new account. Does not auto sign-in.</summary>
        Task<UserProfile> SignUpAsync(string email, string password);

        /// <summary>Returns the profile for the currently authenticated user.</summary>
        Task<UserProfile> GetMyProfileAsync();

        /// <summary>Deletes the currently authenticated user's account and clears the stored token.</summary>
        Task DeleteMyAccountAsync();

        /// <summary>Changes the current user's password. Requires the
        /// current password (the user has one — <see cref="UserProfile.HasPassword"/>
        /// is <c>true</c>). Throws <see cref="HttpRequestException"/> on
        /// failure (401 = current password incorrect).</summary>
        Task ChangePasswordAsync(string currentPassword, string newPassword);

        /// <summary>Sets an initial password for an OAuth-only user (one
        /// whose <see cref="UserProfile.HasPassword"/> is <c>false</c>). The
        /// request body omits <c>current_password</c> entirely; sending it
        /// in the set-initial flow is rejected by the server with a 400.
        /// </summary>
        Task SetInitialPasswordAsync(string newPassword);

        /// <summary>Updates the current user's profile (username and full name).
        /// Email is mutated through the dedicated email-management methods
        /// (<see cref="AddEmailAsync"/> / <see cref="RemoveEmailAsync"/> /
        /// <see cref="SetPrimaryEmailAsync"/>); the server rejects this
        /// endpoint if `email` is included in the body. Returns the
        /// updated profile.</summary>
        Task<UserProfile> UpdateProfileAsync(string username, string fullName);

        /// <summary>Returns all email rows attached to the current user.</summary>
        Task<List<UserEmail>> GetMyEmailsAsync();

        /// <summary>Adds a new email row to the current user, returning the
        /// updated email list. Throws <see cref="HttpRequestException"/> on
        /// 400 (invalid format) or 409 (already taken).</summary>
        Task<List<UserEmail>> AddEmailAsync(string email);

        /// <summary>Removes an email row from the current user, returning
        /// the updated email list. Throws <see cref="HttpRequestException"/>
        /// on 404 (not registered) or 409 (last email or primary).</summary>
        Task<List<UserEmail>> RemoveEmailAsync(string email);

        /// <summary>Promotes an email to the user's primary, returning the
        /// updated email list. Throws <see cref="HttpRequestException"/> on
        /// 404 (not registered) or 409 (target is unverified).</summary>
        Task<List<UserEmail>> SetPrimaryEmailAsync(string email);

        /// <summary>Requests a password-reset email for the given address.
        /// Always succeeds at the HTTP layer (anti-enumeration: the server
        /// returns 200 whether or not the address is registered). Throws
        /// <see cref="HttpRequestException"/> only on transport failures.
        /// No bearer required.</summary>
        Task RequestPasswordResetAsync(string email);

        /// <summary>Re-sends the verification email for an address attached
        /// to the currently authenticated user. Throws
        /// <see cref="HttpRequestException"/> with status 404 if the address
        /// is not on the caller's account, or transport-level failures. The
        /// server is idempotent: re-requesting verification for an already
        /// verified address is a 200 no-op.</summary>
        Task RequestEmailVerificationAsync(string email);

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
