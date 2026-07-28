namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Pure helpers that assemble the hosted <c>/game/</c> WebView URL for the
    /// Play 3D page's launch branches (stored maze, stored game definition, or a
    /// bare authenticated launch). Kept free of HTTP /
    /// WebView dependencies so the base-URL derivation and query-string assembly
    /// are unit-testable in isolation (<see cref="Views.Play3dGamePage"/> delegates to
    /// these). The host page lives alongside the API — its base is the API root
    /// with the trailing <c>/api/…</c> segment stripped and <c>/game/</c> appended.
    /// The bearer token rides as <c>&amp;t=</c> and is passed through verbatim
    /// (JWTs are already URL-safe), matching every other launch path.
    /// </summary>
    public static class Play3dGameHostUrl
    {
        /// <summary>
        /// Derives the hosted <c>/game/</c> base URL from the API root by stripping
        /// the <c>/api/…</c> path off it (falling back to appending <c>game/</c>
        /// when no <c>/api/</c> segment is present).
        /// </summary>
        /// <param name="apiRootUri">The configured API root URI</param>
        /// <returns>The <c>/game/</c> base URL (no query string)</returns>
        public static string BuildBaseUrl(string apiRootUri)
        {
            var apiIndex = apiRootUri.LastIndexOf("/api/", StringComparison.Ordinal);
            return apiIndex >= 0
                ? apiRootUri[..apiIndex] + "/game/"
                : apiRootUri + "game/";
        }

        /// <summary>
        /// Assembles the launch URL for a specific stored maze
        /// (<c>?id=&lt;mazeId&gt;&amp;t=&lt;token&gt;</c>), optionally appending the
        /// user's per-launch settings query so <c>/game/index.html</c> can override
        /// its <c>StartConfig</c> from the URL.
        /// </summary>
        /// <param name="apiRootUri">The configured API root URI</param>
        /// <param name="mazeId">The stored maze id</param>
        /// <param name="bearerToken">The bearer token, or <c>null</c> when signed out</param>
        /// <param name="settingsQuery">The per-launch settings query string (e.g. from <c>MazeGameSettings.ToQueryString</c>), or <c>null</c>/empty</param>
        /// <returns>The full hosted game URL</returns>
        public static string BuildForMaze(string apiRootUri, string mazeId, string? bearerToken, string? settingsQuery)
        {
            var url = BuildBaseUrl(apiRootUri) + $"?id={Uri.EscapeDataString(mazeId)}";
            if (bearerToken is not null) url += $"&t={bearerToken}";
            if (!string.IsNullOrEmpty(settingsQuery)) url += "&" + settingsQuery;
            return url;
        }

        /// <summary>
        /// Assembles the launch URL for a stored game definition
        /// (<c>?def=&lt;id&gt;&amp;t=&lt;token&gt;</c>). The host page re-fetches the
        /// definition's config itself, so only the id and token travel here.
        /// </summary>
        /// <param name="apiRootUri">The configured API root URI</param>
        /// <param name="definitionId">The game definition id</param>
        /// <param name="bearerToken">The bearer token, or <c>null</c> when signed out</param>
        /// <returns>The full hosted game URL</returns>
        public static string BuildForDefinition(string apiRootUri, string definitionId, string? bearerToken)
        {
            var url = BuildBaseUrl(apiRootUri) + $"?def={Uri.EscapeDataString(definitionId)}";
            if (bearerToken is not null) url += $"&t={bearerToken}";
            return url;
        }

        /// <summary>
        /// Assembles the bare authenticated launch URL (<c>?t=&lt;token&gt;</c>) used
        /// when no specific subject is supplied, or just the base URL when signed out.
        /// </summary>
        /// <param name="apiRootUri">The configured API root URI</param>
        /// <param name="bearerToken">The bearer token, or <c>null</c> when signed out</param>
        /// <returns>The full hosted game URL</returns>
        public static string BuildForToken(string apiRootUri, string? bearerToken)
        {
            var baseUrl = BuildBaseUrl(apiRootUri);
            return bearerToken is not null ? baseUrl + $"?t={bearerToken}" : baseUrl;
        }
    }
}
