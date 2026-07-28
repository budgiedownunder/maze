using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Collection detail (<c>GET /game-collections/{id}</c>): the collection's own
    /// metadata plus its member definitions — hydrated, in order, and filtered to
    /// what the viewer may access (inaccessible members and dangling refs are
    /// omitted server-side). This is why it carries <see cref="Definitions"/>
    /// rather than the raw membership.
    /// </summary>
    public class GameCollectionDetailResponse
    {
        /// <summary>Unique identifier.</summary>
        [JsonPropertyName("id")]
        public string Id { get; set; } = "";

        /// <summary>The user that owns this collection.</summary>
        [JsonPropertyName("ownerId")]
        public string OwnerId { get; set; } = "";

        /// <summary>Display name.</summary>
        [JsonPropertyName("name")]
        public string Name { get; set; } = "";

        /// <summary>Optional collection-level description; <c>null</c> when unset.</summary>
        [JsonPropertyName("description")]
        public string? Description { get; set; }

        /// <summary>Access tier: <c>private</c> / <c>shared</c> / <c>public</c> / <c>curated</c>.</summary>
        [JsonPropertyName("visibility")]
        public string Visibility { get; set; } = GameVocabulary.Visibility.Private;

        /// <summary>How the collection is played: <c>arcade</c> or <c>campaign</c>.</summary>
        [JsonPropertyName("playMode")]
        public string PlayMode { get; set; } = GameVocabulary.PlayMode.Arcade;

        /// <summary>Cache-key for the collection's optional image; <c>null</c> when unset.</summary>
        [JsonPropertyName("imageUpdatedAt")]
        public string? ImageUpdatedAt { get; set; }

        /// <summary>Creation timestamp.</summary>
        [JsonPropertyName("createdAt")]
        public DateTimeOffset CreatedAt { get; set; }

        /// <summary>Last-update timestamp.</summary>
        [JsonPropertyName("updatedAt")]
        public DateTimeOffset UpdatedAt { get; set; }

        /// <summary>The accessible member definitions, in collection order.</summary>
        [JsonPropertyName("definitions")]
        public List<GameDefinition> Definitions { get; set; } = new();
    }
}
