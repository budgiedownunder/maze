using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Outcome of a 3D-game session — mirrors the Bevy <c>GameOutcome</c> enum.
    /// </summary>
    public enum GameOutcome
    {
        /// <summary>The player reached the finish cell in time.</summary>
        Win,
        /// <summary>The countdown expired before the player finished.</summary>
        Lose,
    }

    /// <summary>
    /// Result payload posted by the hosted <c>/game/</c> page when a 3D game
    /// ends, captured by <see cref="Views.Play3dGamePage"/> via the platform
    /// WebView message bridge. Mirrors the Bevy <c>GameResult</c> contract
    /// (camelCase JSON). <see cref="Extras"/> is an open map so future
    /// per-feature metrics can be added without breaking this type.
    ///
    /// Currently captured and logged only — leaderboard recording is future work.
    /// </summary>
    public class GameResult
    {
        /// <summary>Win or lose.</summary>
        [JsonPropertyName("outcome")]
        [JsonConverter(typeof(JsonStringEnumConverter))]
        public GameOutcome Outcome { get; set; }

        /// <summary>In-game elapsed time, in milliseconds, when the game ended.</summary>
        [JsonPropertyName("elapsedMs")]
        public long ElapsedMs { get; set; }

        /// <summary>Difficulty label (<c>easy</c> / <c>tricky</c> / <c>hard</c>),
        /// or <c>null</c> for the no-config / specific-maze paths.</summary>
        [JsonPropertyName("difficulty")]
        public string? Difficulty { get; set; }

        /// <summary>Maze row count.</summary>
        [JsonPropertyName("rows")]
        public uint Rows { get; set; }

        /// <summary>Maze column count.</summary>
        [JsonPropertyName("cols")]
        public uint Cols { get; set; }

        /// <summary>Seed of the maze played, or <c>null</c> when no seed was supplied.</summary>
        [JsonPropertyName("seed")]
        public ulong? Seed { get; set; }

        /// <summary>Open map of future per-feature metrics. <c>null</c> when the
        /// game sent none (the Bevy side omits it when empty).</summary>
        [JsonPropertyName("extras")]
        public Dictionary<string, JsonElement>? Extras { get; set; }

        private static readonly JsonSerializerOptions s_jsonOptions = new()
        {
            PropertyNameCaseInsensitive = true,
        };

        /// <summary>
        /// Parses a raw GameResult JSON payload. Returns <c>null</c> on malformed
        /// JSON rather than throwing, so callers on the message-bridge path can
        /// log-and-ignore a bad payload.
        /// </summary>
        /// <param name="json">Raw JSON string from the WebView bridge</param>
        /// <returns>The parsed <see cref="GameResult"/>, or <c>null</c> if the JSON could not be parsed</returns>
        public static GameResult? FromJson(string json)
        {
            try
            {
                return JsonSerializer.Deserialize<GameResult>(json, s_jsonOptions);
            }
            catch (JsonException)
            {
                return null;
            }
        }
    }
}
