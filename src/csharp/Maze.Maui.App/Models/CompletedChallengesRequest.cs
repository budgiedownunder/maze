using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Request body for <c>POST /scores/me/completed</c>: the challenge board keys
    /// (e.g. a campaign's games as <c>def:&lt;id&gt;</c>) to check the caller's
    /// completion of. Capped server-side.
    /// </summary>
    public class CompletedChallengesRequest
    {
        /// <summary>The challenge board keys to check.</summary>
        [JsonPropertyName("challenges")]
        public List<string> Challenges { get; set; } = new();
    }
}
