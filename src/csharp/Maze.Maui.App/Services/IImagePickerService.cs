namespace Maze.Maui.App.Services
{
    /// <summary>
    /// An image the user picked: its raw bytes and declared content type. A
    /// plain data record (no MAUI runtime types) so the view-model layer can
    /// consume it without depending on the platform picker.
    /// </summary>
    public record PickedImage(byte[] Bytes, string ContentType);

    /// <summary>
    /// Abstracts the platform image picker (MAUI <c>MediaPicker</c>) so the
    /// avatar-upload flow stays testable: the view model depends on this
    /// interface, not on the MAUI runtime API directly.
    /// </summary>
    public interface IImagePickerService
    {
        /// <summary>
        /// Prompts the user to pick an image from their device. Returns the
        /// picked image, or <c>null</c> if the user cancelled, the platform
        /// doesn't support picking, or the pick failed.
        /// </summary>
        Task<PickedImage?> PickImageAsync();
    }
}
