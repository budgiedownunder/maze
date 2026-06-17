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

        /// <summary>
        /// Uploads (or replaces) the caller's avatar. The server canonicalises
        /// the image to a 256x256 PNG and returns the new <c>avatar_updated_at</c>
        /// marker. Returns <c>null</c> on failure.
        /// </summary>
        /// <param name="bytes">The image bytes (PNG or JPEG).</param>
        /// <param name="contentType">The image's content type.</param>
        Task<string?> UploadAvatarAsync(byte[] bytes, string contentType);

        /// <summary>
        /// Removes the caller's avatar. Returns <c>true</c> on success (including
        /// the idempotent no-op when there was none), <c>false</c> on failure.
        /// </summary>
        Task<bool> DeleteAvatarAsync();
    }
}
