using Maze.Maui.App.Models;
using System.Net.Http.Json;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// HTTP client service for reading leaderboards and the caller's run history.
    /// Uses the same Bearer-token-injecting pipeline as
    /// <see cref="MazeHttpClientService"/>. Request paths are assembled by
    /// <see cref="ScoreRequestPaths"/> so the query-string logic is unit-testable
    /// without an HTTP round-trip.
    /// </summary>
    public class ScoresHttpClientService : IScoresService
    {
        private readonly HttpClient _httpClient;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        /// <param name="authService">Injected auth service</param>
        public ScoresHttpClientService(ConfigurationService configurationService, IAuthService authService)
        {
            var innerHandler = ApiHttpClientFactory.CreateHandler(configurationService);
            var bearerHandler = new BearerTokenHandler(authService, configurationService, innerHandler);
            _httpClient = ApiHttpClientFactory.Create(configurationService, bearerHandler);
        }

        /// <inheritdoc/>
        public async Task<ScoreboardResponse> GetLeaderboardAsync(
            ScoreSubject subject,
            ScoreMetric? metric = null,
            SortDirection? direction = null,
            int? limit = null,
            int? offset = null,
            bool? includeUsernames = null)
        {
            string path = ScoreRequestPaths.BuildLeaderboardPath(subject.MazeId, subject.Challenge, metric, direction, limit, offset, includeUsernames);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<ScoreboardResponse>() ?? new ScoreboardResponse();
        }

        /// <inheritdoc/>
        public async Task<ScoreboardResponse> GetScoreHistoryAsync(int? limit = null, int? offset = null)
        {
            string path = ScoreRequestPaths.BuildHistoryPath(limit, offset);
            var response = await _httpClient.GetAsync(path);
            response.EnsureSuccessStatusCode();
            return await response.Content.ReadFromJsonAsync<ScoreboardResponse>() ?? new ScoreboardResponse();
        }

        /// <inheritdoc/>
        public async Task<long> ClearLeaderboardAsync(ScoreSubject subject)
        {
            string path = ScoreRequestPaths.BuildResetPath(subject.MazeId, subject.Challenge);
            var response = await _httpClient.DeleteAsync(path);
            response.EnsureSuccessStatusCode();
            ResetScoresResponse? result = await response.Content.ReadFromJsonAsync<ResetScoresResponse>();
            return result?.Deleted ?? 0;
        }
    }
}
