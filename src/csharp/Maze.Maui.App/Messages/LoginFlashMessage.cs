namespace Maze.Maui.App.Messages
{
    /// <summary>
    /// Sent to surface a transient status message on the Sign In page —
    /// today the only sender is <c>SignUpViewModel</c>, which fires it
    /// after a successful signup so the user knows to check their inbox
    /// for a verification email before attempting to sign in. Routed via
    /// <c>WeakReferenceMessenger</c> so the sender doesn't hold a
    /// reference to the receiver across the navigation stack pop.
    /// </summary>
    /// <param name="Message">User-facing copy to render below the form.</param>
    public record LoginFlashMessage(string Message);
}
