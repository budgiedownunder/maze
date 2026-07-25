using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// View model for the Home page — the post-sign-in landing. Hosts the
    /// commands wired to the Play 3D and Design and Play tiles.
    /// </summary>
    public partial class HomeViewModel : BaseViewModel
    {
        private readonly INavigationService _navigationService;
        private readonly IDialogService _dialogService;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service (Play 3D difficulty picker)</param>
        public HomeViewModel(INavigationService navigationService, IDialogService dialogService)
        {
            Title = "Home";
            _navigationService = navigationService;
            _dialogService = dialogService;
        }

        /// <summary>
        /// Launches a 3D game. Prompts the user for a difficulty, then
        /// navigates to the Bevy 3D page with that difficulty — the page
        /// appends it to the WebView URL and the server resolves the preset.
        /// Cancelling the picker stays on the Home page.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task PlayRandom3dAsync()
        {
            var difficulty = await _dialogService.ShowPlay3dDifficultyAsync();
            if (difficulty is null) return;

            await _navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
            {
                { "difficulty", difficulty.Value.ToQueryValue() },
            });
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
