using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A recorded completed run, as returned by the score endpoints. Mirrors the
    /// server's <c>ScoreResponse</c> (snake_case JSON keys). Exactly one of
    /// <see cref="MazeId"/> / <see cref="Challenge"/> is set: a stored user maze
    /// or a curated <c>"&lt;difficulty&gt;:&lt;seed&gt;"</c> game.
    /// </summary>
    public class ScoreEntry
    {
        /// <summary>Row id (server-allocated).</summary>
        [JsonPropertyName("id")]
        public string Id { get; set; } = "";

        /// <summary>The player who recorded the run.</summary>
        [JsonPropertyName("user_id")]
        public string UserId { get; set; } = "";

        /// <summary>The stored maze played, or <c>null</c> for a curated game.</summary>
        [JsonPropertyName("maze_id")]
        public string? MazeId { get; set; }

        /// <summary>The curated game played (<c>"&lt;difficulty&gt;:&lt;seed&gt;"</c>),
        /// or <c>null</c> for a user maze.</summary>
        [JsonPropertyName("challenge")]
        public string? Challenge { get; set; }

        /// <summary>Final score at completion.</summary>
        [JsonPropertyName("score")]
        public ulong Score { get; set; }

        /// <summary>Elapsed run time in milliseconds.</summary>
        [JsonPropertyName("elapsed_ms")]
        public long ElapsedMs { get; set; }

        /// <summary>When the run was recorded (server-stamped).</summary>
        [JsonPropertyName("recorded_at")]
        public DateTimeOffset RecordedAt { get; set; }

        /// <summary>The player's username, when resolved by the server; otherwise
        /// <c>null</c> (the server omits it on personal boards).</summary>
        [JsonPropertyName("username")]
        public string? Username { get; set; }
    }
}
