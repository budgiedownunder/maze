using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Displays the result of a completed maze game session — a win shows the
    /// animated celebration sprite, a loss shows the animated game-over sprite —
    /// plus a result message and a Close button.
    /// </summary>
    public partial class GameResultPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="message">Result message to display below the result image</param>
        /// <param name="won">Whether the game was won (celebration sprite) or lost (game-over sprite)</param>
        public GameResultPopup(string message, bool won)
        {
            InitializeComponent();
            MessageLabel.Text = message;
            // Both outcomes use an animated GIF, so animation always plays.
            ResultImage.Source = won ? "celebrate.gif" : "game_over.gif";
            ResultImage.IsAnimationPlaying = true;
        }

        /// <summary>Closes the popup, signalling that the player wants to play the maze again.</summary>
        private async void OnPlayAgainClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync<bool>(true);

        /// <summary>Closes the popup without replaying.</summary>
        private async void OnCloseClicked(object? sender, EventArgs e) => await Navigation.ClosePopupAsync<bool>(false);
    }
}
