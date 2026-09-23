namespace Maze.Maui.App.Utils;

/// <summary>
/// Character caps for the text fields the server stores, mirroring
/// <c>MAX_USERNAME_CHARS</c> / <c>MAX_NAME_CHARS</c> / <c>MAX_EMAIL_CHARS</c> in
/// <c>src/rust/storage/src/validation.rs</c>. The server rejects an over-long
/// value with a 400 naming the limit; entries and prompts carry the same caps so
/// the limit is reached in the field rather than on submit.
/// </summary>
public static class FieldLimits
{
    /// <summary>Longest username.</summary>
    public const int Username = 64;

    /// <summary>Longest full name, or maze, game or collection name.</summary>
    public const int Name = 255;

    /// <summary>Longest email address.</summary>
    public const int Email = 254;
}
