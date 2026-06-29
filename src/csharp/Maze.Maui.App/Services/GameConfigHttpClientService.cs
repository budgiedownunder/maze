using Maze.Maui.App.Models;
using System.Net.Http.Json;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// HTTP client service for server-controlled game configuration. The Play 3D
    /// config endpoint is unauthenticated, so this uses a plain client (no Bearer
    /// token), mirroring <see cref="AppFeaturesHttpClientService"/>.
    /// </summary>
    public class GameConfigHttpClientService : IGameConfigService
    {
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        public GameConfigHttpClientService(ConfigurationService configurationService)
        {
            _httpClient = ApiHttpClientFactory.Create(configurationService);
        }

        /// <inheritdoc/>
        public async Task<Play3dConfig> GetPlay3dConfigAsync(Difficulty difficulty)
        {
            string path = $"game/play3d-config?difficulty={difficulty.ToQueryValue()}";
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<Play3dConfig>() ?? new Play3dConfig();
        }
    }
}
