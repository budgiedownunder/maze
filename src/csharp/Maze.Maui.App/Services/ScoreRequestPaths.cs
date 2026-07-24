using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Pure helpers that assemble the relative request paths for the score
    /// endpoints. Kept free of HTTP / runtime dependencies so the query-string
    /// logic is unit-testable in isolation (the HTTP client service delegates to
    /// these).
    /// </summary>
    public static class ScoreRequestPaths
    {
        /// <summary>
        /// Assembles the relative <c>scores</c> request path for a leaderboard page.
        /// Exactly one of <paramref name="mazeId"/> / <paramref name="challenge"/>
        /// must be set; the optional ranking / paging values are omitted from the
        /// query string when <c>null</c> (the server applies its defaults).
        /// </summary>
        /// <param name="mazeId">Stored-maze subject, or <c>null</c></param>
        /// <param name="challenge">Curated-challenge subject, or <c>null</c></param>
        /// <param name="metric">Ranking metric, or <c>null</c></param>
        /// <param name="direction">Sort direction, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <param name="includeUsernames">Resolve usernames, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        /// <exception cref="ArgumentException">Neither or both subjects are set</exception>
        public static string BuildLeaderboardPath(
            string? mazeId,
            string? challenge,
            ScoreMetric? metric,
            SortDirection? direction,
            int? limit,
            int? offset,
            bool? includeUsernames)
        {
            if ((mazeId is null) == (challenge is null))
                throw new ArgumentException("A leaderboard requires exactly one of mazeId / challenge");

            var query = new List<string>();
            if (mazeId is not null) query.Add($"maze_id={Uri.EscapeDataString(mazeId)}");
            if (challenge is not null) query.Add($"challenge={Uri.EscapeDataString(challenge)}");
            if (metric is not null) query.Add($"metric={metric.Value.ToQueryValue()}");
            if (direction is not null) query.Add($"direction={direction.Value.ToQueryValue()}");
            if (limit is not null) query.Add($"limit={limit.Value}");
            if (offset is not null) query.Add($"offset={offset.Value}");
            if (includeUsernames is not null) query.Add($"include_usernames={(includeUsernames.Value ? "true" : "false")}");
            return $"scores?{string.Join("&", query)}";
        }

        /// <summary>
        /// Assembles the relative <c>scores</c> request path for resetting a
        /// leaderboard to empty (DELETE). Exactly one of <paramref name="mazeId"/> /
        /// <paramref name="challenge"/> must be set.
        /// </summary>
        /// <param name="mazeId">Stored-maze subject, or <c>null</c></param>
        /// <param name="challenge">Curated-challenge subject, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        /// <exception cref="ArgumentException">Neither or both subjects are set</exception>
        public static string BuildResetPath(string? mazeId, string? challenge)
        {
            if ((mazeId is null) == (challenge is null))
                throw new ArgumentException("A leaderboard reset requires exactly one of mazeId / challenge");

            string query = mazeId is not null
                ? $"maze_id={Uri.EscapeDataString(mazeId)}"
                : $"challenge={Uri.EscapeDataString(challenge!)}";
            return $"scores?{query}";
        }

        /// <summary>
        /// Assembles the relative <c>scores/me</c> request path for a history page;
        /// paging values are omitted from the query string when <c>null</c>.
        /// </summary>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildHistoryPath(int? limit, int? offset)
        {
            var query = new List<string>();
            if (limit is not null) query.Add($"limit={limit.Value}");
            if (offset is not null) query.Add($"offset={offset.Value}");
            return query.Count > 0 ? $"scores/me?{string.Join("&", query)}" : "scores/me";
        }

        /// <summary>
        /// Assembles the relative <c>scores/board-dates</c> request path — the UTC
        /// dates a daily game has a leaderboard for. The definition id is passed as
        /// the snake_case <c>definition_id</c> query param the server expects.
        /// </summary>
        /// <param name="definitionId">The daily game's definition id</param>
        /// <returns>The relative request path</returns>
        public static string BuildBoardDatesPath(string definitionId)
            => $"scores/board-dates?definition_id={Uri.EscapeDataString(definitionId)}";

        /// <summary>
        /// The relative <c>scores/me/completed</c> request path (POST) — given a set
        /// of challenge board keys, the subset the caller has scored on. The keys
        /// travel in the request body, so there is no query string.
        /// </summary>
        /// <returns>The relative request path</returns>
        public static string BuildCompletedPath() => "scores/me/completed";
    }
}
