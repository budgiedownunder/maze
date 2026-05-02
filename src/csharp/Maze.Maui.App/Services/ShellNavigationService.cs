namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Production <see cref="INavigationService"/> that delegates to
    /// <c>Shell.Current</c>. Registered as a singleton in MauiProgram.
    /// </summary>
    public class ShellNavigationService : INavigationService
    {
        /// <inheritdoc/>
        public Task GoBackAsync() => Shell.Current.GoToAsync("..");
    }
}
