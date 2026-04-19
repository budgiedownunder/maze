using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Displays the result of a completed maze game session — celebration sprite,
    /// a result message, and a Close button. Extensible for future game result data
    /// such as points scored and awards won.
    /// </summary>
    public partial class GameResultPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="message">Result message to display below the celebration sprite</param>
        public GameResultPopup(string message)
        {
            InitializeComponent();
            MessageLabel.Text = message;
        }

        private async void OnCloseClicked(object? sender, EventArgs e) => await CloseAsync();
    }
}
