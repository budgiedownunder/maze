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

        /// <inheritdoc/>
        public Task GoToAsync(string route, IDictionary<string, object>? parameters = null)
            => parameters is null
                ? Shell.Current.GoToAsync(route)
                : Shell.Current.GoToAsync(route, parameters);

        /// <inheritdoc/>
        public Task GoToRootAsync(string route) => Shell.Current.GoToAsync(route);
    }
}
