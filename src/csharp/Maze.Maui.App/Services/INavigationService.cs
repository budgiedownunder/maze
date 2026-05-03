namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Abstracts MAUI Shell navigation behind an interface so ViewModels
    /// don't depend on <c>Shell.Current</c> directly. The Shell static
    /// can't be exercised inside a non-MAUI test host; routing through
    /// this interface lets tests inject a mock and verify navigation
    /// steps. See <c>ShellNavigationService</c> for the production impl.
    /// </summary>
    public interface INavigationService
    {
        /// <summary>Navigates back one step in the Shell stack
        /// (equivalent to <c>Shell.Current.GoToAsync("..")</c>).</summary>
        Task GoBackAsync();

        /// <summary>Navigates to a named route, optionally with a
        /// dictionary of query parameters that the destination page
        /// will receive via its <c>[QueryProperty]</c> attributes.</summary>
        Task GoToAsync(string route, IDictionary<string, object>? parameters = null);

        /// <summary>Navigates to the named root page (e.g.
        /// <c>"//MainPage"</c> or <c>"//LoginPage"</c>) which resets
        /// the Shell navigation stack to that root.</summary>
        Task GoToRootAsync(string route);
    }
}
