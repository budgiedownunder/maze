using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Reads leaderboards and the authenticated player's run history from the
    /// score endpoints. Scoring only.
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

        /// <summary>
        /// Resets a leaderboard to empty, deleting every score for one subject (a
        /// stored maze or a curated challenge). The server enforces access — a
        /// stored maze's board is the maze owner's to clear; a curated challenge's
        /// board is admin-only — and rejects the request otherwise.
        /// </summary>
        /// <param name="subject">The board's subject (maze or challenge)</param>
        /// <returns>The number of score rows removed</returns>
        Task<long> ClearLeaderboardAsync(ScoreSubject subject);

        /// <summary>
        /// Reads the UTC dates a daily game has a non-empty leaderboard for (the
        /// days someone has scored on that day's board), most recent first. A static
        /// game — or an unplayed daily one — returns an empty list. Access-checked
        /// like the board read.
        /// </summary>
        /// <param name="definitionId">The daily game's definition id</param>
        /// <returns>The dates with a board, most recent first</returns>
        Task<BoardDatesResponse> GetBoardDatesAsync(string definitionId);

        /// <summary>
        /// Given challenge board keys (e.g. a campaign's games as
        /// <c>def:&lt;id&gt;</c>), returns the subset the caller has scored on — one
        /// request for campaign progress instead of paging the whole history.
        /// </summary>
        /// <param name="challenges">The challenge board keys to check</param>
        /// <returns>The subset the caller has completed</returns>
        Task<CompletedChallengesResponse> GetCompletedChallengesAsync(IReadOnlyList<string> challenges);
    }
}
