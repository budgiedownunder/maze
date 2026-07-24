using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// One ordered member of a collection — a reference to a definition by id.
    /// Presentation (name, description, image) is intrinsic to the referenced
    /// definition, so it is not repeated here.
    /// </summary>
    public class CollectionItem
    {
        /// <summary>The game definition this membership points at.</summary>
        [JsonPropertyName("definitionId")]
        public string DefinitionId { get; set; } = "";

        /// <summary>Position within the collection (ascending).</summary>
        [JsonPropertyName("sortOrder")]
        public uint SortOrder { get; set; }
    }
}
