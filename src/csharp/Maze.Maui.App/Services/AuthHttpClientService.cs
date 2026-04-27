using System.Net.Http.Json;
using System.Net;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Login request body
    /// </summary>
    internal class LoginRequest
    {
        [JsonPropertyName("email")]
        public string Email { get; set; } = "";

        [JsonPropertyName("password")]
        public string Password { get; set; } = "";
    }

    /// <summary>
    /// Login response body
    /// </summary>
    internal class LoginResponse
    {
        [JsonPropertyName("login_token_id")]
        public string LoginTokenId { get; set; } = "";

        [JsonPropertyName("login_token_expires_at")]
        public DateTimeOffset LoginTokenExpiresAt { get; set; }
    }

    /// <summary>
    /// Renew login token response body
    /// </summary>
    internal class RenewResponse
    {
        [JsonPropertyName("login_token_id")]
        public string LoginTokenId { get; set; } = "";

        [JsonPropertyName("login_token_expires_at")]
        public DateTimeOffset LoginTokenExpiresAt { get; set; }
    }

    /// <summary>
    /// Change password request body
    /// </summary>
    internal class ChangePasswordRequest
    {
        [JsonPropertyName("current_password")]
        public string CurrentPassword { get; set; } = "";

        [JsonPropertyName("new_password")]
        public string NewPassword { get; set; } = "";
    }

    /// <summary>
    /// Update profile request body
    /// </summary>
    internal class UpdateProfileRequest
    {
        [JsonPropertyName("username")]
        public string Username { get; set; } = "";

        [JsonPropertyName("full_name")]
        public string FullName { get; set; } = "";

        [JsonPropertyName("email")]
        public string Email { get; set; } = "";
    }

    /// <summary>
    /// Signup request body
    /// </summary>
    internal class SignupRequest
    {
        [JsonPropertyName("email")]
        public string Email { get; set; } = "";

        [JsonPropertyName("password")]
        public string Password { get; set; } = "";
    }

    /// <summary>
    /// Represents an HTTP client service for authentication operations
    /// </summary>
    public class AuthHttpClientService : IAuthService
    {
        private const string TOKEN_STORAGE_KEY = "bearer_token";
        private const string TOKEN_EXPIRY_STORAGE_KEY = "bearer_token_expires_at";
        private const string HEADER_AUTHORIZATION = "Authorization";

        // Custom URL scheme registered on each platform; the server redirects
        // the platform browser to "{scheme}://oauth-callback?token=...&expires_at=..."
        // and WebAuthenticator's URL-scheme handler picks it up.
        internal const string OAUTH_CALLBACK_URL = "maze-app://oauth-callback";

        // Upper bound on how long we wait for the user to complete an OAuth flow
        // in the platform browser. Without this, cancelling the Windows
        // "Open Maze" protocol-activation prompt leaves AuthenticateAsync
        // blocked forever — the broker has no way to detect that the user
        // refused the activation. Five minutes is generous enough for any
        // realistic consent screen but short enough that a stuck flow
        // self-heals.
        private static readonly TimeSpan OAUTH_FLOW_TIMEOUT = TimeSpan.FromMinutes(5);

        private readonly HttpClient _httpClient;
        private readonly ConfigurationService _configurationService;
        private readonly IWebAuthenticatorBroker _webAuthenticator;

        /// <summary>
        /// Constructor. The broker is platform-specific: on Windows MAUI's
        /// built-in WebAuthenticator throws <see cref="PlatformNotSupportedException"/>,
        /// so DI registers a WinUIEx-backed broker on that platform and the
        /// MAUI broker on every other.
        /// </summary>
        public AuthHttpClientService(ConfigurationService configurationService, IWebAuthenticatorBroker webAuthenticator)
        {
            _httpClient = ApiHttpClientFactory.Create(configurationService);
            _configurationService = configurationService;
            _webAuthenticator = webAuthenticator;
        }

        /// <inheritdoc/>
        public async Task<bool> IsAuthenticatedAsync()
        {
            string? token = await SecureStorage.Default.GetAsync(TOKEN_STORAGE_KEY);
            return !string.IsNullOrEmpty(token);
        }

        /// <inheritdoc/>
        public async Task<string?> GetBearerTokenAsync()
            => await SecureStorage.Default.GetAsync(TOKEN_STORAGE_KEY);

        /// <inheritdoc/>
        public async Task<UserProfile> SignInAsync(string email, string password)
        {
            var body = JsonSerializer.Serialize(new LoginRequest { Email = email, Password = password });
            var content = new StringContent(body, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("login", content);
            await EnsureSuccessAsync(response, "Sign in failed");

            string json = await response.Content.ReadAsStringAsync();
            var loginResponse = JsonSerializer.Deserialize<LoginResponse>(json)
                ?? throw new Exception("Invalid login response from server");

            if (string.IsNullOrEmpty(loginResponse.LoginTokenId))
                throw new Exception("Server returned an empty login token");

            await SecureStorage.Default.SetAsync(TOKEN_STORAGE_KEY, loginResponse.LoginTokenId);
            await SecureStorage.Default.SetAsync(TOKEN_EXPIRY_STORAGE_KEY, loginResponse.LoginTokenExpiresAt.ToString("O"));

            return await GetMyProfileAsync();
        }

        /// <inheritdoc/>
        public async Task<OAuthSignInResult> SignInWithOAuthAsync(string providerName)
        {
            if (string.IsNullOrWhiteSpace(providerName))
                throw new ArgumentException("Provider name must be supplied", nameof(providerName));

            var startUrl = new Uri(new Uri(_configurationService.ApiRootUri),
                $"auth/oauth/{Uri.EscapeDataString(providerName)}/start?origin=mobile");
            var callbackUrl = new Uri(OAUTH_CALLBACK_URL);

            // .WaitAsync turns "stuck waiting forever" into a TimeoutException
            // the ViewModel can translate into a friendly cancellation message.
            var result = await _webAuthenticator
                .AuthenticateAsync(startUrl, callbackUrl)
                .WaitAsync(OAUTH_FLOW_TIMEOUT);

            // Server-side recoverable errors (signup disabled, email not
            // verified, etc.) come back via `reason=<code>` on the same
            // callback URL — surface them as a typed exception so the
            // ViewModel can show a friendly per-code message instead of a
            // generic "Sign in failed" toast.
            if (result.Properties.TryGetValue("reason", out var reason) && !string.IsNullOrEmpty(reason))
                throw new OAuthFlowFailedException(reason);

            if (!result.Properties.TryGetValue("token", out var token) || string.IsNullOrEmpty(token))
                throw new Exception($"OAuth response did not include a bearer token (broker returned {result.Properties.Count} properties: [{string.Join(", ", result.Properties.Keys)}])");
            if (!result.Properties.TryGetValue("expires_at", out var expiresAt) || string.IsNullOrEmpty(expiresAt))
                throw new Exception("OAuth response did not include a token expiry");

            // The server emits `new_user=true` on the redirect URL only when
            // account::resolve created a brand-new user; the ViewModel layer
            // uses this to open the Account UI with a welcome banner.
            bool isNewUser = result.Properties.TryGetValue("new_user", out var newUserRaw)
                             && string.Equals(newUserRaw, "true", StringComparison.Ordinal);

            await SecureStorage.Default.SetAsync(TOKEN_STORAGE_KEY, token);
            await SecureStorage.Default.SetAsync(TOKEN_EXPIRY_STORAGE_KEY, expiresAt);

            var profile = await GetMyProfileAsync();
            return new OAuthSignInResult { Profile = profile, IsNewUser = isNewUser };
        }

        /// <inheritdoc/>
        public async Task SignOutAsync()
        {
            try
            {
                string? token = await SecureStorage.Default.GetAsync(TOKEN_STORAGE_KEY);
                if (!string.IsNullOrEmpty(token))
                {
                    using var request = new HttpRequestMessage(HttpMethod.Post, "logout");
                    request.Headers.Add(HEADER_AUTHORIZATION, $"Bearer {token}");
                    await _httpClient.SendAsync(request);
                }
            }
            finally
            {
                SecureStorage.Default.Remove(TOKEN_STORAGE_KEY);
                SecureStorage.Default.Remove(TOKEN_EXPIRY_STORAGE_KEY);
            }
        }

        /// <inheritdoc/>
        public async Task<UserProfile> SignUpAsync(string email, string password)
        {
            var body = JsonSerializer.Serialize(new SignupRequest
            {
                Email = email,
                Password = password
            });
            var content = new StringContent(body, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("signup", content);
            await EnsureSuccessAsync(response, "Sign up failed");

            string json = await response.Content.ReadAsStringAsync();
            return JsonSerializer.Deserialize<UserProfile>(json)
                ?? throw new Exception("Invalid signup response from server");
        }

        /// <inheritdoc/>
        public async Task<UserProfile> GetMyProfileAsync()
        {
            using var request = new HttpRequestMessage(HttpMethod.Get, "users/me");
            await AddBearerHeaderAsync(request);
            var response = await _httpClient.SendAsync(request);
            await EnsureSuccessAsync(response, "Failed to load profile");

            string json = await response.Content.ReadAsStringAsync();
            return JsonSerializer.Deserialize<UserProfile>(json)
                ?? throw new Exception("Invalid profile response from server");
        }

        /// <inheritdoc/>
        public async Task DeleteMyAccountAsync()
        {
            try
            {
                using var request = new HttpRequestMessage(HttpMethod.Delete, "users/me");
                await AddBearerHeaderAsync(request);
                var response = await _httpClient.SendAsync(request);
                await EnsureSuccessAsync(response, "Failed to delete account");
            }
            finally
            {
                SecureStorage.Default.Remove(TOKEN_STORAGE_KEY);
                SecureStorage.Default.Remove(TOKEN_EXPIRY_STORAGE_KEY);
            }
        }

        /// <inheritdoc/>
        public async Task ChangePasswordAsync(string currentPassword, string newPassword)
        {
            var body = JsonSerializer.Serialize(new ChangePasswordRequest
            {
                CurrentPassword = currentPassword,
                NewPassword = newPassword
            });
            using var request = new HttpRequestMessage(HttpMethod.Put, "users/me/password");
            await AddBearerHeaderAsync(request);
            request.Content = new StringContent(body, Encoding.UTF8, "application/json");
            var response = await _httpClient.SendAsync(request);
            await EnsureSuccessAsync(response, "Change password failed");
        }

        /// <inheritdoc/>
        public async Task<UserProfile> UpdateProfileAsync(string username, string fullName, string email)
        {
            var body = JsonSerializer.Serialize(new UpdateProfileRequest
            {
                Username = username,
                FullName = fullName,
                Email = email
            });
            using var request = new HttpRequestMessage(HttpMethod.Put, "users/me/profile");
            await AddBearerHeaderAsync(request);
            request.Content = new StringContent(body, Encoding.UTF8, "application/json");
            var response = await _httpClient.SendAsync(request);
            await EnsureSuccessAsync(response, "Update profile failed");

            string json = await response.Content.ReadAsStringAsync();
            return JsonSerializer.Deserialize<UserProfile>(json)
                ?? throw new Exception("Invalid profile response from server");
        }

        /// <inheritdoc/>
        public async Task<bool> RenewAsync()
        {
            try
            {
                string? token = await SecureStorage.Default.GetAsync(TOKEN_STORAGE_KEY);
                if (string.IsNullOrEmpty(token))
                    return false;

                using var request = new HttpRequestMessage(HttpMethod.Post, "login/renew");
                request.Headers.Add(HEADER_AUTHORIZATION, $"Bearer {token}");
                var response = await _httpClient.SendAsync(request);
                if (!response.IsSuccessStatusCode)
                    return false;

                string json = await response.Content.ReadAsStringAsync();
                var renewResponse = JsonSerializer.Deserialize<RenewResponse>(json);
                if (renewResponse is null)
                    return false;

                await SecureStorage.Default.SetAsync(TOKEN_EXPIRY_STORAGE_KEY, renewResponse.LoginTokenExpiresAt.ToString("O"));
                return true;
            }
            catch
            {
                return false;
            }
        }

        /// <inheritdoc/>
        public async Task<DateTimeOffset?> GetTokenExpiryAsync()
        {
            string? raw = await SecureStorage.Default.GetAsync(TOKEN_EXPIRY_STORAGE_KEY);
            if (DateTimeOffset.TryParse(raw, out var expiry))
                return expiry;
            return null;
        }

        private async Task AddBearerHeaderAsync(HttpRequestMessage request)
        {
            string? token = await SecureStorage.Default.GetAsync(TOKEN_STORAGE_KEY);
            if (!string.IsNullOrEmpty(token))
                request.Headers.Add(HEADER_AUTHORIZATION, $"Bearer {token}");
        }

        private static async Task EnsureSuccessAsync(HttpResponseMessage response, string contextMessage)
        {
            if (response.IsSuccessStatusCode)
                return;

            string body = "";
            try { body = await response.Content.ReadAsStringAsync(); }
            catch { }

            string detail = string.IsNullOrWhiteSpace(body)
                ? response.ReasonPhrase ?? response.StatusCode.ToString()
                : body;

            throw new HttpRequestException($"{contextMessage}: {detail}", null, response.StatusCode);
        }
    }
}
