using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Reads leaderboards and the authenticated player's run history from the
    /// score endpoints. Scoring only — game configuration (e.g. the Play 3D
    /// difficulty presets) lives in <see cref="IGameConfigService"/>.
    /// </summary>
    public interface IScoresService
    {
        /// <summary>
        /// Reads a page of a leaderboard for one subject (a stored maze or a
        /// curated challenge), ranked by <paramref name="metric"/> /
        /// <paramref name="direction"/> (the server defaults to best-first for the
        /// metric when omitted).
        /// </summary>
        /// <param name="subject">The board's subject (maze or challenge)</param>
        /// <param name="metric">Ranking metric, or <c>null</c> for the server default</param>
        /// <param name="direction">Sort direction, or <c>null</c> for the metric's natural order</param>
        /// <param name="limit">Page size, or <c>null</c> for the server default</param>
        /// <param name="offset">Page offset, or <c>null</c> for the start</param>
        /// <param name="includeUsernames">Resolve player usernames, or <c>null</c> for the server default</param>
        /// <returns>A page of the leaderboard</returns>
        Task<ScoreboardResponse> GetLeaderboardAsync(
            ScoreSubject subject,
            ScoreMetric? metric = null,
            SortDirection? direction = null,
            int? limit = null,
            int? offset = null,
            bool? includeUsernames = null);

        /// <summary>
        /// Reads a page of the authenticated player's own run history (most recent
        /// first).
        /// </summary>
        /// <param name="limit">Page size, or <c>null</c> for the server default</param>
        /// <param name="offset">Page offset, or <c>null</c> for the start</param>
        /// <returns>A page of the player's history</returns>
        Task<ScoreboardResponse> GetScoreHistoryAsync(int? limit = null, int? offset = null);
    }
}
