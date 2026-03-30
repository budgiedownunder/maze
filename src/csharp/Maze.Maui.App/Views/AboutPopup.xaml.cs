using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A popup that displays information about the application.
    /// This is how the popup appears on Windows Desktop:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Windows</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/windows-about.png" height="500" width="500"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    ///  
    /// and this is how it appears on Android/iOS devices:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Android</strong></th>
    ///         <th><strong>iOS</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/android-about.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/ios-about.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
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
