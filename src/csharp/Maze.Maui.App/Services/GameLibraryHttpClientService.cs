using Maze.Maui.App.Models;
using System.Net;
using System.Net.Http.Json;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// HTTP client service for reading the stored 3D game library — definitions,
    /// collections, the featured catalogue and their images. Uses the same
    /// Bearer-token-injecting pipeline as <see cref="MazeHttpClientService"/>.
    /// Request paths are assembled by <see cref="GameLibraryRequestPaths"/> so the
    /// query-string logic is unit-testable without an HTTP round-trip.
    /// </summary>
    public class GameLibraryHttpClientService : IGameLibraryService
    {
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        /// <param name="authService">Injected auth service</param>
        public GameLibraryHttpClientService(ConfigurationService configurationService, IAuthService authService)
        {
            var innerHandler = ApiHttpClientFactory.CreateHandler(configurationService);
            var bearerHandler = new BearerTokenHandler(authService, configurationService, innerHandler);
            _httpClient = ApiHttpClientFactory.Create(configurationService, bearerHandler);
        }

        /// <inheritdoc/>
        public async Task<GameDefinitionListResponse> ListGameDefinitionsAsync(
            GameListScope? scope = null, string? query = null, GameListSort? sort = null, int? limit = null, int? offset = null)
        {
            string path = GameLibraryRequestPaths.BuildDefinitionListPath(scope, query, sort, limit, offset);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<GameDefinitionListResponse>() ?? new GameDefinitionListResponse();
        }

        /// <inheritdoc/>
        public async Task<GamePlayResponse> GetGameDefinitionAsync(string id)
        {
            string path = GameLibraryRequestPaths.BuildDefinitionPath(id);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<GamePlayResponse>() ?? new GamePlayResponse();
        }

        /// <inheritdoc/>
        public async Task<GameCollectionListResponse> ListGameCollectionsAsync(
            GameListScope? scope = null, string? query = null, GameListSort? sort = null, int? limit = null, int? offset = null)
        {
            string path = GameLibraryRequestPaths.BuildCollectionListPath(scope, query, sort, limit, offset);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<GameCollectionListResponse>() ?? new GameCollectionListResponse();
        }

        /// <inheritdoc/>
        public async Task<GameCollectionDetailResponse> GetGameCollectionAsync(string id)
        {
            string path = GameLibraryRequestPaths.BuildCollectionPath(id);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<GameCollectionDetailResponse>() ?? new GameCollectionDetailResponse();
        }

        /// <inheritdoc/>
        public async Task<FeaturedGameItemsListResponse> GetFeaturedGameItemsAsync(int? limit = null, int? offset = null)
        {
            string path = GameLibraryRequestPaths.BuildFeaturedPath(limit, offset);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<FeaturedGameItemsListResponse>() ?? new FeaturedGameItemsListResponse();
        }

        /// <inheritdoc/>
        public async Task<byte[]?> GetGameImageAsync(GameEntityKind kind, string id, string? imageUpdatedAt = null)
        {
            string path = GameLibraryRequestPaths.BuildImagePath(kind, id, imageUpdatedAt);
            var response = await _httpClient.GetAsync(path);
            if (response.StatusCode == HttpStatusCode.NotFound)
                return null; // no image — the caller falls back to a placeholder
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadAsByteArrayAsync();
        }
    }
}
