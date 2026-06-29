using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// <see cref="IAvatarService"/> over the same Bearer-token-injecting HTTP
    /// pipeline as <see cref="ScoresHttpClientService"/> /
    /// <see cref="MazeHttpClientService"/>. Fetches the avatar bytes; a 404 (no
    /// avatar) or any failure resolves to <c>null</c> so callers show the
    /// placeholder.
    /// </summary>
    public class AvatarHttpClientService : IAvatarService
    {
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Constructor.
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        /// <param name="authService">Injected auth service (supplies the bearer token)</param>
        public AvatarHttpClientService(ConfigurationService configurationService, IAuthService authService)
        {
            var innerHandler = ApiHttpClientFactory.CreateHandler(configurationService);
            var bearerHandler = new BearerTokenHandler(authService, configurationService, innerHandler);
            _httpClient = ApiHttpClientFactory.Create(configurationService, bearerHandler);
        }

        /// <inheritdoc/>
        public async Task<byte[]?> TryLoadAvatarBytesAsync(string userId, string? avatarUpdatedAt)
        {
            // No marker => the user has no avatar; skip the guaranteed 404.
            if (string.IsNullOrEmpty(userId) || string.IsNullOrEmpty(avatarUpdatedAt))
            {
                return null;
            }

            try
            {
                var path = $"users/{Uri.EscapeDataString(userId)}/avatar?v={Uri.EscapeDataString(avatarUpdatedAt)}";
                using var response = await _httpClient.GetAsync(path);
                if (!response.IsSuccessStatusCode)
                {
                    return null; // 404 (no avatar) or a transient error => placeholder
                }
                return await response.Content.ReadAsByteArrayAsync();
            }
            catch
            {
                return null; // network failure => placeholder
            }
        }

        /// <inheritdoc/>
        public async Task<string?> UploadAvatarAsync(byte[] bytes, string contentType)
        {
            try
            {
                using var content = new MultipartFormDataContent();
                var fileContent = new ByteArrayContent(bytes);
                fileContent.Headers.ContentType = new MediaTypeHeaderValue(
                    string.IsNullOrEmpty(contentType) ? "application/octet-stream" : contentType);
                // Part name "file" matches the server's multipart field.
                content.Add(fileContent, "file", "avatar");

                using var response = await _httpClient.PostAsync("users/me/avatar", content);
                if (!response.IsSuccessStatusCode)
                {
                    return null;
                }
                var dto = await response.Content.ReadFromJsonAsync<AvatarUpdatedResponse>();
                return dto?.AvatarUpdatedAt;
            }
            catch
            {
                return null;
            }
        }

        /// <inheritdoc/>
        public async Task<bool> DeleteAvatarAsync()
        {
            try
            {
                using var response = await _httpClient.DeleteAsync("users/me/avatar");
                return response.IsSuccessStatusCode;
            }
            catch
            {
                return false;
            }
        }

        // Server response for a successful avatar upload — just the new marker.
        private sealed class AvatarUpdatedResponse
        {
            [JsonPropertyName("avatar_updated_at")]
            public string? AvatarUpdatedAt { get; set; }
        }
    }
}
