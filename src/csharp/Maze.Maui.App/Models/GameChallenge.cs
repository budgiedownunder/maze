using System.Globalization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The leaderboard challenge-key convention for stored 3D games (the C# mirror
    /// of the web client's <c>gameChallengeKey</c> / <c>gameIdFromChallenge</c>): a
    /// static game's board is <c>def:&lt;id&gt;</c>; a daily game's is
    /// <c>def:&lt;id&gt;:&lt;yyyy-mm-dd&gt;</c> (a fresh board per UTC day).
    /// </summary>
    public static class GameChallenge
    {
        private const string DefPrefix = "def:";

        /// <summary>Today's UTC date as <c>yyyy-mm-dd</c> (the daily-board day boundary is UTC).</summary>
        /// <returns>The date string</returns>
        public static string TodayUtc() => DateTime.UtcNow.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture);

        /// <summary>
        /// The board key for a game — <c>def:&lt;id&gt;</c> for a static game,
        /// <c>def:&lt;id&gt;:&lt;date&gt;</c> for a daily one (defaulting to today, UTC).
        /// </summary>
        /// <param name="definitionId">The game definition id</param>
        /// <param name="rotation">The game's rotation (<c>static</c> / <c>daily</c>)</param>
        /// <param name="dateUtc">The daily board date, or <c>null</c> for today</param>
        /// <returns>The challenge board key</returns>
        public static string For(string definitionId, string rotation, string? dateUtc = null)
            => rotation == GameVocabulary.Rotation.Daily
                ? $"{DefPrefix}{definitionId}:{dateUtc ?? TodayUtc()}"
                : $"{DefPrefix}{definitionId}";

        /// <summary>
        /// The definition id a <c>def:</c> challenge refers to (dropping any daily
        /// date suffix), or <c>null</c> when the challenge is not a stored-game board.
        /// </summary>
        /// <param name="challenge">The challenge board key</param>
        /// <returns>The definition id, or <c>null</c></returns>
        public static string? DefinitionIdFromChallenge(string challenge)
        {
            if (!challenge.StartsWith(DefPrefix, StringComparison.Ordinal))
                return null;

            string rest = challenge[DefPrefix.Length..];
            int colon = rest.IndexOf(':');
            return colon >= 0 ? rest[..colon] : rest;
        }
    }
}
