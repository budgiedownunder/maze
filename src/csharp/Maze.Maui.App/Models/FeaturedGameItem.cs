using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// One hydrated entry of the admin-ordered featured catalogue. Exactly one of
    /// <see cref="Definition"/> / <see cref="Collection"/> is present, matching
    /// <see cref="Kind"/> (<c>definition</c> / <c>collection</c>).
    /// <see cref="OwnerUsername"/> is resolved server-side.
    /// </summary>
    public class FeaturedGameItem
    {
        /// <summary>Which kind of entity this row points at (<c>definition</c> / <c>collection</c>).</summary>
        [JsonPropertyName("kind")]
        public string Kind { get; set; } = "";

        /// <summary>The owning user's username, resolved server-side.</summary>
        [JsonPropertyName("ownerUsername")]
        public string OwnerUsername { get; set; } = "";

        /// <summary>The hydrated definition when <see cref="Kind"/> is <c>definition</c>; else <c>null</c>.</summary>
        [JsonPropertyName("definition")]
        public GameDefinition? Definition { get; set; }

        /// <summary>The hydrated collection when <see cref="Kind"/> is <c>collection</c>; else <c>null</c>.</summary>
        [JsonPropertyName("collection")]
        public GameCollection? Collection { get; set; }
    }
}
