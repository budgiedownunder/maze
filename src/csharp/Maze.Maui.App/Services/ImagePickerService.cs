namespace Maze.Maui.App.Services
{
    /// <summary>
    /// <see cref="IImagePickerService"/> backed by MAUI's <c>MediaPicker</c>
    /// (the gallery on mobile, a file dialog on desktop). Reads the picked file
    /// into memory and reports its content type (from the platform, falling back
    /// to the file extension). App-only — kept out of the file-linked test
    /// surface because <c>MediaPicker</c> is a MAUI runtime API.
    /// </summary>
    public class ImagePickerService : IImagePickerService
    {
        /// <inheritdoc/>
        public async Task<PickedImage?> PickImageAsync()
        {
            try
            {
                // PickPhotosAsync is the non-obsolete API; an avatar is a single
                // image, so take the first of the (possibly multi) selection.
                var picked = await MediaPicker.Default.PickPhotosAsync();
                FileResult? result = picked is { Count: > 0 } ? picked[0] : null;
                if (result is null)
                {
                    return null; // user cancelled
                }

                using Stream stream = await result.OpenReadAsync();
                using var memory = new MemoryStream();
                await stream.CopyToAsync(memory);

                string contentType = string.IsNullOrEmpty(result.ContentType)
                    ? GuessContentType(result.FileName)
                    : result.ContentType;
                return new PickedImage(memory.ToArray(), contentType);
            }
            catch (FeatureNotSupportedException)
            {
                return null; // picking unsupported on this platform
            }
            catch
            {
                return null; // permission denied / read failure — caller shows the placeholder
            }
        }

        // Falls back to the file extension when the platform doesn't report a
        // content type. The server validates by decoding regardless, but the
        // client's PNG/JPEG check keys off this value.
        private static string GuessContentType(string? fileName)
        {
            string ext = Path.GetExtension(fileName ?? "").ToLowerInvariant();
            return ext switch
            {
                ".png" => "image/png",
                ".jpg" or ".jpeg" => "image/jpeg",
                _ => "application/octet-stream",
            };
        }
    }
}
