using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Read-only access to the stored 3D game library — definitions, collections
    /// and the featured catalogue — plus their images. The app <b>plays</b> games
    /// authored in the web UI; it does not create or edit them, so this interface
    /// exposes reads only. Scoring lives in <see cref="IScoresService"/>.
    /// </summary>
    public interface IGameLibraryService
    {
        /// <summary>
        /// Reads a page of game definitions in the given <paramref name="scope"/>
        /// (default <see cref="GameListScope.Visible"/>). <paramref name="query"/> is
        /// honoured with <see cref="GameListScope.Mine"/> / <see cref="GameListScope.Public"/>,
        /// and <paramref name="sort"/> with <see cref="GameListScope.Public"/>.
        /// </summary>
        /// <param name="scope">Result scope, or <c>null</c> for the server default</param>
        /// <param name="query">Case-insensitive name filter, or <c>null</c></param>
        /// <param name="sort">Result ordering, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>A page of definitions</returns>
        Task<GameDefinitionListResponse> ListGameDefinitionsAsync(
            GameListScope? scope = null, string? query = null, GameListSort? sort = null, int? limit = null, int? offset = null);

        /// <summary>
        /// Play-fetch of a single definition — access-checked (a game the caller
        /// can't see 404s). The returned <c>config</c> has the effective seed spliced
        /// in (date-mixed for a daily game), and the response carries the leaderboard
        /// <c>challengeKey</c>.
        /// </summary>
        /// <param name="id">The definition id</param>
        /// <returns>The definition plus its play-time challenge key</returns>
        Task<GamePlayResponse> GetGameDefinitionAsync(string id);

        /// <summary>
        /// Reads a page of game collections in the given <paramref name="scope"/>
        /// (same scope/query/sort rules as <see cref="ListGameDefinitionsAsync"/>).
        /// </summary>
        /// <param name="scope">Result scope, or <c>null</c> for the server default</param>
        /// <param name="query">Case-insensitive name filter, or <c>null</c></param>
        /// <param name="sort">Result ordering, or <c>null</c></param>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>A page of collections</returns>
        Task<GameCollectionListResponse> ListGameCollectionsAsync(
            GameListScope? scope = null, string? query = null, GameListSort? sort = null, int? limit = null, int? offset = null);

        /// <summary>
        /// Reads a collection's detail — its metadata plus its member definitions,
        /// hydrated, in order, and filtered to what the caller may access.
        /// </summary>
        /// <param name="id">The collection id</param>
        /// <returns>The collection and its accessible members</returns>
        Task<GameCollectionDetailResponse> GetGameCollectionAsync(string id);

        /// <summary>
        /// Reads a page of the admin-ordered featured catalogue (curated definitions
        /// + collections, hydrated and in sort order).
        /// </summary>
        /// <param name="limit">Page size, or <c>null</c></param>
        /// <param name="offset">Page offset, or <c>null</c></param>
        /// <returns>A page of the featured catalogue</returns>
        Task<FeaturedGameItemsListResponse> GetFeaturedGameItemsAsync(int? limit = null, int? offset = null);

        /// <summary>
        /// Fetches a definition's or collection's image bytes over an authenticated
        /// request, or <c>null</c> when it has none (a 404). Access-checked like the
        /// play-fetch; <paramref name="imageUpdatedAt"/> is the cache-buster marker.
        /// </summary>
        /// <param name="kind">Which entity the image belongs to</param>
        /// <param name="id">The entity id</param>
        /// <param name="imageUpdatedAt">The image marker, or <c>null</c></param>
        /// <returns>The PNG bytes, or <c>null</c> when there is no image</returns>
        Task<byte[]?> GetGameImageAsync(GameEntityKind kind, string id, string? imageUpdatedAt = null);
    }
}
