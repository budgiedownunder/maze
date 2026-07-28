using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Pure helpers that assemble the relative request paths for the game-library
    /// (definition / collection / featured / image) read endpoints. Kept free of
    /// HTTP / runtime dependencies so the query-string logic is unit-testable in
    /// isolation (the HTTP client service delegates to these). Paths are relative —
    /// the client's base address already includes <c>/api/v1/</c>.
    /// </summary>
    public static class GameLibraryRequestPaths
    {
        /// <summary>Assembles <c>game-definitions</c> for a list page (see <see cref="BuildListPath"/>).</summary>
        /// <param name="scope">Result scope, or <c>null</c> for the server default (<c>visible</c>)</param>
        /// <param name="query">Case-insensitive name filter, or <c>null</c>/empty</param>
        /// <param name="sort">Result ordering, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildDefinitionListPath(GameListScope? scope, string? query, GameListSort? sort, int? limit, int? offset)
            => BuildListPath("game-definitions", scope, query, sort, limit, offset);

        /// <summary>Assembles <c>game-collections</c> for a list page (see <see cref="BuildListPath"/>).</summary>
        /// <param name="scope">Result scope, or <c>null</c> for the server default (<c>visible</c>)</param>
        /// <param name="query">Case-insensitive name filter, or <c>null</c>/empty</param>
        /// <param name="sort">Result ordering, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildCollectionListPath(GameListScope? scope, string? query, GameListSort? sort, int? limit, int? offset)
            => BuildListPath("game-collections", scope, query, sort, limit, offset);

        /// <summary>Assembles the play-fetch path for a single definition.</summary>
        /// <param name="id">The definition id</param>
        /// <returns>The relative request path</returns>
        public static string BuildDefinitionPath(string id) => $"game-definitions/{Uri.EscapeDataString(id)}";

        /// <summary>Assembles the detail path for a single collection.</summary>
        /// <param name="id">The collection id</param>
        /// <returns>The relative request path</returns>
        public static string BuildCollectionPath(string id) => $"game-collections/{Uri.EscapeDataString(id)}";

        /// <summary>Assembles the featured-catalogue path for a page; paging omitted when <c>null</c>.</summary>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildFeaturedPath(int? limit, int? offset)
        {
            var query = new List<string>();
            if (limit is not null) query.Add($"limit={limit.Value}");
            if (offset is not null) query.Add($"offset={offset.Value}");
            return query.Count > 0 ? $"featured-game-items?{string.Join("&", query)}" : "featured-game-items";
        }

        /// <summary>Assembles the image serve-GET path for a definition or collection, with the
        /// <paramref name="updatedAt"/> marker as a <c>?v=</c> cache-buster when known.</summary>
        /// <param name="kind">Which entity the image belongs to</param>
        /// <param name="id">The entity id</param>
        /// <param name="updatedAt">The <c>imageUpdatedAt</c> marker, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildImagePath(GameEntityKind kind, string id, string? updatedAt)
        {
            string basePath = $"{kind.ToPathSegment()}/{Uri.EscapeDataString(id)}/image";
            return string.IsNullOrEmpty(updatedAt) ? basePath : $"{basePath}?v={Uri.EscapeDataString(updatedAt)}";
        }

        /// <summary>
        /// Assembles a scoped list path for the game-definition / game-collection
        /// lists (identical query shape). The <c>q</c> filter is honoured with
        /// <see cref="GameListScope.Mine"/> / <see cref="GameListScope.Public"/> and
        /// <c>sort</c> with <see cref="GameListScope.Public"/>; the server ignores
        /// the rest. Optional values are omitted when <c>null</c>/empty.
        /// </summary>
        /// <param name="entity">The list endpoint (<c>game-definitions</c> / <c>game-collections</c>)</param>
        /// <param name="scope">Result scope, or <c>null</c></param>
        /// <param name="query">Case-insensitive name filter, or <c>null</c>/empty</param>
        /// <param name="sort">Result ordering, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>The relative request path</returns>
        public static string BuildListPath(string entity, GameListScope? scope, string? query, GameListSort? sort, int? limit, int? offset)
        {
            var parts = new List<string>();
            if (scope is not null) parts.Add($"scope={scope.Value.ToQueryValue()}");
            if (!string.IsNullOrEmpty(query)) parts.Add($"q={Uri.EscapeDataString(query)}");
            if (sort is not null) parts.Add($"sort={sort.Value.ToQueryValue()}");
            if (limit is not null) parts.Add($"limit={limit.Value}");
            if (offset is not null) parts.Add($"offset={offset.Value}");
            return parts.Count > 0 ? $"{entity}?{string.Join("&", parts)}" : entity;
        }
    }
}
