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
    }
}
