using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A page of the admin-ordered featured catalogue — curated definitions +
    /// collections, hydrated and in sort order. <see cref="HasMore"/> says whether
    /// a further page exists.
    /// </summary>
    public class FeaturedGameItemsListResponse
    {
        /// <summary>The page of featured items, in admin (sort) order.</summary>
        [JsonPropertyName("items")]
        public List<FeaturedGameItem> Items { get; set; } = new();

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
