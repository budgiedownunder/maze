using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// The 3D Games hub — the landing that offers the four browse scopes
    /// (Featured · My Games · Shared with me · Community). Featured is live; the
    /// other three are placeholders until their sub-pages ship.
    /// </summary>
    public partial class Play3dHubViewModel : BaseViewModel
    {
        private readonly INavigationService _navigationService;

        /// <summary>Constructor</summary>
        /// <param name="navigationService">Injected navigation service</param>
        public Play3dHubViewModel(INavigationService navigationService)
        {
            Title = "3D Games";
            _navigationService = navigationService;
        }

        /// <summary>Opens the Featured sub-page.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task GoToFeatured() => _navigationService.GoToAsync("Play3dFeaturedPage");

        /// <summary>Opens the My Games sub-page.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task GoToMyGames() => _navigationService.GoToAsync("Play3dMyGamesPage");

        /// <summary>Opens the Shared with me sub-page.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task GoToShared() => _navigationService.GoToAsync("Play3dSharedPage");

        /// <summary>Opens the Community sub-page.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task GoToCommunity() => _navigationService.GoToAsync("Play3dCommunityPage");
    }
}
