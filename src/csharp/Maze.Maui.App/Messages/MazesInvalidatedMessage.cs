namespace Maze.Maui.App.Messages
{
    /// <summary>
    /// Sent when the cached maze list is no longer trustworthy and must
    /// be reloaded from the server. Fires on sign-out and account
    /// deletion. Routed via <c>WeakReferenceMessenger</c> so producers
    /// don't depend on consumers — see also <see cref="NewMazeItemMessage"/>.
    /// </summary>
    public record MazesInvalidatedMessage();
}
