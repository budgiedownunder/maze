using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A page of a leaderboard or personal history. Mirrors the server's score
    /// board response: <see cref="Limit"/> is the effective (server-capped) page
    /// size and <see cref="HasMore"/> says whether a further page exists.
    /// </summary>
    public class ScoreboardResponse
    {
        /// <summary>The page of entries, ordered by the request's metric/direction.</summary>
        [JsonPropertyName("scores")]
        public List<ScoreEntry> Scores { get; set; } = new();

        /// <summary>The effective (server-capped) page size.</summary>
        [JsonPropertyName("limit")]
        public int Limit { get; set; }

        /// <summary>The offset this page started at.</summary>
        [JsonPropertyName("offset")]
        public int Offset { get; set; }

        /// <summary>Whether a further page exists beyond this one.</summary>
        [JsonPropertyName("has_more")]
        public bool HasMore { get; set; }
    }
}
