using CommunityToolkit.Mvvm.Input;
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
        /// Launches a random 3D game. Navigates to the Bevy 3D page without
        /// a MazeItem parameter — the page interprets the absence of a maze
        /// id as random-game mode.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task PlayRandom3dAsync()
        {
            await _navigationService.GoToAsync(nameof(Play3dGamePage));
        }

        /// <summary>
        /// Navigates to the maze list page (Design and Play).
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        async Task GoToDesignAndPlayAsync()
        {
            await _navigationService.GoToAsync("MazesPage");
        }
    }
}
