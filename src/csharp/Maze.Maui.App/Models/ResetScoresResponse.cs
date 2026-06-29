using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Result of resetting a leaderboard (DELETE <c>/scores</c>): the number of
    /// score rows removed. Mirrors the server's reset response.
    /// </summary>
    public class ResetScoresResponse
    {
        /// <summary>The number of score rows removed.</summary>
        [JsonPropertyName("deleted")]
        public long Deleted { get; set; }
    }
}
