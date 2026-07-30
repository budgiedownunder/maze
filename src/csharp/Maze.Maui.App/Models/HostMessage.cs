using System.Text.Json;

namespace Maze.Maui.App.Models
{
    /// <summary>What a payload posted by the hosted <c>/game/</c> page represents.</summary>
    public enum HostMessageKind
    {
        /// <summary>A finished run — parse with <see cref="GameResult.FromJson"/>.</summary>
        Result,
        /// <summary>A run that died — parse with <see cref="GameFailure.FromJson"/>.</summary>
        Failure,
    }

    /// <summary>
    /// Discriminator for the single message channel the hosted <c>/game/</c> page
    /// shares across every platform bridge. The page tags each payload with a
    /// <c>kind</c>; this reads that tag so the bridge can route without parsing
    /// the whole payload.
    ///
    /// Anything that is not explicitly <c>"failure"</c> is treated as a result —
    /// including a payload carrying no <c>kind</c> at all, which is what the host
    /// page sent before the envelope existed. Malformed JSON is likewise reported
    /// as a result, so it reaches <see cref="GameResult.FromJson"/> and produces
    /// the same "unparseable payload" warning as before rather than being
    /// silently reclassified.
    /// </summary>
    public static class HostMessage
    {
        /// <summary>
        /// Reads the envelope's <c>kind</c> without parsing the rest of the payload.
        /// </summary>
        /// <param name="json">Raw JSON string from the WebView bridge</param>
        /// <returns>The payload's kind; <see cref="HostMessageKind.Result"/> unless it is explicitly tagged as a failure</returns>
        public static HostMessageKind KindOf(string json)
        {
            try
            {
                using var document = JsonDocument.Parse(json);
                if (document.RootElement.ValueKind == JsonValueKind.Object
                    && document.RootElement.TryGetProperty("kind", out var kind)
                    && kind.ValueKind == JsonValueKind.String
                    && string.Equals(kind.GetString(), "failure", StringComparison.OrdinalIgnoreCase))
                {
                    return HostMessageKind.Failure;
                }
            }
            catch (JsonException)
            {
                // Malformed payload — fall through to Result so the existing
                // result path reports it, exactly as it did before the envelope.
            }
            return HostMessageKind.Result;
        }
    }
}
