using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A stored, parametric 3D game (camelCase wire shape). Unlike a maze it holds
    /// no grid: <see cref="Config"/> is an opaque, client-owned JSON blob the host
    /// page consumes verbatim, so the app does not interpret it. <see cref="Seed"/>
    /// is server-owned. <see cref="Visibility"/> / <see cref="Rotation"/> are the
    /// lenient lowercase wire strings (see <see cref="Models.GameVocabulary"/>).
    /// </summary>
    public class GameDefinition
    {
        /// <summary>Unique identifier.</summary>
        [JsonPropertyName("id")]
        public string Id { get; set; } = "";

        /// <summary>The user that owns this definition.</summary>
        [JsonPropertyName("ownerId")]
        public string OwnerId { get; set; } = "";

        /// <summary>Display name.</summary>
        [JsonPropertyName("name")]
        public string Name { get; set; } = "";

        /// <summary>Optional description; <c>null</c> when unset.</summary>
        [JsonPropertyName("description")]
        public string? Description { get; set; }

        /// <summary>Access tier: <c>private</c> / <c>shared</c> / <c>public</c> / <c>curated</c>.</summary>
        [JsonPropertyName("visibility")]
        public string Visibility { get; set; } = GameVocabulary.Visibility.Private;

        /// <summary>Generation seed (server-owned; hidden from the editor).</summary>
        [JsonPropertyName("seed")]
        public ulong Seed { get; set; }

        /// <summary>Layout/board rotation: <c>static</c> or <c>daily</c>.</summary>
        [JsonPropertyName("rotation")]
        public string Rotation { get; set; } = GameVocabulary.Rotation.Static;

        /// <summary>Opaque, client-owned generation + render parameters, forwarded verbatim to the host page.</summary>
        [JsonPropertyName("config")]
        public JsonElement Config { get; set; }

        /// <summary>Cache-key for the game's optional image; <c>null</c> when unset.</summary>
        [JsonPropertyName("imageUpdatedAt")]
        public string? ImageUpdatedAt { get; set; }

        /// <summary>Creation timestamp.</summary>
        [JsonPropertyName("createdAt")]
        public DateTimeOffset CreatedAt { get; set; }

        /// <summary>Last-update timestamp.</summary>
        [JsonPropertyName("updatedAt")]
        public DateTimeOffset UpdatedAt { get; set; }
    }
}
