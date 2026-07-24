using Maze.Maui.App.Views;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Shared launch entry points for the Play 3D page — the single place the
    /// browser cards, collection pickers, and leaderboard Play actions route a
    /// launch through, so they all navigate identically. The page re-fetches the
    /// definition's config from its id, so a bare navigate is all that's needed.
    /// </summary>
    public static class Play3dLauncher
    {
        /// <summary>
        /// Navigates to the Play 3D page for a stored game definition, forwarding
        /// its id so the page assembles the <c>/game/?def=&lt;id&gt;</c> URL.
        /// </summary>
        /// <param name="navigationService">The navigation service</param>
        /// <param name="definitionId">The game definition id to launch</param>
        /// <returns>Task</returns>
        public static Task LaunchDefinitionAsync(INavigationService navigationService, string definitionId)
            => navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
            {
                { "def", definitionId },
            });
    }
}
