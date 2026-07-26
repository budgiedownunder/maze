using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// View model for the Home page — the post-sign-in landing. Hosts the
    /// commands wired to the Today's Challenge, 3D Games, Mazes and Leaderboards
    /// tiles.
    /// </summary>
    public partial class HomeViewModel : BaseViewModel
    {
        private readonly INavigationService _navigationService;
        private readonly IGameLibraryService _gameLibrary;
        private readonly IDialogService _dialogService;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="gameLibrary">Injected game-library read service (daily-challenge lookup)</param>
        /// <param name="dialogService">Injected dialog service (nothing-to-play / error alerts)</param>
        public HomeViewModel(
            INavigationService navigationService,
            IGameLibraryService gameLibrary,
            IDialogService dialogService)
        {
            Title = "Home";
            _navigationService = navigationService;
            _gameLibrary = gameLibrary;
            _dialogService = dialogService;
        }

        /// <summary>
        /// Resolves and plays today's daily challenge: find the curated "Daily
        /// Challenges" collection in the featured catalogue and launch its daily
        /// member (the host page date-mixes the seed for today, UTC). Alerts when
        /// there is nothing to play, or when the lookup fails.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task PlayTodaysChallengeAsync()
        {
            if (IsBusy)
                return;

            IsBusy = true;
            try
            {
                await DailyChallengeLauncher.LaunchAsync(_gameLibrary, _navigationService, _dialogService);
            }
            finally
            {
                IsBusy = false;
            }
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
