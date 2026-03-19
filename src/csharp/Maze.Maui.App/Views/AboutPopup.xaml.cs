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
            double screenWidth = DeviceDisplay.Current.MainDisplayInfo.Width
                / DeviceDisplay.Current.MainDisplayInfo.Density;
            WidthRequest = Math.Min(screenWidth * 0.85, 400);
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
