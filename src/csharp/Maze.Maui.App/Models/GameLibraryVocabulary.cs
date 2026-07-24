namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Which slice of the game-definition / game-collection lists to read — sent
    /// as the <c>scope</c> query value (see
    /// <see cref="GameLibraryVocabularyExtensions.ToQueryValue(GameListScope)"/>).
    /// </summary>
    public enum GameListScope
    {
        /// <summary>Everything the caller may see (own + shared + public + curated).</summary>
        Visible,
        /// <summary>Only the caller's own items (any visibility).</summary>
        Mine,
        /// <summary>Only items shared with the caller (not owned; excludes public/curated).</summary>
        Shared,
        /// <summary>The cross-owner Community pool (public items the caller doesn't own).</summary>
        Public,
    }

    /// <summary>
    /// Result ordering for a game list — sent as the <c>sort</c> query value.
    /// Honoured with <see cref="GameListScope.Public"/>; every other scope is
    /// name-ordered.
    /// </summary>
    public enum GameListSort
    {
        /// <summary>Case-insensitive A–Z by name (the default).</summary>
        Name,
        /// <summary>Most recently created first.</summary>
        Newest,
    }

    /// <summary>
    /// Which entity a game-library id refers to — selects the endpoint family and,
    /// on a featured-catalogue row, which of the two hydrated payloads is present.
    /// </summary>
    public enum GameEntityKind
    {
        /// <summary>A single stored game.</summary>
        Definition,
        /// <summary>A grouping of games.</summary>
        Collection,
    }

    /// <summary>
    /// Maps the game-library vocabulary to the lowercase tokens / path segments the
    /// server expects, mirroring the values it parses.
    /// </summary>
    public static class GameLibraryVocabularyExtensions
    {
        /// <summary>Returns the <c>scope</c> query token.</summary>
        /// <param name="scope">Scope value</param>
        /// <returns>Lowercase query token</returns>
        public static string ToQueryValue(this GameListScope scope) => scope switch
        {
            GameListScope.Visible => "visible",
            GameListScope.Mine => "mine",
            GameListScope.Shared => "shared",
            GameListScope.Public => "public",
            _ => throw new ArgumentOutOfRangeException(nameof(scope), scope, null),
        };

        /// <summary>Returns the <c>sort</c> query token.</summary>
        /// <param name="sort">Sort value</param>
        /// <returns>Lowercase query token</returns>
        public static string ToQueryValue(this GameListSort sort) => sort switch
        {
            GameListSort.Name => "name",
            GameListSort.Newest => "newest",
            _ => throw new ArgumentOutOfRangeException(nameof(sort), sort, null),
        };

        /// <summary>Returns the wire kind token (<c>definition</c> / <c>collection</c>).</summary>
        /// <param name="kind">Entity kind</param>
        /// <returns>Lowercase wire token</returns>
        public static string ToWireString(this GameEntityKind kind) => kind switch
        {
            GameEntityKind.Definition => "definition",
            GameEntityKind.Collection => "collection",
            _ => throw new ArgumentOutOfRangeException(nameof(kind), kind, null),
        };

        /// <summary>Returns the REST path segment (<c>game-definitions</c> / <c>game-collections</c>).</summary>
        /// <param name="kind">Entity kind</param>
        /// <returns>The path segment</returns>
        public static string ToPathSegment(this GameEntityKind kind) => kind switch
        {
            GameEntityKind.Definition => "game-definitions",
            GameEntityKind.Collection => "game-collections",
            _ => throw new ArgumentOutOfRangeException(nameof(kind), kind, null),
        };
    }
}
