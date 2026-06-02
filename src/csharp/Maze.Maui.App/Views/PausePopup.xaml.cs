using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The 2D game pause menu — offers Resume and Restart. Shown while the game
    /// is paused; closing it returns the chosen <see cref="PauseMenuResult"/>.
    /// </summary>
    public partial class PausePopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public PausePopup()
        {
            InitializeComponent();
        }

        /// <summary>Closes the popup, signalling that the player wants to resume.</summary>
        private async void OnResumeClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync(PauseMenuResult.Resume);

        /// <summary>Closes the popup, signalling that the player wants to restart the maze.</summary>
        private async void OnRestartClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync(PauseMenuResult.Restart);
    }
}
