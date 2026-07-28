using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The play-fetch of a single definition (<c>GET /game-definitions/{id}</c>):
    /// the definition (its <c>config</c> carries the effective, possibly
    /// date-mixed, seed) plus its leaderboard subject key and whether that board is
    /// tracked. The definition fields are flattened onto the same JSON object, so
    /// this inherits them.
    /// </summary>
    public class GamePlayResponse : GameDefinition
    {
        /// <summary>The leaderboard subject to record runs against: <c>def:&lt;id&gt;</c> for a
        /// static game, <c>def:&lt;id&gt;:&lt;yyyy-mm-dd&gt;</c> (today, UTC) for a daily one.</summary>
        [JsonPropertyName("challengeKey")]
        public string ChallengeKey { get; set; } = "";

        /// <summary>Whether runs are leaderboard-tracked (published games only).</summary>
        [JsonPropertyName("leaderboardTracked")]
        public bool LeaderboardTracked { get; set; }
    }
}
