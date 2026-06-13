namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The metric a leaderboard ranks by — sent as the <c>metric</c> query value
    /// (see <see cref="ScoreVocabularyExtensions.ToQueryValue(ScoreMetric)"/>).
    /// </summary>
    public enum ScoreMetric
    {
        /// <summary>Rank by elapsed run time.</summary>
        Time,
        /// <summary>Rank by final score.</summary>
        Score,
    }

    /// <summary>
    /// The primary metric's sort direction — sent as the <c>direction</c> query
    /// value (see <see cref="ScoreVocabularyExtensions.ToQueryValue(SortDirection)"/>).
    /// </summary>
    public enum SortDirection
    {
        /// <summary>Ascending (e.g. fastest time first).</summary>
        Ascending,
        /// <summary>Descending (e.g. highest score first).</summary>
        Descending,
    }

    /// <summary>
    /// Maps the score-ranking vocabulary to the lowercase tokens the score
    /// endpoints expect, mirroring the values the server parses.
    /// </summary>
    public static class ScoreVocabularyExtensions
    {
        /// <summary>Returns the <c>metric</c> query token (<c>time</c> / <c>score</c>).</summary>
        /// <param name="metric">Metric value</param>
        /// <returns>Lowercase query token</returns>
        public static string ToQueryValue(this ScoreMetric metric) => metric switch
        {
            ScoreMetric.Time => "time",
            ScoreMetric.Score => "score",
            _ => throw new ArgumentOutOfRangeException(nameof(metric), metric, null),
        };

        /// <summary>Returns the <c>direction</c> query token (<c>asc</c> / <c>desc</c>).</summary>
        /// <param name="direction">Direction value</param>
        /// <returns>Lowercase query token</returns>
        public static string ToQueryValue(this SortDirection direction) => direction switch
        {
            SortDirection.Ascending => "asc",
            SortDirection.Descending => "desc",
            _ => throw new ArgumentOutOfRangeException(nameof(direction), direction, null),
        };
    }
}
