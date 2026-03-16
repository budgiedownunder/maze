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
        [JsonPropertyName("username")]
        public string Username { get; set; } = "";

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
    }

    /// <summary>
    /// Signup request body
    /// </summary>
    internal class SignupRequest
    {
        [JsonPropertyName("username")]
        public string Username { get; set; } = "";

        [JsonPropertyName("full_name")]
        public string FullName { get; set; } = "";

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
        private const string HEADER_AUTHORIZATION = "Authorization";
        private const double REQUEST_TIMEOUT_SECONDS = 30.0;

        private readonly ConfigurationService _configurationService;
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        public AuthHttpClientService(ConfigurationService configurationService)
        {
            _configurationService = configurationService;
            _httpClient = CreateHttpClient();
        }

        private HttpClient CreateHttpClient()
        {
            HttpClientHandler handler = new HttpClientHandler();

            if (_configurationService.DisableStrictTLSCertificateValidation)
                handler.ServerCertificateCustomValidationCallback = (message, cert, chain, sslPolicyErrors) => true;

            return new HttpClient(handler)
            {
                BaseAddress = new Uri(_configurationService.ApiRootUri),
                Timeout = TimeSpan.FromSeconds(REQUEST_TIMEOUT_SECONDS)
            };
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
        public async Task<UserProfile> SignInAsync(string username, string password)
        {
            var body = JsonSerializer.Serialize(new LoginRequest { Username = username, Password = password });
            var content = new StringContent(body, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("login", content);
            await EnsureSuccessAsync(response, "Sign in failed");

            string json = await response.Content.ReadAsStringAsync();
            var loginResponse = JsonSerializer.Deserialize<LoginResponse>(json)
                ?? throw new Exception("Invalid login response from server");

            if (string.IsNullOrEmpty(loginResponse.LoginTokenId))
                throw new Exception("Server returned an empty login token");

            await SecureStorage.Default.SetAsync(TOKEN_STORAGE_KEY, loginResponse.LoginTokenId);

            return await GetMyProfileAsync();
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
            }
        }

        /// <inheritdoc/>
        public async Task<UserProfile> SignUpAsync(string username, string fullName, string email, string password)
        {
            var body = JsonSerializer.Serialize(new SignupRequest
            {
                Username = username,
                FullName = fullName,
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
            }
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
