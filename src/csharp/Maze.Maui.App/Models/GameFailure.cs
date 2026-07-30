using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Failure payload posted by the hosted <c>/game/</c> page when a 3D run dies
    /// instead of finishing — a Rust panic, an uncaught script error, or a
    /// renderer that ran out of memory. Captured by
    /// <see cref="Views.Play3dGamePage"/> via the platform WebView message bridge,
    /// which shows <see cref="Reason"/> to the player and logs the whole payload.
    ///
    /// Distinguished from a <see cref="GameResult"/> by the envelope's
    /// <c>kind</c> field — see <see cref="HostMessage"/>. Both types are lenient
    /// about missing fields, because a run that died mid-frame may not be able to
    /// describe itself fully.
    /// </summary>
    public class GameFailure
    {
        /// <summary>
        /// Short human-readable cause, shown to the player verbatim (e.g.
        /// "The game ran out of memory"). Empty when the host could not
        /// classify the failure.
        /// </summary>
        [JsonPropertyName("reason")]
        public string Reason { get; set; } = "";

        /// <summary>
        /// Diagnostic detail for the log — the underlying error text, or a Rust
        /// panic's message and source location. Not shown to the player.
        /// </summary>
        [JsonPropertyName("detail")]
        public string? Detail { get; set; }

        /// <summary>
        /// Where the run was when it died (e.g. <c>load</c>, <c>play</c>), so a
        /// log can tell a failure to start apart from one after minutes of play.
        /// </summary>
        [JsonPropertyName("phase")]
        public string? Phase { get; set; }

        private static readonly JsonSerializerOptions s_jsonOptions = new()
        {
            PropertyNameCaseInsensitive = true,
        };

        /// <summary>
        /// Parses a raw failure JSON payload. Returns <c>null</c> on malformed
        /// JSON rather than throwing, matching <see cref="GameResult.FromJson"/>
        /// so the message-bridge path can log-and-ignore a bad payload.
        /// </summary>
        /// <param name="json">Raw JSON string from the WebView bridge</param>
        /// <returns>The parsed <see cref="GameFailure"/>, or <c>null</c> if the JSON could not be parsed</returns>
        public static GameFailure? FromJson(string json)
        {
            try
            {
                return JsonSerializer.Deserialize<GameFailure>(json, s_jsonOptions);
            }
            catch (JsonException)
            {
                return null;
            }
        }
    }
}
