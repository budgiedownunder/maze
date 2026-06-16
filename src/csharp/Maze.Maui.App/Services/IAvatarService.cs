namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Loads user avatar images over the bearer-authenticated API. The avatar
    /// serve route (<c>GET /users/{id}/avatar</c>) is guarded, so a plain image
    /// URL can't reach it — this fetches the raw bytes through the auth pipeline.
    /// Returns the bytes (not a UI <c>ImageSource</c>) so the view-model layer
    /// stays free of MAUI runtime types; the <c>AvatarView</c> control turns the
    /// bytes into an image and shows the placeholder when there are none.
    /// </summary>
    public interface IAvatarService
    {
        /// <summary>
        /// Loads the avatar PNG bytes for <paramref name="userId"/>, using
        /// <paramref name="avatarUpdatedAt"/> as the cache-buster. Returns
        /// <c>null</c> when the user has no avatar (no marker, or a 404) or on
        /// any failure — callers then show the generic placeholder.
        /// </summary>
        /// <param name="userId">The user whose avatar to load.</param>
        /// <param name="avatarUpdatedAt">The user's <c>avatar_updated_at</c>
        /// marker; <c>null</c>/empty means the user has no avatar.</param>
        Task<byte[]?> TryLoadAvatarBytesAsync(string userId, string? avatarUpdatedAt);
    }
}
