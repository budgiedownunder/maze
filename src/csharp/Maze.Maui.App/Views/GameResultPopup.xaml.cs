using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Displays the result of a completed maze game session — a win shows the
    /// animated celebration sprite, a loss shows the game-over (skull) image —
    /// plus a result message and a Close button.
    /// </summary>
    public partial class GameResultPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="message">Result message to display below the result image</param>
        /// <param name="won">Whether the game was won (celebration sprite) or lost (game-over image)</param>
        public GameResultPopup(string message, bool won)
        {
            InitializeComponent();
            MessageLabel.Text = message;
            ResultImage.Source = won ? "celebrate.gif" : "game_over.png";
            ResultImage.IsAnimationPlaying = won;
        }

        /// <summary>Closes the popup, signalling that the player wants to play the maze again.</summary>
        private async void OnPlayAgainClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync<bool>(true);

        /// <summary>Closes the popup without replaying.</summary>
        private async void OnCloseClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync<bool>(false);
    }
}
