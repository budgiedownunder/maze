using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The subset of the server's Play 3D config the client consumes: the curated
    /// difficulty's fixed maze seed, used to key its leaderboard
    /// (<c>challenge = "&lt;difficulty&gt;:&lt;seed&gt;"</c>). The server response
    /// carries further fields (dimensions, time limit, …) that are not needed here.
    /// </summary>
    public class Play3dConfig
    {
        /// <summary>Difficulty label (<c>easy</c> / <c>tricky</c> / <c>hard</c>).</summary>
        [JsonPropertyName("difficulty")]
        public string Difficulty { get; set; } = "";

        /// <summary>The difficulty's fixed RNG seed.</summary>
        [JsonPropertyName("seed")]
        public ulong Seed { get; set; }
    }
}
