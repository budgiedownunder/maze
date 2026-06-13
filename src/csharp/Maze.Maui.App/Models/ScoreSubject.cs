namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The subject a leaderboard ranks — exactly one of a stored maze id or a
    /// curated challenge string. Constructed via the factory methods so an
    /// instance always names exactly one of the two.
    /// </summary>
    public readonly struct ScoreSubject
    {
        /// <summary>The stored maze id, or <c>null</c> for a curated game.</summary>
        public string? MazeId { get; }

        /// <summary>The curated challenge string, or <c>null</c> for a user maze.</summary>
        public string? Challenge { get; }

        private ScoreSubject(string? mazeId, string? challenge)
        {
            MazeId = mazeId;
            Challenge = challenge;
        }

        /// <summary>A board scoped to a stored user maze.</summary>
        /// <param name="mazeId">The maze id</param>
        /// <returns>The subject</returns>
        public static ScoreSubject ForMaze(string mazeId) => new(mazeId, null);

        /// <summary>A board scoped to a curated challenge string.</summary>
        /// <param name="challenge">The <c>"&lt;difficulty&gt;:&lt;seed&gt;"</c> string</param>
        /// <returns>The subject</returns>
        public static ScoreSubject ForChallenge(string challenge) => new(null, challenge);

        /// <summary>A board scoped to a curated game, built from its difficulty and seed.</summary>
        /// <param name="difficulty">Difficulty label (<c>easy</c> / <c>tricky</c> / <c>hard</c>)</param>
        /// <param name="seed">The difficulty's fixed seed</param>
        /// <returns>The subject</returns>
        public static ScoreSubject ForCuratedGame(string difficulty, ulong seed) =>
            ForChallenge(BuildChallenge(difficulty, seed));

        /// <summary>
        /// Canonical form of a curated-challenge subject: <c>"&lt;difficulty&gt;:&lt;seed&gt;"</c>.
        /// The single source for the convention on the C# side, matching the game
        /// host and the server's challenge keying.
        /// </summary>
        /// <param name="difficulty">Difficulty label</param>
        /// <param name="seed">The difficulty's fixed seed</param>
        /// <returns>The challenge string</returns>
        public static string BuildChallenge(string difficulty, ulong seed) => $"{difficulty}:{seed}";
    }
}
