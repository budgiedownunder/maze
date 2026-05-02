namespace Maze.Maui.App.Messages
{
    /// <summary>
    /// Sent after the password endpoint succeeds (set or change). Lets
    /// other ViewModels learn that <c>HasPassword</c> is now <c>true</c>
    /// without re-fetching the profile from the server.
    /// </summary>
    public record PasswordSetMessage();
}
