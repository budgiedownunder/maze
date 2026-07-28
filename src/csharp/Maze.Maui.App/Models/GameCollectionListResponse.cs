using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A page of game collections the caller may see. <see cref="Limit"/> is the
    /// effective (server-capped) page size and <see cref="HasMore"/> says whether a
    /// further page exists.
    /// </summary>
    public class GameCollectionListResponse
    {
        /// <summary>The page of collections, de-duplicated and ordered.</summary>
        [JsonPropertyName("collections")]
        public List<GameCollection> Collections { get; set; } = new();

        /// <summary>The effective (server-capped) page size.</summary>
        [JsonPropertyName("limit")]
        public int Limit { get; set; }

        /// <summary>The offset this page started at.</summary>
        [JsonPropertyName("offset")]
        public int Offset { get; set; }

        /// <summary>Whether a further page exists beyond this one.</summary>
        [JsonPropertyName("hasMore")]
        public bool HasMore { get; set; }
    }
}
