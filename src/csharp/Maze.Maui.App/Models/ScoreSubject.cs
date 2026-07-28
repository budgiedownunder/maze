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

        /// <summary>A board scoped to a challenge string (e.g. a stored game's <c>def:&lt;id&gt;</c>).</summary>
        /// <param name="challenge">The challenge board key</param>
        /// <returns>The subject</returns>
        public static ScoreSubject ForChallenge(string challenge) => new(null, challenge);

        /// <summary>A board scoped to a stored 3D game definition — its <c>def:&lt;id&gt;</c> board.</summary>
        /// <param name="definitionId">The game definition id</param>
        /// <returns>The subject</returns>
        public static ScoreSubject ForDefinition(string definitionId) => ForChallenge($"def:{definitionId}");
    }
}
