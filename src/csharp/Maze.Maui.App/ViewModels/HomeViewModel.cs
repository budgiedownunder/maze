using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// View model for the Home page — the post-sign-in landing. Hosts the
    /// commands wired to the 3D Games, Mazes and Leaderboards tiles.
    /// </summary>
    public partial class HomeViewModel : BaseViewModel
    {
        private readonly INavigationService _navigationService;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="navigationService">Injected navigation service</param>
        public HomeViewModel(INavigationService navigationService)
        {
            Title = "Home";
            _navigationService = navigationService;
        }

        /// <summary>
        /// Opens the 3D Games browser hub (Featured / My Games / Shared / Community).
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task GoTo3dGamesAsync()
        {
            // String route (not nameof) — the test project file-links this view
            // model but not the Page types, so the symbols aren't in scope there.
            await _navigationService.GoToAsync("Play3dHubPage");
        }

        /// <summary>
        /// Navigates to the maze list page.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task GoToMazesAsync()
        {
            // String route (not nameof) — the test project file-links this view
            // model but not the Page types, so the symbols aren't in scope there.
            await _navigationService.GoToAsync("MazesPage");
        }

        /// <summary>
        /// Navigates to the Leaderboards page.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task GoToLeaderboardsAsync()
        {
            await _navigationService.GoToAsync("LeaderboardsPage");
        }
    }
}
