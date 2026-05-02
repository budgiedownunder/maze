namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Abstracts MAUI Shell navigation behind an interface so ViewModels
    /// don't depend on <c>Shell.Current</c> directly. The Shell static
    /// can't be exercised inside a non-MAUI test host; routing through
    /// this interface lets tests inject a mock and verify the back-nav
    /// step. See <c>ShellNavigationService</c> for the production impl.
    /// </summary>
    public interface INavigationService
    {
        /// <summary>Navigates back one step in the Shell stack
        /// (equivalent to <c>Shell.Current.GoToAsync("..")</c>).</summary>
        Task GoBackAsync();
    }
}
