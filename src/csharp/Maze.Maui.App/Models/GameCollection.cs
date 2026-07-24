using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// An ordered, presentation-only grouping of game definitions (camelCase wire
    /// shape). It carries its own <see cref="Visibility"/> and <see cref="PlayMode"/>
    /// (lenient lowercase wire strings — see <see cref="Models.GameVocabulary"/>);
    /// membership is order-only (<see cref="Items"/>).
    /// </summary>
    public class GameCollection
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

        /// <summary>Access tier: <c>private</c> / <c>shared</c> / <c>public</c> / <c>curated</c>.</summary>
        [JsonPropertyName("visibility")]
        public string Visibility { get; set; } = GameVocabulary.Visibility.Private;

        /// <summary>How the collection is played: <c>arcade</c> or <c>campaign</c>.</summary>
        [JsonPropertyName("playMode")]
        public string PlayMode { get; set; } = GameVocabulary.PlayMode.Arcade;

        /// <summary>Optional collection-level description; <c>null</c> when unset.</summary>
        [JsonPropertyName("description")]
        public string? Description { get; set; }

        /// <summary>Cache-key for the collection's optional image; <c>null</c> when unset.</summary>
        [JsonPropertyName("imageUpdatedAt")]
        public string? ImageUpdatedAt { get; set; }

        /// <summary>The member games, in display order.</summary>
        [JsonPropertyName("items")]
        public List<CollectionItem> Items { get; set; } = new();

        /// <summary>Creation timestamp.</summary>
        [JsonPropertyName("createdAt")]
        public DateTimeOffset CreatedAt { get; set; }

        /// <summary>Last-update timestamp.</summary>
        [JsonPropertyName("updatedAt")]
        public DateTimeOffset UpdatedAt { get; set; }
    }
}
