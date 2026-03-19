using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A popup that displays information about the application.
    /// </summary>
    public partial class AboutPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public AboutPopup()
        {
            InitializeComponent();
        }

        /// <summary>
        /// Handles the Close button click.
        /// </summary>
        private async void OnCloseClicked(object sender, EventArgs e)
        {
            await CloseAsync();
        }
    }
}
