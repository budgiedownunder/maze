using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Response for <c>POST /scores/me/completed</c>: the subset of the requested
    /// challenges the caller has scored on (used to derive campaign progress —
    /// win-only submission means a score on a game equals completion).
    /// </summary>
    public class CompletedChallengesResponse
    {
        /// <summary>The requested challenges the caller has a score on.</summary>
        [JsonPropertyName("completed")]
        public List<string> Completed { get; set; } = new();
    }
}
