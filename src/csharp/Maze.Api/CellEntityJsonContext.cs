using System.Text.Json.Serialization;

namespace Maze.Api
{
    /// <summary>
    /// Source-generated JSON metadata for <see cref="CellEntityInfo"/> and its derived
    /// types, so the per-cell override (de)serialisation in <see cref="Maze"/> is
    /// trim- and AOT-safe (the app is trimmed for mobile). Unset (null) override fields
    /// are omitted on write so the emitted entity matches the canonical wire form
    /// (e.g. <c>{"type":"E","damage":2}</c>); the Rust side round-trips it.
    /// </summary>
    [JsonSourceGenerationOptions(DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
    [JsonSerializable(typeof(CellEntityInfo))]
    internal sealed partial class CellEntityJsonContext : JsonSerializerContext
    {
    }
}
